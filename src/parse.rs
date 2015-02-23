use std::str::FromStr;
use time::{strptime, now};
use types::Hit;

pub fn parse_line(line: &str) -> Option<Hit> {
    let re = regex!(r#"(\d+\.\d+\.\d+\.\d+) - - \[(.+) \+\d{4}\] "\w+ ([^ ]+) [^ "]+" (\d+) \d+ "([^"]*)" "([^"]*)""#);
    let cap = match re.captures(line) {
        Some(cap) => cap,
        None => return None
    };
    Some(Hit {
        host: cap.at(1).unwrap().to_string(),
        time: match strptime(cap.at(2).unwrap(), "%d/%b/%Y:%H:%M:%S") {
            Ok(tm) => tm,
            Err(_) => now()
        },
        status: match FromStr::from_str(cap.at(4).unwrap()) {
            Ok(i) => i,
            Err(_) => 999
        },
        path: cap.at(3).unwrap().to_string(),
        referer: cap.at(5).unwrap().to_string(),
        agent: cap.at(6).unwrap().to_string(),
    })
}

