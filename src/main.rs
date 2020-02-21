use std::env;

mod input_parser;
mod scenario_parser;
mod net_client;


fn main() -> std::io::Result<()> {
    {
        let args: Vec<String> = env::args().collect();

        let (path, port) = input_parser::parse_args(&args);

        println!("Sending events from {} to 127.0.0.1:{}", &path, &port);

        let json_events = scenario_parser::json_to_event_vector(&path)?;

        net_client::run(&port, &json_events).unwrap();
    }
    Ok(())
}
