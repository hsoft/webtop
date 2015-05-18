use ncurses::{newwin, box_, mvwprintw, wrefresh};

pub struct HelpPanel {
    scrx: i32,
    visible: bool,
}

impl HelpPanel {
    pub fn new(scrx: i32) -> HelpPanel {
        HelpPanel {
            scrx: scrx,
            visible: false,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn refresh(&self) {
        if !self.visible {
            return
        }
        let lines = [
            "h - Host mode",
            "p - Path mode",
            "r - Referer mode",
            "↑/↓ - Selection",
            "q - Quit/Close panel",
        ];
        let width = 25;
        let w = newwin(7, width, 1, self.scrx - width);
        for (index, text) in lines.iter().enumerate() {
            mvwprintw(w, (index+1) as i32, 1, text);
        }
        box_(w, 0, 0);
        wrefresh(w);
    }
}

