#![feature(globs)]

extern crate ncurses;

use std::io::{File, BufferedReader};
use std::io::fs::PathExtensions;
use ncurses::*;

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
    let maxlines = 3;
    let mut timer = ::std::io::Timer::new().unwrap();
    let mut input : i32 = -1;
    initscr();
    raw();
    keypad(stdscr, true);
    nodelay(stdscr, true);
    noecho();

    while input == -1 {
        let fp = File::open(&filepath).ok().expect("");
        let mut reader = BufferedReader::new(fp);
        for (index, line) in reader.lines().enumerate() {
            mvprintw(index as i32, 0, line.ok().expect("").as_slice());
            if index == maxlines-1 {
                break;
            }
        }
        let msg = format!(
            "This program reads the first {} lines of {} and updates automatically",
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

