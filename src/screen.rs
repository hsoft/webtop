use ncurses::{
    mvprintw, attron, attroff, A_REVERSE
};

pub struct Screen {
    pub maxlines: usize,
    pub selected_index: usize,
}

impl Screen {
    pub fn new(maxlines: usize) -> Screen {
        Screen {
            maxlines: maxlines,
            selected_index: 0,
        }
    }

    pub fn printline(&self, index: usize, msg: &str) {
        if self.selected_index == index {
            attron(A_REVERSE());
        }
        mvprintw(index as i32, 0, msg);
        attroff(A_REVERSE());
    }

    pub fn up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn down(&mut self) {
        if self.selected_index < self.maxlines {
            self.selected_index += 1;
        }
    }
}

