#![feature(globs)]
#![feature(phase)]
#[phase(plugin)]

extern crate regex_macros;
extern crate regex;
extern crate time;
extern crate ncurses;

use std::io::File;
use std::io::fs::PathExtensions;
use std::collections::hash_map::{HashMap, Occupied, Vacant};
use time::{Tm, strptime, strftime, now};
use ncurses::*;

const QUIT_KEY: i32 = 'q' as i32;
const HOST_KEY: i32 = 'h' as i32;
const PATH_KEY: i32 = 'p' as i32;
const REFERER_KEY: i32 = 'r' as i32;

#[deriving(Clone)]
struct Hit {
    host: String,
    time: Tm,
    status: uint,
    path: String,
    referer: String,
    agent: String,
}

#[deriving(Clone)]
struct Visit {
    host: String,
    hit_count: uint,
    first_hit_time: Tm,
    last_hit_time: Tm,
    last_path: String,
    referer: String,
    agent: String,
}

type VisitCounter = HashMap<String, Box<Visit>>;

enum ProgramMode {
    Host,
    URLPath,
    Referer,
}

fn parse_line(line: &str) -> Option<Hit> {
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

fn count_hit(visits: &mut VisitCounter, hit: &Hit, key: &String) {
    let _ = match visits.entry(key.clone()) {
        Vacant(e) => {
            let visit = box Visit {
                host: hit.host.clone(),
                hit_count: 1,
                first_hit_time: hit.time,
                last_hit_time: hit.time,
                last_path: hit.path.clone(),
                referer: hit.referer.clone(),
                agent: hit.agent.clone(),
            };
            e.set(visit);
        }
        Occupied(e) => {
            let visit: &mut Box<Visit> = e.into_mut();
            visit.hit_count += 1;
            visit.last_hit_time = hit.time;
            visit.last_path = hit.path.clone();
            return;
        },
    };
}

fn purge_visits(visits: &mut VisitCounter, last_seen_time: Tm) {
    let mut toremove = Vec::new();
    let last_seen_ts = last_seen_time.to_timespec();
    for (key, value) in visits.iter() {
        if last_seen_ts.sec - value.last_hit_time.to_timespec().sec > 5 * 60 {
            toremove.push(key.clone());
        }
    }
    for key in toremove.iter() {
        visits.remove(key);
    }
}

fn mainloop(filepath: &Path, maxlines: uint) -> i32 {
    let mut timer = ::std::io::Timer::new().unwrap();
    let mut last_size: i64 = 0;
    let mut last_seen_time: Tm = time::now();
    let mut visits: VisitCounter = HashMap::new();
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
        for line in contents.split('\n') {
            let hit = match parse_line(line) {
                Some(hit) => hit,
                None => continue
            };
            count_hit(&mut visits, &hit, &hit.host);
            last_seen_time = hit.time;
        }
        purge_visits(&mut visits, last_seen_time);
        let mut sorted_visits: Vec<&Box<Visit>> = visits.values().collect();
        sorted_visits.sort_by(
            |a, b| match (&a.hit_count).cmp(&b.hit_count).reverse() {
                Equal => a.last_hit_time.to_timespec().cmp(&b.last_hit_time.to_timespec()).reverse(),
                x => x,
            }
        );
        erase();
        for (index, visit) in sorted_visits.iter().take(maxlines).enumerate() {
            let time_fmt = strftime("%Y-%m-%d %H:%M:%S", &visit.last_hit_time).unwrap();
            let visit_fmt = format!(
                "{:>4} | {:<15} | {} | {} | {}",
                visit.hit_count, visit.host, time_fmt, visit.last_path, visit.referer
            );
            mvprintw(index as i32, 0, visit_fmt.as_slice());
        }
        let mode_str = match mode {
            Host => "Host",
            URLPath => "Path",
            Referer => "Referer",
        };
        let msg = format!(
            "{} active visits. Last read: {} bytes. {} mode. Hit 'q' to quit, 'h/p/r' for the different modes",
            visits.len(), read_size, mode_str
        );
        mvprintw((maxlines+1) as i32, 0, msg.as_slice());
        refresh();
        timer.sleep(::std::time::Duration::milliseconds(1000));
        let input = getch();
        mode = match input {
            QUIT_KEY => return input,
            PATH_KEY => URLPath,
            HOST_KEY => Host,
            REFERER_KEY => Referer,
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

