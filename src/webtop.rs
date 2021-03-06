extern crate regex;
extern crate time;
extern crate ncurses;
extern crate libc;
extern crate number_prefix;

use std::io::prelude::*;
use std::io;
use std::fs;
use std::ffi::CString;
use std::path::Path;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use time::precise_time_s;
use ncurses::{
    initscr, getch, raw, keypad, nodelay, noecho, stdscr, endwin, newterm, set_term, curs_set,
    CURSOR_VISIBILITY, setlocale, LcCategory
};
use ncurses::ll;
use visits::*;
use parse::Parser;
use screen::Screen;

mod hit;
mod visits;
mod parse;
mod screen;
mod help_panel;
mod visit_detail_panel;

const HELP_KEY: i32 = '?' as i32;
const QUIT_KEY: i32 = 'q' as i32;
const HOST_KEY: i32 = 'h' as i32;
const PATH_KEY: i32 = 'p' as i32;
const REFERER_KEY: i32 = 'r' as i32;
const DETAIL_KEY: i32 = 'd' as i32;
const UP_KEY: i32 = 259;
const DOWN_KEY: i32 = 258;

#[derive(PartialEq, Copy, Clone)]
enum ProgramMode {
    Host,
    URLPath,
    Referer,
}

#[derive(Copy, Clone)]
enum PathOrStdin<'a> {
    Path(&'a Path),
    Stdin(&'a Receiver<String>),
}

struct WholeThing<'a> {
    inpath: PathOrStdin<'a>,
    parser: Parser,
    screen: Screen,
    last_size: i64,
    visit_stats: VisitStats,
    selected_visitid: VisitID,
    mode: ProgramMode,
}

impl<'a> WholeThing<'a> {
    fn new(inpath: PathOrStdin) -> WholeThing {
        WholeThing {
            inpath: inpath,
            parser: Parser::new(),
            screen: Screen::new(),
            last_size: 0,
            visit_stats: VisitStats::new(),
            selected_visitid: 0,
            mode: ProgramMode::Host,
        }
    }

    fn refresh_visit_stats(&mut self) {
        let contents = match self.inpath {
            PathOrStdin::Path(filepath) => {
                let fsize = fs::metadata(filepath).ok().expect("can't stat").len() as i64;
                if fsize < self.last_size {
                    let msg = "Something weird is happening with the target file, skipping this round.";
                    self.screen.printstatus(&msg[..]);
                    return;
                }
                let read_size: i64 = if self.last_size > 0 { fsize - self.last_size } else { 90000 };
                self.last_size = fsize;
                let mut fp = match fs::File::open(filepath) {
                    Ok(fp) => fp,
                    Err(e) => {
                        let msg = format!(
                            "Had troube reading {}! Error: {}",
                            filepath.display(), e,
                        );
                        self.screen.printstatus(&msg[..]);
                        return;
                    },
                };
                let _ = fp.seek(io::SeekFrom::End(-read_size));
                let mut res = String::new();
                fp.read_to_string(&mut res).unwrap();
                res
            },
            PathOrStdin::Stdin(rx) => {
                let mut res = String::new();
                loop {
                    match rx.try_recv() {
                        Ok(msg) => {
                            res.push_str(&msg[..]);
                        },
                        Err(_) => { break; }
                    }
                }
                res
            },
        };
        let read_size = contents.len();
        for line in contents.split('\n') {
            let hit = match self.parser.parse_line(line) {
                Some(hit) => hit,
                None => continue
            };
            self.visit_stats.feed_hit(&hit);
        }
        self.visit_stats.purge_visits();
        match self.mode {
            ProgramMode::URLPath => self.output_path_mode(),
            ProgramMode::Referer => self.output_referer_mode(),
            ProgramMode::Host => self.output_host_mode(),
        };
        let mode_str = match self.mode {
            ProgramMode::Host => "Host",
            ProgramMode::URLPath => "Path",
            ProgramMode::Referer => "Referer",
        };
        let msg = format!(
            "{} active visits. Last read: {} bytes. {} mode. Hit '?' for help.",
            self.visit_stats.visit_count(), read_size, mode_str
        );
        self.screen.printstatus(&msg[..]);
        self.screen.refresh();
    }

    fn output_host_mode(&mut self) {
        self.screen.erase();
        for (index, visit) in self.visit_stats.iter_sorted_visits().take(self.screen.maxlines() as usize).enumerate() {
            let problem_marker = if visit.has_problems() { "!" } else { " " };
            let visit_fmt = format!(
                "{}{:>4} | {:<15} | {} | {:<6} | {} | {}",
                problem_marker, visit.hit_count, visit.host, visit.fmt_time_range(),
                visit.fmt_bytes(), visit.last_path, visit.referer
            );
            self.screen.printline(index as u32, &visit_fmt[..]);
            if (index as u32) == self.screen.selected_index {
                self.selected_visitid = visit.id;
            }
        }
        self.screen.adjust_selection();
    }

