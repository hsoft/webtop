use std::cmp::{min, max};
use ncurses::{
    stdscr, getmaxy, getmaxx, mvprintw, erase, refresh, mvaddnstr,
    attron, attroff, A_REVERSE
};
use visits::Visit;
use help_panel::HelpPanel;
use visit_detail_panel::VisitDetailPanel;

pub struct Screen {
    scrx: u32,
    scry: u32,
    pub selected_index: u32,
    maxindex: u32,
    help_panel: HelpPanel,
    visit_detail_panel: VisitDetailPanel,
}

impl Screen {
    pub fn new() -> Screen {
        let scry = getmaxy(stdscr);
        let scrx = getmaxx(stdscr);

        Screen {
            scrx: scrx as u32,
            scry: scry as u32,
            selected_index: 0,
            maxindex: 0,
            help_panel: HelpPanel::new(scrx),
            visit_detail_panel: VisitDetailPanel::new(scry, scrx),
        }
    }

    /* Public */
    pub fn maxlines(&self) -> u32 {
        (self.scry - 2) as u32
    }

    pub fn erase(&mut self) {
        erase();
        self.maxindex = 0;
    }

    /* Returns whether there was something to close */
    pub fn close_top_dialog(&mut self) -> bool {
        if self.help_panel.is_visible() {
            self.help_panel.toggle();
            true
        }
        else if self.visit_detail_panel.is_visible() {
            self.visit_detail_panel.close();
            true
        }
        else {
            false
        }
    }

    pub fn refresh(&self) {
        refresh();
        self.visit_detail_panel.refresh();
        self.help_panel.refresh();
    }

    pub fn printline(&mut self, index: u32, msg: &str) {
        if self.selected_index == index {
            attron(A_REVERSE());
        }
        mvaddnstr(index as i32, 0, msg, self.scrx as i32);
        attroff(A_REVERSE());
        self.maxindex = max(self.maxindex, index);
    }

    pub fn printstatus(&self, msg: &str) {
        mvprintw((self.scry-1) as i32, 0, msg);
    }

    pub fn up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn down(&mut self) {
        if self.selected_index < min(self.maxlines(), self.maxindex) {
            self.selected_index += 1;
        }
    }

    pub fn adjust_selection(&mut self) {
        self.selected_index = min(self.maxindex, self.selected_index);
    }

    pub fn toggle_help(&mut self) {
        self.help_panel.toggle();
        self.refresh();
    }

    pub fn show_visit_details(&mut self, visit: &Visit) {
        self.visit_detail_panel.set_visit(visit)
    }
}

