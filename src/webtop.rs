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

const QUIT_KEY: i32 = 'q' as i32;
const HOST_KEY: i32 = 'h' as i32;
const PATH_KEY: i32 = 'p' as i32;

#[deriving(Clone)]
struct Hit {
    host: String,
    time: Tm,
    status: uint,
    path: String,
    referer: String,
    agent: String,
}

type HitCounter = HashMap<String, Vec<Box<Hit>>>;

enum ProgramMode {
    Host,
    URLPath,
}

fn cmp_time(a: &Tm, b: &Tm) -> Ordering {
    let va = vec![a.tm_year, a.tm_yday, a.tm_hour, a.tm_min, a.tm_sec, a.tm_nsec];
    let vb = vec![b.tm_year, b.tm_yday, b.tm_hour, b.tm_min, b.tm_sec, b.tm_nsec];
    for (ia, ib) in va.iter().zip(vb.iter()) {
        match ia.cmp(ib) {
            Equal => continue,
            Less => return Less,
            Greater => return Greater,
        }
    }
    Equal
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

fn count_hit(hit_counter: &mut HitCounter, hit: &Hit, key: &String) {
    let _ = match hit_counter.entry(key.clone()) {
        Vacant(_) => {}
        Occupied(e) => {
            let mut counter: &mut Vec<Box<Hit>> = e.into_mut();
            counter.push(box hit.clone());
            return;
        },
    };
    let counter = vec![box hit.clone()];
    hit_counter.insert(key.clone(), counter);
}

fn mainloop(filepath: &Path, maxlines: uint) -> i32 {
    let mut timer = ::std::io::Timer::new().unwrap();
    let mut last_size: i64 = 0;
    let mut host_counters: HitCounter = HashMap::new();
    let mut path_counters: HitCounter = HashMap::new();
    let mut mode = Host;
    loop {
        let fsize = filepath.stat().ok().expect("can't stat").size as i64;
        if fsize < last_size {
            let msg = "Something weird is happening with the target file, skipping this round.";
            mvprintw((maxlines+1) as i32, 0, msg.as_slice());
            continue;
        }
        let read_size: i64 = if last_size > 0 { fsize - last_size } else { 90000 };
        last_size = fsize;
        let mut fp = match File::open(filepath) {
            Ok(fp) => fp,
            Err(e) => {
                let msg = format!(
                    "Had troube reading {}! Error: {}",
                    filepath.display(), e,
                );
                mvprintw((maxlines+1) as i32, 0, msg.as_slice());
                continue;
            },
        };
        let _ = fp.seek(-read_size, ::std::io::SeekEnd);
        let raw_contents = fp.read_to_end().unwrap();
        let contents = ::std::str::from_utf8(raw_contents.as_slice()).unwrap();
        for line in contents.split('\n').rev() {
            let hit = match parse_line(line) {
                Some(hit) => hit,
                None => continue
            };
            count_hit(&mut host_counters, &hit, &hit.host);
            count_hit(&mut path_counters, &hit, &hit.path);
        }
        let counters = match mode {
            Host => &host_counters,
            URLPath => &path_counters,
        };
        let mut sorted_counters: Vec<&Vec<Box<Hit>>> = counters.values().collect();
        sorted_counters.sort_by(
            |a, b| match (&b.len()).cmp(&a.len()) {
                Equal => cmp_time(&b[b.len()-1].time, &a[a.len()-1].time),
                x => x,
            }
        );
        erase();
        for (index, counter) in sorted_counters.iter().take(maxlines).enumerate() {
            let hit = counter[counter.len()-1].clone();
            let time_fmt = strftime("%Y-%m-%d %H:%M:%S", &hit.time).unwrap();
            let hit_fmt = format!(
                "{:>4} | {:<15} | {} | {} | {} | {}",
                counter.len(), hit.host, time_fmt, hit.status, hit.path, hit.referer
            );
            mvprintw(index as i32, 0, hit_fmt.as_slice());
        }
        let mode_str = match mode {
            Host => "Host",
            URLPath => "Path",
        };
        let msg = format!(
            "Last read: {} bytes. {} mode. Hit 'q' to quit, 'h' for host mode, 'p' for path mode.",
            read_size, mode_str
        );
        mvprintw((maxlines+1) as i32, 0, msg.as_slice());
        refresh();
        timer.sleep(::std::time::Duration::milliseconds(1000));
        let input = getch();
        mode = match input {
            QUIT_KEY => return input,
            PATH_KEY => URLPath,
            HOST_KEY => Host,
            _ => mode,
        }
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

    let last_input = mainloop(&filepath, maxlines);

    endwin();
    println!("Program ended with last input {}", last_input);
}

