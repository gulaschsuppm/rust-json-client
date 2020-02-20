use std::net::UdpSocket;
use json;
use std::fs::File;
use std::io::{self, BufRead};
use chrono::{NaiveDateTime, Duration};
use std::thread;

fn main() -> std::io::Result<()> {
    {
        let file = File::open("event.json")?;
        let lines = io::BufReader::new(file).lines();

        let mut event_vec = Vec::new();

        let mut old_time: Option<NaiveDateTime> = None;

        // println!("{}", old_time);

        for line in lines {
            let json_msg = json::parse(line?.as_str()).unwrap();

            // println!("{}", json_msg);

            let event_time = json_msg["msg"]["EventTime"].as_str().unwrap();

            // println!("{}", event_time);

            let parsed_time = NaiveDateTime::parse_from_str(event_time, "%Y-%b-%d %H:%M:%S.%f").unwrap();
            // println!("{}", parsed_time);

            let mut diff = Duration::milliseconds(0);
            if let Some(_i) = old_time {
                diff = parsed_time.signed_duration_since(old_time.unwrap());
            }

            // println!("{}", diff);

            event_vec.push((json_msg.to_string(), diff));

            // println!("{} {}", event_vec.last().unwrap().0, event_vec.last().unwrap().1);

            old_time = Some(parsed_time);
        }

        let socket = UdpSocket::bind("127.0.0.1:34254")?;

        for event in event_vec {
            thread::sleep(event.1.to_std().unwrap());
            println!("{}", event.0);
            socket.send_to(event.0.as_bytes(), "127.0.0.1:4242")?;
        }
    }
    Ok(())
}
