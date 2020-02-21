use chrono::Duration;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::thread;
use std::fs::File;
use std::io::{self, Write};

pub fn run(iport: String, oport: String, events: Vec<(String, Duration)>) -> io::Result<()> {
    let socket = UdpSocket::bind(format!("127.0.0.1:{}", iport))?;

    let socket_clone = socket.try_clone().unwrap();

    let (tx, rx) = mpsc::channel();

    let recv_thread = thread::spawn(move || {
        let mut out_file = File::create("out.json").unwrap();
        let mut buf = [0; 256];

        socket_clone.set_read_timeout(Some(std::time::Duration::from_secs(10))).unwrap();

        loop {
            if let Ok(i) = socket_clone.recv_from(&mut buf) {
                if i.0 == 256 {
                    println!("Received {} bytes, probably lost some...", i.0);
                }

                println!("Received: {:?}", &buf[..i.0]);

                out_file.write_all(&buf[..i.0]).unwrap();
            }

            if let Ok(_) = rx.try_recv() {
                return
            }
        }
    });

    for event in events {
        thread::sleep(event.1.to_std().unwrap());
        println!("Sending: {}", event.0);
        socket.send_to(event.0.as_bytes(), format!("127.0.0.1:{}", oport))?;
    }
    tx.send(true).unwrap();

    recv_thread.join().unwrap();

    Ok(())
}