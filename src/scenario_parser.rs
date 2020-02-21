use std::fs::File;
use std::io::{self, BufRead};
use chrono::{NaiveDateTime, Duration};

pub fn json_to_event_vector(path: String) -> std::io::Result<Vec<(String, Duration)>> {
    let mut vector = read_file(path)?;

    let mut old_time: Option<NaiveDateTime> = None;

    let vector = vector.iter_mut().map(|(msg, delay)| {
        let parsed_time = get_time(msg).unwrap();

        if let Some(_i) = old_time {
            *delay = parsed_time.signed_duration_since(old_time.unwrap());
        }

        old_time = Some(parsed_time);

        (msg.clone(), *delay)
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_does_not_exist() {
        let res = read_file(String::from("does_not_exist.json"));

        assert!(res.is_err());
    }
}
