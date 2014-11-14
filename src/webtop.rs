#![feature(globs)]

extern crate ncurses;
extern crate regex;
extern crate time;

use std::io::File;
use std::io::fs::PathExtensions;
use std::collections::hash_map::{HashMap, Occupied, Vacant};
use time::{Tm, strptime, strftime, now};
use regex::Regex;
use ncurses::*;

#[deriving(Clone)]
struct Hit {
    host: String,
    time: Tm,
    status: uint,
    path: String,
    referer: String,
    agent: String,
}

struct HitCounter {
    host: String,
    last_hit: Box<Hit>,
    count: uint,
}

fn parse_line(line: &str) -> Option<Hit> {
    let re = Regex::new(r#"(\d+\.\d+\.\d+\.\d+) - - \[(.+) \+\d{4}\] "\w+ ([^ ]+) [^ "]+" (\d+) \d+ "([^"]*)" "([^"]*)""#)
        .ok().expect("");
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

fn mainloop(filepath: &Path, maxlines: uint) {
    let mut timer = ::std::io::Timer::new().unwrap();
    let mut input : i32 = -1;
    while input == -1 {
        let mut fp = File::open(filepath).ok().expect("");
        let _ = fp.seek(-20000, ::std::io::SeekEnd);
        let raw_contents = fp.read_to_end().unwrap();
        let contents = ::std::str::from_utf8(raw_contents.as_slice()).unwrap();
        let mut counters: HashMap<String, HitCounter> = HashMap::new();
        for line in contents.split('\n').rev() {
            let hit_box = match parse_line(line) {
                Some(hit) => box hit,
                None => continue
            };
            let _ = match counters.entry(hit_box.clone().host.clone()) {
                Vacant(_) => {}
                Occupied(e) => {
                    let mut counter: &mut HitCounter = e.into_mut();
                    counter.count += 1;
                    continue;
                },
            };
            let counter = HitCounter {
                host: hit_box.host.clone(),
                last_hit: hit_box.clone(),
                count: 1,
            };
            counters.insert(hit_box.host, counter);
        }
        let mut sorted_counters: Vec<&HitCounter> = counters.values().collect();
        sorted_counters.sort_by(|a, b| (&b.count).cmp(&a.count));
        for (index, counter) in sorted_counters.iter().take(maxlines).enumerate() {
            let hit = counter.last_hit.clone();
            let time_fmt = strftime("%Y-%m-%d %H:%M:%S", &hit.time).unwrap();
            let hit_fmt = format!(
                "{} | {} | {} | {} | {} | {}",
                counter.count, hit.host, time_fmt, hit.status, hit.path, hit.referer
            );
            mvprintw(index as i32, 0, hit_fmt.as_slice());
        }
        let msg = format!(
            "This program reads the last {} lines of {} and updates automatically",
            maxlines, filepath.display()
        );
        mvprintw((maxlines+1) as i32, 0, msg.as_slice());
        refresh();
        timer.sleep(::std::time::Duration::milliseconds(1000));
        input = getch();
    }
}
fn main()
{
    let args = ::std::os::args();
    if args.len() < 2 {
        println!("You need to specify a file to watch.");
        return;
    }
    let filepath = Path::new(args[1].as_slice());
    if !filepath.exists() {
        println!("{} doesn't exist! aborting.", filepath.display());
        return;
    }
    initscr();
    raw();
    keypad(stdscr, true);
    nodelay(stdscr, true);
    noecho();

    let scry = getmaxy(stdscr) as uint;
    let maxlines = scry - 2;

    mainloop(&filepath, maxlines);

    endwin();
}

