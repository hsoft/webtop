#![feature(box_syntax)]
#![feature(core)]
#![feature(env)]
#![feature(std_misc)]
#![feature(old_io)]
#![feature(old_path)]
#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;
extern crate time;
extern crate ncurses;

use std::old_io::File;
use std::old_io::fs::PathExtensions;
use std::cmp::Ordering;
use std::collections::hash_map::{HashMap, Entry};
use std::collections::hash_set::HashSet;
use time::{Tm, strftime, now};
use ncurses::{
    mvprintw, refresh, erase, initscr, getch, raw, keypad, nodelay, noecho, stdscr, getmaxy, endwin
};
use types::Visit;
use parse::parse_line;

mod types;
mod parse;

const QUIT_KEY: i32 = 'q' as i32;
const HOST_KEY: i32 = 'h' as i32;
const PATH_KEY: i32 = 'p' as i32;
const REFERER_KEY: i32 = 'r' as i32;

type VisitID = usize;
type VisitHolder = HashMap<VisitID, Box<Visit>>;
type HostVisitMap = HashMap<String, VisitID>;
type PathVisitMap = HashMap<String, Box<HashSet<VisitID>>>;

enum ProgramMode {
    Host,
    URLPath,
    Referer,
}

fn purge_visits(visits: &mut VisitHolder, host_visit_map: &mut HostVisitMap, last_seen_time: Tm) {
    let mut toremove: Vec<usize> = Vec::new();
    let last_seen_ts = last_seen_time.to_timespec();
    for (visitid, visit) in visits.iter() {
        if last_seen_ts.sec - visit.last_hit_time.to_timespec().sec > 5 * 60 {
            toremove.push(*visitid);
            host_visit_map.remove(&visit.host);
        }
    }
    for visitid in toremove.iter() {
        visits.remove(visitid);
    }
}

fn output_host_mode(visits: &VisitHolder, maxlines: usize) {
    let mut sorted_visits: Vec<&Box<Visit>> = visits.values().collect();
    sorted_visits.sort_by(
        |a, b| match (&a.hit_count).cmp(&b.hit_count).reverse() {
            Ordering::Equal => a.last_hit_time.to_timespec().cmp(&b.last_hit_time.to_timespec()).reverse(),
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
        mvprintw(index as i32, 0, &visit_fmt[..]);
    }
}

fn output_path_mode(path_visit_map: &PathVisitMap, maxlines: usize) {
    let mut sorted_path_chunks: Vec<(&str, usize)> = path_visit_map.iter().map(
        |(key, value)| (&key[..], value.len())
    ).collect();
    sorted_path_chunks.sort_by(
        |a, b| a.1.cmp(&b.1).reverse()
    );
    erase();
    for (index, pair) in sorted_path_chunks.iter().take(maxlines).enumerate() {
        let path = pair.0;
        let visit_count = pair.1;
        let path_fmt = format!(
            "{:>4} | {}",
            visit_count, path,
        );
        mvprintw(index as i32, 0, &path_fmt[..]);
    }
}

fn mainloop(filepath: &Path, maxlines: usize) -> i32 {
    let mut timer = ::std::old_io::Timer::new().unwrap();
    let mut last_size: i64 = 0;
    let mut last_seen_time: Tm = time::now();
    let mut visits: VisitHolder = HashMap::new();
    let mut visit_counter:usize = 0;
    let mut host_visit_map: HostVisitMap = HashMap::new();
    let mut path_visit_map: PathVisitMap = HashMap::new();
    let mut mode = ProgramMode::Host;
    loop {
        let fsize = filepath.stat().ok().expect("can't stat").size as i64;
        if fsize < last_size {
            let msg = "Something weird is happening with the target file, skipping this round.";
            mvprintw((maxlines+1) as i32, 0, &msg[..]);
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
                mvprintw((maxlines+1) as i32, 0, &msg[..]);
                continue;
            },
        };
        let _ = fp.seek(-read_size, ::std::old_io::SeekEnd);
        let raw_contents = fp.read_to_end().unwrap();
        let contents = ::std::str::from_utf8(&raw_contents[..]).unwrap();
        for line in contents.split('\n') {
            let hit = match parse_line(line) {
                Some(hit) => hit,
                None => continue
            };
            let key = &hit.host;
            let visitid: VisitID = match host_visit_map.entry(key.clone()) {
                Entry::Occupied(e) => {
                    *e.get()
                }
                Entry::Vacant(e) => {
                    visit_counter += 1;
                    let visitid = visit_counter;
                    let visit = box Visit {
                        host: hit.host.clone(),
                        hit_count: 0,
                        first_hit_time: hit.time,
                        last_hit_time: hit.time,
                        last_path: hit.path.clone(),
                        referer: hit.referer.clone(),
                        agent: hit.agent.clone(),
                    };
                    visits.insert(visitid, visit);
                    e.insert(visitid);
                    visitid
                }
            };
            let visit: &mut Box<Visit> = visits.get_mut(&visitid).unwrap();
            visit.hit_count += 1;
            visit.last_hit_time = hit.time;
            visit.last_path = hit.path.clone();
            last_seen_time = hit.time;
            let key = &hit.path;
            match path_visit_map.entry(key.clone()) {
                Entry::Occupied(e) => {
                    let visits: &mut Box<HashSet<usize>> = e.into_mut();
                    visits.insert(visitid);
                }
                Entry::Vacant(e) => {
                    let mut visits = box HashSet::new();
                    visits.insert(visitid);
                    e.insert(visits);
                }
            };
        }
        purge_visits(&mut visits, &mut host_visit_map, last_seen_time);
        match mode {
            ProgramMode::URLPath => output_path_mode(&path_visit_map, maxlines),
            _ => output_host_mode(&visits, maxlines),
        };
        let mode_str = match mode {
            ProgramMode::Host => "Host",
            ProgramMode::URLPath => "Path",
            ProgramMode::Referer => "Referer",
        };
        let msg = format!(
            "{} active visits. Last read: {} bytes. {} mode. Hit 'q' to quit, 'h/p/r' for the different modes",
            visits.len(), read_size, mode_str
        );
        mvprintw((maxlines+1) as i32, 0, &msg[..]);
        refresh();
        timer.sleep(::std::time::Duration::milliseconds(1000));
        let input = getch();
        mode = match input {
            QUIT_KEY => return input,
            PATH_KEY => ProgramMode::URLPath,
            HOST_KEY => ProgramMode::Host,
            REFERER_KEY => ProgramMode::Referer,
            _ => mode,
        }
    }
}

fn main()
{
    let mut args = ::std::env::args();
    if args.len() < 2 {
        println!("You need to specify a file to watch.");
        return;
    }
    let _ = args.next();
    let filepath = Path::new(&args.next().unwrap()[..]);
    if !filepath.exists() {
        println!("{} doesn't exist! aborting.", filepath.display());
        return;
    }
    initscr();
    raw();
    keypad(stdscr, true);
    nodelay(stdscr, true);
    noecho();

    let scry = getmaxy(stdscr) as usize;
    let maxlines = scry - 2;

    let last_input = mainloop(&filepath, maxlines);

    endwin();
    println!("Program ended with last input {}", last_input);
}

