use std::str::FromStr;
use time::{strptime, now};
use regex::Regex;
use hit::Hit;

pub struct Parser {
    re_main: Regex,
    re_path: Regex,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            re_main: Regex::new(
                r#"(\d+\.\d+\.\d+\.\d+) - - \[(.+) \+\d{4}\] "\w+ ([^ ]+) [^ "]+" (\d+) (\d+) "([^"]*)" "([^"]*)""#
            ).unwrap(),
            // Clean the part after the "?"
            re_path: Regex::new(
                r#"([^\?]+).*"#
            ).unwrap(),
        }
    }

    pub fn parse_line(&self, line: &str) -> Option<Hit> {
        let cap = match self.re_main.captures(line) {
            Some(cap) => cap,
            None => return None
        };
        let path = cap.at(3).unwrap();
        let cleaned_path = self.re_path.captures(path).unwrap().at(1).unwrap();
        let referer = cap.at(6).unwrap();
        let cleaned_referer = self.re_path.captures(referer).unwrap().at(1).unwrap();
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
            bytes: match FromStr::from_str(cap.at(5).unwrap()) {
                Ok(i) => i,
                Err(_) => 0
            },
            path: cleaned_path.to_string(),
            referer: cleaned_referer.to_string(),
            agent: cap.at(7).unwrap().to_string(),
        })
    }
}

