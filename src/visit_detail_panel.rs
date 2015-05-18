use visits::Visit;
use ncurses::{newwin, box_, mvwprintw, wrefresh};

pub struct VisitDetailPanel {
    scry: i32,
    scrx: i32,
    visit: Option<Visit>,
}

impl VisitDetailPanel {
    pub fn new(scry: i32, scrx: i32) -> VisitDetailPanel {
        VisitDetailPanel {
            scry: scry,
            scrx: scrx,
            visit: None,
        }
    }

    pub fn is_visible(&self) -> bool {
        match self.visit {
            Some(_) => true,
            None => false,
        }
    }

    pub fn close(&mut self) {
        self.visit = None;
    }

    pub fn set_visit(&mut self, visit: &Visit) {
        self.visit = Some(visit.clone());
    }

    pub fn refresh(&self) {
        match self.visit {
            Some(ref visit) => {
                let width = 30;
                let height = self.scry - 1;
                let w = newwin(height, width, 0, self.scrx - width);
                box_(w, 0, 0);
                mvwprintw(w, 1, 1, &visit.host[..]);
                wrefresh(w);
            },
            None => return,
        };
    }
}
