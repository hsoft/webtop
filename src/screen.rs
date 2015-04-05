use std::cmp::{min, max};
use ncurses::{
    mvaddstr, erase, attron, attroff, A_REVERSE
};

pub struct Screen {
    pub maxlines: usize,
    pub selected_index: usize,
    maxindex: usize,
}

impl Screen {
    pub fn new(maxlines: usize) -> Screen {
        Screen {
            maxlines: maxlines,
            selected_index: 0,
            maxindex: 0,
        }
    }

    pub fn erase(&mut self) {
        erase();
        self.maxindex = 0;
    }

    pub fn printline(&mut self, index: usize, msg: &str) {
        if self.selected_index == index {
            attron(A_REVERSE());
        }
        mvaddstr(index as i32, 0, msg);
        attroff(A_REVERSE());
        self.maxindex = max(self.maxindex, index);
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
}

