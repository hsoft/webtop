#![feature(globs)]

extern crate ncurses;

use std::io::File;
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
            mvprintw(index as i32, 0, line);
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

