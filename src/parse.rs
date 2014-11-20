extern crate regex_macros;
extern crate regex;
extern crate time;

use time::{strptime, now};
use types::Hit;

pub fn parse_line(line: &str) -> Option<Hit> {
    let re = regex!(r#"(\d+\.\d+\.\d+\.\d+) - - \[(.+) \+\d{4}\] "\w+ ([^ ]+) [^ "]+" (\d+) \d+ "([^"]*)" "([^"]*)""#);
    let cap = match re.captures(line) {
        Some(cap) => cap,
        None => return None
    };
    Some(Hit {
        host: cap.at(1).to_string(),
        time: match strptime(cap.at(2), "%d/%b/%Y:%H:%M:%S") {
            Ok(tm) => tm,
            Err(_) => now()
        },
        status: match from_str(cap.at(4)) {
            Some(i) => i,
            None => 999
        },
        path: cap.at(3).to_string(),
        referer: cap.at(5).to_string(),
        agent: cap.at(6).to_string(),
    })
}

