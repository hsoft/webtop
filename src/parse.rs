use std::str::FromStr;
use time::{strptime, now};
use regex::Regex;
use visits::Hit;

pub struct Parser {
    re: Regex,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            re: Regex::new(r#"(\d+\.\d+\.\d+\.\d+) - - \[(.+) \+\d{4}\] "\w+ ([^ ]+) [^ "]+" (\d+) \d+ "([^"]*)" "([^"]*)""#).unwrap(),
        }
    }

    pub fn parse_line(&self, line: &str) -> Option<Hit> {
        let cap = match self.re.captures(line) {
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
}

