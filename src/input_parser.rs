use getopts::Options;

pub fn parse_args(args: Vec<String>) -> (String, String, String) {
    if args.len() < 1 {
        panic!("Need to have at least the program as argument.");
    }
    // Set default values
    let mut file = String::from("events.json");
    let mut oport = String::from("4242");
    let mut iport = String::from("34254");

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
        panic!("{}", opts.usage(&brief));
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



#[cfg(test)]
mod parse_args {
    use super::*;

    #[test]
    #[should_panic]
    fn invalid_num_of_args() {
        let args = Vec::new();

        let _ = parse_args(args);
    }

    #[test]
    fn no_args_received() {
        let args = vec![String::from("self")];

        let (file, oport, iport) = parse_args(args);

        assert_eq!(file, "events.json");
        assert_eq!(iport, "34254");
        assert_eq!(oport, "4242");
    }

    #[test]
    fn args_received() {
        let args = vec![
            String::from("self"),
            String::from("-f"),
            String::from("file.json"),
            String::from("-i"),
            String::from("1111"),
            String::from("-o"),
            String::from("2222")
        ];

        let (file, oport, iport) = parse_args(args);

        assert_eq!(file,  "file.json");
        assert_eq!(iport, "1111");
        assert_eq!(oport, "2222");
    }
}
