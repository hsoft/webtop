#![feature(globs)]

extern crate ncurses;
extern crate regex;
extern crate time;

use std::io::File;
use std::io::fs::PathExtensions;
use time::{Tm, strptime, strftime, now};
use regex::Regex;
use ncurses::*;

struct Hit<'r> {
    host: &'r str,
    time: Tm,
    status: uint,
    path: &'r str,
    referer: &'r str,
    agent: &'r str,
}

fn parse_line(line: &str) -> Option<Box<Hit>> {
    let re = Regex::new(r#"(\d+\.\d+\.\d+\.\d+) - - \[(.+) \+\d{4}\] "\w+ ([^ ]+) [^ "]+" (\d+) \d+ "([^"]*)" "([^"]*)""#)
        .ok().expect("");
    let cap = match re.captures(line) {
        Some(cap) => cap,
        None => return None
    };
    Some(box Hit {
        host: cap.at(1),
        time: match strptime(cap.at(2), "%d/%b/%Y:%H:%M:%S") {
            Ok(tm) => tm,
            Err(_) => now()
        },
        status: match from_str(cap.at(4)) {
            Some(i) => i,
            None => 999
        },
        path: cap.at(3),
        referer: cap.at(5),
        agent: cap.at(6),
    })
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
    let mut timer = ::std::io::Timer::new().unwrap();
    let mut input : i32 = -1;
    initscr();
    raw();
    keypad(stdscr, true);
    nodelay(stdscr, true);
    noecho();

    let scry = getmaxy(stdscr) as uint;
    let maxlines = scry - 2;

    while input == -1 {
        let mut fp = File::open(&filepath).ok().expect("");
        let _ = fp.seek(-20000, ::std::io::SeekEnd);
        let raw_contents = fp.read_to_end().unwrap();
        let contents = ::std::str::from_utf8(raw_contents.as_slice()).unwrap();
        let lines = contents.split('\n').rev();
        for (index, line) in lines.enumerate() {
            let hit_box = match parse_line(line) {
                Some(hit_box) => hit_box,
                None => continue
            };
            let time_fmt = strftime("%Y-%m-%d %H:%M:%S", &hit_box.time);
            let hit_fmt = format!(
                "{} | {} | {} | {} | {} | {}",
                hit_box.host, time_fmt, hit_box.status, hit_box.path, hit_box.referer, hit_box.agent
            );
            mvprintw(index as i32, 0, hit_fmt.as_slice());
            if index == maxlines-1 {
                break;
            }
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
    endwin();
    println!("test {}", input);
}

