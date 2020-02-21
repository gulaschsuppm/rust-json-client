use chrono::Duration;
use std::net::{TcpListener, SocketAddr, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::fs::File;
use std::io::{self, Write, Read};

pub fn run(iport: &String, events: &Vec<(String, Duration)>) -> io::Result<()> {
    let (mut socket, _) = connect_tcp(&iport)?;

    let mut socket_clone = socket.try_clone()?;

    let (tx, rx) = mpsc::channel();

    let recv_thread = thread::spawn(move || {
        let mut out_file = File::create("out.json").unwrap();
        let mut buf = [0; 256];

        socket_clone.set_read_timeout(Some(std::time::Duration::from_secs(10))).unwrap();

        loop {
            if let Ok(i) = socket_clone.read(&mut buf) {
                if i == 256 {
                    println!("Received {} bytes, probably lost some...", i);
                }

                println!("Received: {:?}", &buf[..i]);

                out_file.write_all(&buf[..i]).unwrap();
            }

            if let Ok(_) = rx.try_recv() {
                return
            }
        }
    });

    for event in events {
        thread::sleep(event.1.to_std().unwrap());
        println!("Sending: {}", event.0);
        socket.write(event.0.as_bytes())?;
    }
    tx.send(true).unwrap();

    recv_thread.join().unwrap();

    Ok(())
}

/// Will block
fn connect_tcp(iport: &String) -> io::Result<(TcpStream, SocketAddr)> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", iport))?;

    listener.accept()
}


#[cfg(test)]
mod run {
    // use super::*;
    //
    // #[test]
    // fn port_not_open() {
    //     let events: Vec<(String, Duration)> = Vec::new();
    //
    //     // Go for 443 (HTTPS) but no guarantee...
    //     let res = run(String::from("443"), String::from("4242"), events);
    //
    //     assert!(res.is_err());
    // }
    //
    // #[test]
    // fn receive_event() {
    //     let first_string = String::from("First message");
    //     let second_string = String::from("Second message");
    //     let events: Vec<(String, Duration)> = vec![(first_string.clone(), Duration::seconds(0)), (second_string.clone(), Duration::seconds(0))];
    //
    //     let test_socket = UdpSocket::bind("127.0.0.1:4242").unwrap();
    //
    //     let res = run(String::from("4343"), String::from("4242"), events);
    //
    //     let first_msg =
    //
    //     assert!(res.is_err());
    // }
}