use std::net::{UdpSocket, SocketAddr};
use json;
use std::fs::File;
use std::io::{self, BufRead, Write, Error};
use chrono::{NaiveDateTime, Duration};
use std::thread;
use getopts::Options;
use std::env;
use std::process::exit;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;

fn parse_args() -> (String, String, String) {
    // Set default values
    let mut file = String::from("events.json");
    let mut oport = String::from("4242");
    let mut iport = String::from("34254");

    let args: Vec<String> = env::args().collect();

    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("f", "file", "file containing JSON formatted events, line separated. Default: events.json", "FILE");
    opts.optopt("o", "outport", "socket port for output. Default: 4242", "PORT");
    opts.optopt("i", "inport", "socket port for input. Default: 34254", "PORT");

    let matches = opts.parse(&args[1..]).unwrap();

    // If the help option is contained, print the usage and exit gracefully
    if matches.opt_present("h") {
        let brief = format!("Usage: {} [-h] [-f FILE] [-o PORT] [-i PORT]", program);
        print!("{}", opts.usage(&brief));
        exit(0);
    }

    if let Some(i) = matches.opt_str("f") {
        file = i;
    }

    if let Some(i) = matches.opt_str("o") {
        oport = i;
    }

    if let Some(i) = matches.opt_str("i") {
        iport = i;
    }

    (file, oport, iport)
}

fn main() -> std::io::Result<()> {
    {
        let args = parse_args();

        println!("Parsing {}, sending events to port {} and receiving at port {}", args.0, args.1, args.2);

        let file = File::open(args.0)?;
        let lines = io::BufReader::new(file).lines();

        let mut event_vec = Vec::new();

        let mut old_time: Option<NaiveDateTime> = None;

        for line in lines {
            let json_msg = json::parse(line?.as_str()).unwrap();

            let event_time = json_msg["msg"]["EventTime"].as_str().unwrap();

            let parsed_time = NaiveDateTime::parse_from_str(event_time, "%Y-%b-%d %H:%M:%S.%f").unwrap();

            let mut diff = Duration::milliseconds(0);
            if let Some(_i) = old_time {
                diff = parsed_time.signed_duration_since(old_time.unwrap());
            }

            event_vec.push((json_msg.to_string(), diff));

            old_time = Some(parsed_time);
        }

        let socket = UdpSocket::bind(format!("127.0.0.1:{}", args.2))?;

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

                    out_file.write_all(&buf[..i.0]);
                }

                if let Ok(_) = rx.try_recv() {
                    return
                }
            }
        });

        for event in event_vec {
            thread::sleep(event.1.to_std().unwrap());
            println!("{}", event.0);
            socket.send_to(event.0.as_bytes(), format!("127.0.0.1:{}", args.1))?;
        }

        tx.send(true).unwrap();

        recv_thread.join().unwrap();
    }
    Ok(())
}
