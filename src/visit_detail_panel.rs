use visits::Visit;
use ncurses::{newwin, box_, mvwinsnstr, wrefresh};

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
                let width = self.scrx / 2;
                let height = self.scry - 1;
                let lines = [
                    &visit.host[..],
                    &visit.fmt_time_range()[..],
                    &format!("Hits: {}", visit.hit_count)[..],
                    &format!("4xx: {}", visit.hit_4xx_count)[..],
                    &format!("5xx: {}", visit.hit_5xx_count)[..],
                    &visit.referer[..],
                    &visit.agent[..],
                ];
                let w = newwin(height, width, 0, self.scrx - width);
                for (index, text) in lines.iter().enumerate() {
                    mvwinsnstr(w, (index+1) as i32, 1, text, width-2);
                }
                box_(w, 0, 0);
                wrefresh(w);
            },
            None => return,
        };
    }
}
