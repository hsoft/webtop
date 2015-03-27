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
use time::{Tm, strftime, now, precise_time_s};
use ncurses::{
    mvprintw, refresh, initscr, getch, raw, keypad, nodelay, noecho, stdscr, getmaxy, endwin
};
use types::Visit;
use parse::parse_line;
use screen::Screen;

mod types;
mod parse;
mod screen;

const QUIT_KEY: i32 = 'q' as i32;
const HOST_KEY: i32 = 'h' as i32;
const PATH_KEY: i32 = 'p' as i32;
const REFERER_KEY: i32 = 'r' as i32;
const UP_KEY: i32 = 259;
const DOWN_KEY: i32 = 258;

type VisitID = usize;
type VisitHolder = HashMap<VisitID, Box<Visit>>;
type HostVisitMap = HashMap<String, VisitID>;
type PathVisitMap = HashMap<String, Box<HashSet<VisitID>>>;

#[derive(PartialEq)]
#[derive(Copy)]
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

fn output_host_mode(visits: &VisitHolder, screen: &mut Screen) {
    let mut sorted_visits: Vec<&Box<Visit>> = visits.values().collect();
    sorted_visits.sort_by(
        |a, b| match (&a.hit_count).cmp(&b.hit_count).reverse() {
            Ordering::Equal => a.last_hit_time.to_timespec().cmp(&b.last_hit_time.to_timespec()).reverse(),
            x => x,
        }
    );
    screen.erase();
    for (index, visit) in sorted_visits.iter().take(screen.maxlines).enumerate() {
        let time_fmt = strftime("%Y-%m-%d %H:%M:%S", &visit.last_hit_time).unwrap();
        let visit_fmt = format!(
            "{:>4} | {:<15} | {} | {} | {}",
            visit.hit_count, visit.host, time_fmt, visit.last_path, visit.referer
        );
        screen.printline(index, &visit_fmt[..]);
    }
    screen.adjust_selection();
}

fn output_path_mode(path_visit_map: &PathVisitMap, screen: &mut Screen) {
    let mut sorted_path_chunks: Vec<(&str, usize)> = path_visit_map.iter().map(
        |(key, value)| (&key[..], value.len())
    ).collect();
    sorted_path_chunks.sort_by(
        |a, b| a.1.cmp(&b.1).reverse()
    );
    screen.erase();
    for (index, pair) in sorted_path_chunks.iter().take(screen.maxlines).enumerate() {
        let path = pair.0;
        let visit_count = pair.1;
        let path_fmt = format!(
            "{:>4} | {}",
            visit_count, path,
        );
        screen.printline(index, &path_fmt[..]);
    }
    screen.adjust_selection();
}

struct WholeThing {
    screen: Screen,
    last_size: i64,
    last_seen_time: Tm,
    visits: VisitHolder,
    visit_counter: usize,
    host_visit_map: HostVisitMap,
    path_visit_map: PathVisitMap,
    mode: ProgramMode,
}

fn refresh_visit_stats(filepath: &Path, wt: &mut WholeThing) {
    let fsize = filepath.stat().ok().expect("can't stat").size as i64;
    if fsize < wt.last_size {
        let msg = "Something weird is happening with the target file, skipping this round.";
        mvprintw((wt.screen.maxlines+1) as i32, 0, &msg[..]);
        return;
    }
    let read_size: i64 = if wt.last_size > 0 { fsize - wt.last_size } else { 90000 };
    wt.last_size = fsize;
    let mut fp = match File::open(filepath) {
        Ok(fp) => fp,
        Err(e) => {
            let msg = format!(
                "Had troube reading {}! Error: {}",
                filepath.display(), e,
            );
            mvprintw((wt.screen.maxlines+1) as i32, 0, &msg[..]);
            return;
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
        let visitid: VisitID = match wt.host_visit_map.entry(key.clone()) {
            Entry::Occupied(e) => {
                *e.get()
            }
            Entry::Vacant(e) => {
                wt.visit_counter += 1;
                let visitid = wt.visit_counter;
                let visit = Box::new(Visit {
                    host: hit.host.clone(),
                    hit_count: 0,
                    first_hit_time: hit.time,
                    last_hit_time: hit.time,
                    last_path: hit.path.clone(),
                    referer: hit.referer.clone(),
                    agent: hit.agent.clone(),
                });
                wt.visits.insert(visitid, visit);
                e.insert(visitid);
                visitid
            }
        };
        let visit: &mut Box<Visit> = wt.visits.get_mut(&visitid).unwrap();
        visit.hit_count += 1;
        visit.last_hit_time = hit.time;
        visit.last_path = hit.path.clone();
        wt.last_seen_time = hit.time;
        let key = &hit.path;
        match wt.path_visit_map.entry(key.clone()) {
            Entry::Occupied(e) => {
                let visits: &mut Box<HashSet<usize>> = e.into_mut();
                visits.insert(visitid);
            }
            Entry::Vacant(e) => {
                let mut visits = Box::new(HashSet::new());
                visits.insert(visitid);
                e.insert(visits);
            }
        };
    }
    purge_visits(&mut wt.visits, &mut wt.host_visit_map, wt.last_seen_time);
    match wt.mode {
        ProgramMode::URLPath => output_path_mode(&wt.path_visit_map, &mut wt.screen),
        _ => output_host_mode(&wt.visits, &mut wt.screen),
    };
    let mode_str = match wt.mode {
        ProgramMode::Host => "Host",
        ProgramMode::URLPath => "Path",
        ProgramMode::Referer => "Referer",
    };
    let msg = format!(
        "{} active visits. Last read: {} bytes. {} mode. Hit 'q' to quit, 'h/p/r' for the different modes",
        wt.visits.len(), read_size, mode_str
    );
    mvprintw((wt.screen.maxlines+1) as i32, 0, &msg[..]);
    refresh();
}

fn mainloop(filepath: &Path, maxlines: usize) -> i32 {
    let mut timer = ::std::old_io::Timer::new().unwrap();
    let mut last_refresh_time: f64 = 0.0;
    let mut wt = WholeThing {
        screen: Screen::new(maxlines),
        last_size: 0,
        last_seen_time:  time::now(),
        visits: HashMap::new(),
        visit_counter: 0,
        host_visit_map: HashMap::new(),
        path_visit_map: HashMap::new(),
        mode: ProgramMode::Host,
    };
    loop {
        if precise_time_s() - last_refresh_time > 1.0 {
            refresh_visit_stats(filepath, &mut wt);
            last_refresh_time = precise_time_s();
        }
        timer.sleep(::std::time::Duration::milliseconds(50));
        let input = getch();
        if input >= 0 {
            wt.mode = match input {
                QUIT_KEY => return input,
                PATH_KEY => ProgramMode::URLPath,
                HOST_KEY => ProgramMode::Host,
                REFERER_KEY => ProgramMode::Referer,
                UP_KEY => { wt.screen.up(); wt.mode },
                DOWN_KEY => { wt.screen.down(); wt.mode },
                _ => wt.mode,
            };
            last_refresh_time = 0.0;
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

