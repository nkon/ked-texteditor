use std::io::Write;
use termion::*;

use std::str;

use crate::*;

pub struct StatusBar {
    file_name: String,
    insert_mode_flag: bool,
    window: Window,
    changed: bool,
}

impl StatusBar {
    pub fn new(window: Window) -> Self {
        StatusBar {
            file_name: String::from(""),
            insert_mode_flag: true,
            window: window,
            changed: false,
        }
    }
    pub fn redraw(&mut self, output: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        let mut bar = String::from("");
        for _ in 0..self.window.width() {
            bar.push(' ');
        }
        bar.replace_range(0..self.file_name.len(), &self.file_name);
        bar.replace_range(
            bar.len() - 5..bar.len() - 4,
            if self.changed == true { "*" } else { " " },
        );
        bar.replace_range(
            bar.len() - 3..bar.len(),
            if self.insert_mode_flag == true {
                "Ins"
            } else {
                "Ovr"
            },
        );

        write!(
            output,
            "{}{}{}{}{}{}{}",
            cursor::Goto(self.window.scr_cur_x(), self.window.scr_cur_y()),
            color::Fg(color::Black),
            color::Bg(color::White),
            bar,
            color::Fg(color::White),
            color::Bg(color::Black),
            cursor::Show,
        )
        .unwrap();
        output.flush().unwrap();
    }
    pub fn toggle_insert_mode(&mut self) {
        if self.insert_mode_flag {
            self.insert_mode_flag = false;
        } else {
            self.insert_mode_flag = true;
        }
    }
    pub fn insert_mode_flag(&self) -> bool {
        self.insert_mode_flag
    }
    pub fn set_file_name(&mut self, file_name: &str) {
        self.file_name = String::from(file_name);
    }
    pub fn set_changed(&mut self, changed: bool) {
        self.changed = changed;
    }
    pub fn changed(self) -> bool {
        self.changed
    }
}
