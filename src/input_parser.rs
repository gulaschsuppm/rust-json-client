use getopts::Options;

pub fn parse_args(args: &Vec<String>) -> (String, String) {
    if args.len() < 1 {
        panic!("Need to have at least the program as argument.");
    }
    // Set default values
    let mut file = String::from("events.json");
    let mut port = String::from("4242");

    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("f", "file", "file containing JSON formatted events, line separated, default: events.json", "FILE");
    opts.optopt("p", "port", "TCP socket port, default: 4242", "PORT");

    let matches = opts.parse(&args[1..]).unwrap();

    // If the help option is contained, print the usage and exit gracefully
    if matches.opt_present("h") {
        let brief = format!("Usage: {} [-h] [-f FILE] [-o PORT] [-i PORT]", program);
        panic!("{}", opts.usage(&brief));
    }

    if let Some(i) = matches.opt_str("f") {
        file = i;
    }

    if let Some(i) = matches.opt_str("p") {
        port = i;
    }

    (file, port)
}



#[cfg(test)]
mod parse_args {
    use super::*;

    #[test]
    #[should_panic]
    fn invalid_num_of_args() {
        let args = Vec::new();

        let _ = parse_args(&args);
    }

    #[test]
    fn no_args_received() {
        let args = vec![String::from("self")];

        let (file, port) = parse_args(&args);

        assert_eq!(file, "events.json");
        assert_eq!(port, "4242");
    }

    #[test]
    fn args_received() {
        let args = vec![
            String::from("self"),
            String::from("-f"),
            String::from("file.json"),
            String::from("-p"),
            String::from("1111"),
        ];

        let (file, port) = parse_args(&args);

        assert_eq!(file, "file.json");
        assert_eq!(port, "1111");
    }
}
