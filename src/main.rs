use std::env;

mod input_parser;
mod scenario_parser;
mod net_client;


fn main() -> std::io::Result<()> {
    {
        let args: Vec<String> = env::args().collect();

        let args = input_parser::parse_args(args);

        println!("Parsing {}, sending events to port {} and receiving at port {}", args.0, args.1, args.2);

        let json_events = scenario_parser::json_to_event_vector(args.0)?;

        net_client::run(args.2, args.1, json_events).unwrap();
    }
    Ok(())
}
