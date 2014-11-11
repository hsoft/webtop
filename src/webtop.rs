#![feature(globs)]

extern crate ncurses;

use std::io::{File, BufferedReader};
use ncurses::*;

fn main()
{
    let filename = "test.txt";
    let maxlines = 3;
    let mut timer = ::std::io::Timer::new().unwrap();
    let mut input : i32 = -1;
    initscr();
    raw();
    keypad(stdscr, true);
    nodelay(stdscr, true);
    noecho();

    while input == -1 {
        let fp = File::open(&Path::new(filename)).ok().expect("");
        let mut reader = BufferedReader::new(fp);
        for (index, line) in reader.lines().enumerate() {
            mvprintw(index as i32, 0, line.ok().expect("").as_slice());
            if index == maxlines-1 {
                break;
            }
        }
        let msg = format!(
            "This program reads the first {} lines of {} and updates automatically",
            maxlines, filename
        );
        mvprintw((maxlines+1) as i32, 0, msg.as_slice());
        refresh();
        timer.sleep(::std::time::Duration::milliseconds(1000));
        input = getch();
    }
    endwin();
    println!("test {}", input);
}

