use std::cmp::{min, max};
use ncurses::{
    stdscr, getmaxy, getmaxx, mvaddstr, mvprintw, box_, erase, refresh, 
    wrefresh, newwin, mvwprintw,
    attron, attroff, A_REVERSE
};

pub struct Screen {
    pub maxlines: u32,
    pub selected_index: u32,
    maxindex: u32,
    scrx: i32,
    show_help: bool,
}

impl Screen {
    pub fn new() -> Screen {
        let scry = getmaxy(stdscr);
        let scrx = getmaxx(stdscr);
        let maxlines = (scry - 2) as u32;

        Screen {
            maxlines: maxlines,
            selected_index: 0,
            maxindex: 0,
            scrx: scrx,
            show_help: false,
        }
    }

    fn show_help_window(&self) {
        let lines = [
            "h - Host mode",
            "p - Path mode",
            "r - Referer mode",
            "↑/↓ - Selection",
            "q - Quit",
        ];
        let width = 20;
        let w = newwin(7, width, 1, self.scrx - width);
        for (index, text) in lines.iter().enumerate() {
            mvwprintw(w, (index+1) as i32, 1, text);
        }
        box_(w, 0, 0);
        wrefresh(w);
    }

    /* Public */
    pub fn erase(&mut self) {
        erase();
        self.maxindex = 0;
    }

    pub fn refresh(&self) {
        refresh();
        if self.show_help {
            self.show_help_window();
        }
    }

    pub fn printline(&mut self, index: u32, msg: &str) {
        if self.selected_index == index {
            attron(A_REVERSE());
        }
        mvaddstr(index as i32, 0, msg);
        attroff(A_REVERSE());
        self.maxindex = max(self.maxindex, index);
    }

    pub fn printstatus(&self, msg: &str) {
        mvprintw((self.maxlines+1) as i32, 0, msg);
    }

    pub fn up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn down(&mut self) {
        if self.selected_index < min(self.maxlines, self.maxindex) {
            self.selected_index += 1;
        }
    }

    pub fn adjust_selection(&mut self) {
        self.selected_index = min(self.maxindex, self.selected_index);
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.refresh();
    }
}

