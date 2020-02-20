use std::env;

mod input_parsing;

mod scenario_parser {
    use std::fs::File;
    use std::io::{self, BufRead};
    use chrono::{NaiveDateTime, Duration};

    pub fn json_to_event_vector(path: String) -> std::io::Result<Vec<(String, Duration)>> {
        let vector = read_file(path)?;

        let mut old_time: Option<NaiveDateTime> = None;

        let vector = vector.into_iter().map(|(msg, _delay)| {
            let parsed_time = get_time(&msg).unwrap();

            let mut diff = Duration::milliseconds(0);
            if let Some(_i) = old_time {
                diff = parsed_time.signed_duration_since(old_time.unwrap());
            }

            old_time = Some(parsed_time);

            (msg.clone(), diff)
        }).collect();

        Ok(vector)
    }

    fn read_file(path: String) -> std::io::Result<Vec<(String, Duration)>> {
        let file = File::open(path)?;

        let vector: Vec<(String, Duration)> = io::BufReader::new(file)
            .lines()
            .map(|x| (x.unwrap(), Duration::milliseconds(0)))
            .collect();
        Ok(vector)
    }

    fn get_time(json_msg: &String) -> io::Result<NaiveDateTime> {
        let json_msg = json::parse(json_msg.as_str()).unwrap();

        let event_time = json_msg["msg"]["EventTime"].as_str().unwrap();

        Ok(NaiveDateTime::parse_from_str(event_time, "%Y-%b-%d %H:%M:%S.%f").unwrap())
    }
}

mod net_client {
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

                    println!("{:?}", &buf[..i.0]);

                    out_file.write_all(&buf[..i.0]).unwrap();
                }

                if let Ok(_) = rx.try_recv() {
                    return
                }
            }
        });

        for event in events {
            thread::sleep(event.1.to_std().unwrap());
            println!("{}", event.0);
            socket.send_to(event.0.as_bytes(), format!("127.0.0.1:{}", oport))?;
        }

        tx.send(true).unwrap();

        recv_thread.join().unwrap();

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    {
        let args: Vec<String> = env::args().collect();

        let args = input_parsing::parse_args(args);

        println!("Parsing {}, sending events to port {} and receiving at port {}", args.0, args.1, args.2);

        let json_events = scenario_parser::json_to_event_vector(args.0)?;

        net_client::run(args.2, args.1, json_events).unwrap();
    }
    Ok(())
}
