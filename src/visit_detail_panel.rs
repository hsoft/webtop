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

    // Private
    fn _output_contents(&self, visit: &Visit) {
        let width = self.scrx / 2;
        let height = self.scry - 1;
        let lines = [
            &visit.host[..],
            &visit.fmt_time_range()[..],
            &visit.fmt_bytes()[..],
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
        let hits_startline = lines.len() + 2;
        let hits_height = (height as usize) - hits_startline - 1;
        let hits_overflow = visit.hits.len() > hits_height;
        let take_hits = if hits_overflow { hits_height - 1} else { hits_height };
        for (index, hit) in visit.hits.iter().take(take_hits).enumerate() {
            let fmt = format!("{} {} {}", hit.fmt_time(), hit.status, hit.path);
            mvwinsnstr(w, (index+hits_startline) as i32, 1, &fmt, width-2);
        }
        if hits_overflow {
            let count = visit.hits.len() - take_hits;
            let fmt = format!("[{} more hits]", count);
            mvwinsnstr(w, height-2, 1, &fmt, width-2);
        }
        box_(w, 0, 0);
        wrefresh(w);
    }

    // Public
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
            Some(ref visit) => self._output_contents(visit),
            None => return,
        };
    }
}