    fn output_path_mode(&mut self) {
        self.screen.erase();
        for (index, pair) in self.visit_stats.iter_sorted_path_chunks().take(self.screen.maxlines() as usize).enumerate() {
            let path = pair.0;
            let visit_count = pair.1;
            let path_fmt = format!(
                "{:>4} | {}",
                visit_count, path,
            );
            self.screen.printline(index as u32, &path_fmt[..]);
        }
        self.screen.adjust_selection();
    }

    fn output_referer_mode(&mut self) {
        self.screen.erase();
        for (index, pair) in self.visit_stats.iter_sorted_referer_chunks().take(self.screen.maxlines() as usize).enumerate() {
            let referer = pair.0;
            let visit_count = pair.1;
            let referer_fmt = format!(
                "{:>4} | {}",
                visit_count, referer,
            );
            self.screen.printline(index as u32, &referer_fmt[..]);
        }
        self.screen.adjust_selection();
    }

    fn show_selected_visit(&mut self) {
        let visit = self.visit_stats.get_visit_by_id(self.selected_visitid);
        match visit {
            Some(visit) => self.screen.show_visit_details(visit),
            None => (),
        };
    }

    fn mainloop(&mut self) -> i32 {
        let mut last_refresh_time: f64 = 0.0;
        loop {
            if precise_time_s() - last_refresh_time > 1.0 {
                self.refresh_visit_stats();
                last_refresh_time = precise_time_s();
            }
            thread::sleep_ms(50);
            let input = getch();
            if input >= 0 {
                self.mode = match input {
                    QUIT_KEY => {
                        if !self.screen.close_top_dialog() { return input} else { self.mode }
                    },
                    PATH_KEY => ProgramMode::URLPath,
                    HOST_KEY => ProgramMode::Host,
                    REFERER_KEY => ProgramMode::Referer,
                    UP_KEY => { self.screen.up(); self.mode },
                    DOWN_KEY => { self.screen.down(); self.mode },
                    HELP_KEY => { self.screen.toggle_help(); self.mode },
                    DETAIL_KEY => { self.show_selected_visit(); self.mode },
                    _ => self.mode,
                };
                last_refresh_time = 0.0;
            }
        }
    }

}

fn main()
{
    setlocale(LcCategory::all, "en_US.UTF-8");
    let mut args = ::std::env::args();
    if args.len() < 2 {
        println!("You need to specify a file to watch.");
        return;
    }
    let _ = args.next();
    let inpath = &args.next().unwrap()[..];
    let filepath = &Path::new(inpath);
    let (stdin_tx, stdin_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let (stdin_stopped_tx, stdin_stopped_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let path = match inpath {
        "-" => {
            thread::spawn(move || {
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    let mut topush = line.unwrap();
                    topush.push_str(&"\n");
                    stdin_tx.send(topush).unwrap();
                }
                stdin_stopped_tx.send(true).unwrap();
            });
            PathOrStdin::Stdin(&stdin_rx)
        },
        _ => {
            if fs::metadata(filepath).is_err() {
                println!("{} doesn't exist! aborting.", filepath.display());
                return;
            }
            PathOrStdin::Path(filepath)
        },
    };
    if unsafe { libc::isatty(libc::STDIN_FILENO) } != 1 {
        println!("STDIN is not a terminal. Trying to get in touch with a terminal now...");
        let tty_fp = unsafe { libc::fopen(
            CString::new(&"/dev/tty"[..]).unwrap().as_ptr(),
            CString::new(&"r"[..]).unwrap().as_ptr(),
        ) as ll::FILE_p};
        let stdout_fp = unsafe { libc::fdopen(
            libc::STDOUT_FILENO,
            CString::new(&"w"[..]).unwrap().as_ptr(),
        ) as ll::FILE_p};
        let term = newterm(None, stdout_fp, tty_fp);
        set_term(term);
    }
    else {
        initscr();
    }
    raw();
    keypad(stdscr, true);
    nodelay(stdscr, true);
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    let mut wt = WholeThing::new(path);
    let last_input = wt.mainloop();

    endwin();
    println!("Program ended with last input {}", last_input);
    match path {
        PathOrStdin::Stdin(_) => {
            match stdin_stopped_rx.try_recv() {
                Err(_) => {
                    println!("STDIN still active, process stalled. Press CTRL-C to end it.");
                }
                _ => {},
            }
        },
        _ => {},
    }
}

