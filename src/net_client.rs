use chrono::Duration;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::fs::File;
use std::io::{self, Write, Read};

pub fn run(port: &String, events: &Vec<(String, Duration)>) -> io::Result<()> {
    let mut socket = connect_tcp(&port)?;

    let mut socket_clone = socket.try_clone()?;

    let (tx, rx) = mpsc::channel();

    let recv_thread = thread::spawn(move || {
        let mut out_file = File::create("out.json").unwrap();
        let mut buf = [0; 1024];

        socket_clone.set_read_timeout(Some(std::time::Duration::from_secs(10))).unwrap();

        loop {
            if let Ok(i) = socket_clone.read(&mut buf) {
                if i == 1024 {
                    println!("Received {} bytes, probably lost some...", i);
                }

                let frames = read_frames(&buf, i);

                for frame in frames {
                    println!("Received: {:?}", frame);
                    out_file.write_all(frame.as_slice()).unwrap();
                }
            }

            if let Ok(_) = rx.try_recv() {
                return
            }
        }
    });

    for (event, dur) in events {
        thread::sleep(dur.to_std().unwrap());
        println!("Sending: {}", &event);
        let len = event.len() as u8 as char;

        let mut frame = event.clone();
        frame.insert(0, '\x02');
        frame.insert(1, len);
        frame.insert(frame.len(), '\x03');

        socket.write(frame.as_bytes())?;
        socket.flush()?;
    }

    tx.send(true).unwrap();

    recv_thread.join().unwrap();

    Ok(())
}

/// Will block
fn connect_tcp(port: &String) -> io::Result<TcpStream> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;

    let (socket, _) = listener.accept()?;

    socket.set_nodelay(true)?;

    Ok(socket)
}

fn read_frames(buffer: &[u8; 1024], size: usize) -> Vec<Vec<u8>> {
    let mut pos = 0;
    let mut frames: Vec<Vec<u8>> = Vec::new();

    while pos < size {
        if buffer[pos] == 2 {
            pos += 1;
            let size = buffer[pos] as usize;
            pos += 1;

            let mut frame = Vec::new();

            frame.extend_from_slice(&buffer[pos..pos+size]);

            println!("{:?}", frame);

            frames.push(frame);

            if buffer[pos + size] != 3 {
                panic!("Frame mismatch, end not found");
            }
            pos += size + 1;
        } else {
            panic!("Frame mismatch, start not found");
        }
    }

    frames
}

#[cfg(test)]
mod run {
    use super::*;

    #[test]
    fn port_not_open() {
        let events: Vec<(String, Duration)> = Vec::new();
        let port = String::from("443");

        // Go for 443 (HTTPS) but no guarantee...
        let res = run(&port, &events);

        assert!(res.is_err());
    }

    #[test]
    fn send_events_with_no_duration() {
        let port = String::from("5665");

        let event_msgs = [
            String::from("First message"),
            String::from("Second message")
        ];

        // let mut seconds = 0;
        // let events: Vec<(String, Duration)> = event_msgs.iter().map(|msg| {
        //     let pair = (msg.clone(), chrono::Duration::seconds(seconds));
        //     seconds += 1;
        //     return pair
        // }).collect();

        let events: Vec<(String, Duration)> = event_msgs.iter().map(|msg| (msg.clone(), chrono::Duration::seconds(0))).collect();
        assert_eq!(events.len(), 2);

        let port_clone = port.clone();
        let run_thread = thread::spawn(move || {
            let _ = run(&port_clone, &events).unwrap();
        });

        // Wait for the TCP port to open.
        std::thread::sleep(std::time::Duration::from_secs(2));
        let mut tcp_socket = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();

        let mut buf = [0_u8; 1024];
        let size = tcp_socket.read(&mut buf).unwrap();

        run_thread.join().unwrap();

        let frames = read_frames(&buf, size);
        assert_eq!(frames.len(), 2);

        assert_eq!(frames[0].len(), event_msgs[0].len());
        assert_eq!(frames[1].len(), event_msgs[1].len());
    }
}

#[cfg(test)]
mod read_frames {
    use super::*;

    #[test]
    #[should_panic]
    fn no_start_marker() {
        let message = [0; 1024];
        let size = 5 as usize;

        let _ = read_frames(&message, size);
    }

    #[test]
    #[should_panic]
    fn no_end_marker() {
        let mut message = [0; 1024];
        let size = 5 as usize;

        message[0] = 2; // Start marker

        let _ = read_frames(&message, size);
    }

    #[test]
    #[should_panic]
    fn more_than_full_frame() {
        let mut message = [0; 1024];
        let size = 5 as usize;

        // Put a full message in but size is longer
        message[0] = 2; // Start marker
        message[1] = 1; // Length
        message[2] = 3; // End marker
        message[3] = 0; // No start marker here...

        let _ = read_frames(&message, size);
    }

    #[test]
    fn correct_empty_frame() {
        let mut message = [0; 1024];
        let size = 3 as usize;

        // Put a full message in but size is longer
        message[0] = 2; // Start marker
        message[1] = 0; // Length
        message[2] = 3; // End marker

        let res = read_frames(&message, size);

        assert_eq!(res.len(), 1);

        let expected_frame: Vec<u8> = vec![];
        assert_eq!(res[0].len(), 0);
        assert_eq!(res[0], expected_frame);
    }

    #[test]
    fn correct_single_frame() {
        let mut message = [4; 1024];
        let size = 35 as usize;

        // Put a full message in but size is longer
        message[0] = 2; // Start marker
        message[1] = 32; // Length
        message[34] = 3; // End marker

        let res = read_frames(&message, size);

        assert_eq!(res.len(), 1);

        let expected_frame: Vec<u8> = vec![4; 32];
        assert_eq!(res[0].len(), 32);
        assert_eq!(res[0], expected_frame);
    }

    #[test]
    fn correct_two_frames() {
        let mut message = [4; 1024];
        let size = 30 as usize;

        // Put a full message in but size is longer
        message[0] = 2; // Start marker
        message[1] = 16; // Length
        message[18] = 3; // End marker

        // Second message
        message[19] = 2; // Start marker
        message[20] = 8; // Length
        message[29] = 3; // End marker

        let res = read_frames(&message, size);

        assert_eq!(res.len(), 2);

        let expected_frame1: Vec<u8> = vec![4; 16];
        assert_eq!(res[0].len(), 16);
        assert_eq!(res[0], expected_frame1);

        let expected_frame2: Vec<u8> = vec![4; 8];
        assert_eq!(res[1].len(), 8);
        assert_eq!(res[1], expected_frame2);
    }
}
