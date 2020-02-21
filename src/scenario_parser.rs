use std::fs::File;
use std::io::{self, BufRead};
use chrono::{NaiveDateTime, Duration};

pub fn json_to_event_vector(path: &String) -> std::io::Result<Vec<(String, Duration)>> {
    let mut vector = read_file(path)?;

    let mut old_time: Option<NaiveDateTime> = None;

    let vector = vector.iter_mut().map(|(msg, delay)| {
        let parsed_time = get_time(msg);

        if let Some(_i) = old_time {
            *delay = parsed_time.signed_duration_since(old_time.unwrap());
        }

        old_time = Some(parsed_time);

        (msg.clone(), *delay)
    }).collect();

    Ok(vector)
}

fn read_file(path: &String) -> std::io::Result<Vec<(String, Duration)>> {
    let file = File::open(path)?;

    let vector: Vec<(String, Duration)> = io::BufReader::new(file)
        .lines()
        .map(|x| (x.unwrap(), Duration::milliseconds(0)))
        .collect();
    Ok(vector)
}

/// Will panic!
fn get_time(json_msg: &String) -> NaiveDateTime {
    let json_msg = json::parse(json_msg.as_str()).unwrap();

    let event_time = json_msg["msg"]["EventTime"].as_str().unwrap();

    NaiveDateTime::parse_from_str(event_time, "%Y-%b-%d %H:%M:%S.%f").unwrap()
}


#[cfg(test)]
mod read_file {
    use super::*;
    use std::io::Write;

    #[test]
    fn file_does_not_exist() {
        let path = String::from("does_not_exist.json");
        let res = read_file(&path);

        assert!(res.is_err());
    }

    #[test]
    fn read_json_values() {
        let path = String::from("read_json_values.json");
        let mut file = File::create(&path).unwrap();
        file.write_all("{\"int\":1,\"string\":\"hello\"}\n{\"int\":2,\"string\":\"world\"}".as_bytes()).unwrap();

        let res = read_file(&path);

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res[0].0, "{\"int\":1,\"string\":\"hello\"}");
        assert_eq!(res[0].1, Duration::milliseconds(0));

        assert_eq!(res[1].0, "{\"int\":2,\"string\":\"world\"}");
        assert_eq!(res[1].1, Duration::milliseconds(0));

        let _ = std::fs::remove_file(&path);
    }
}

#[cfg(test)]
mod get_time {
    use super::*;

    #[test]
    #[should_panic]
    fn not_json() {
        let event = String::from(r#"{"int":1"#);

        let _ = get_time(&event);
    }

    #[test]
    #[should_panic]
    fn no_event_time() {
        let event = String::from(r#"{"msg":{"Event":"empty"}}"#);

        let _ = get_time(&event);
    }

    #[test]
    #[should_panic]
    fn event_time_not_a_string() {
        let event = String::from(r#"{"msg":{"EventTime":1}}"#);

        let _ = get_time(&event);
    }

    #[test]
    #[should_panic]
    fn event_time_wrong_format() {
        let event = String::from(r#"{"msg":{"EventTime":"11-10-1988 8:30:12.22"}}"#);

        let _ = get_time(&event);
    }

    #[test]
    fn read_event_time() {
        let event = String::from(r#"{"msg":{"EventTime":"1988-Oct-11 8:30:12.22"}}"#);

        let res = get_time(&event);

        assert_eq!(res.to_string(), String::from("1988-10-11 08:30:12.000000022"));
    }
}

#[cfg(test)]
mod json_to_event_vector {
    use super::*;
    use std::io::Write;

    #[test]
    fn convert_time_to_duration() {
        let path = String::from("convert_time_to_duration.json");
        let mut file = File::create(&path).unwrap();
        file.write_all("{\"msg\":{\"EventTime\":\"1988-Oct-11 8:30:12.22\"}}\n{\"msg\":{\"EventTime\":\"1988-Oct-11 8:30:13.22\"}}".as_bytes()).unwrap();

        let res = json_to_event_vector(&path);

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res[1].1, Duration::seconds(1));

        let _ = std::fs::remove_file(&path);
    }
}
