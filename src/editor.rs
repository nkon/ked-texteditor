use std::io::{stdin, stdout, Write};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::*;

use getopts::Options;
use std::env;

use std::fs::File;
use std::io::{BufRead, BufReader};

use std::str;

use crate::*;

pub struct Editor {
    buf: EditBuffer,
    status: StatusBar,
}

impl Editor {
    pub fn new(win: Window, status: StatusBar) -> Self {
        Editor {
            buf: EditBuffer::new(win),
            status: status,
        }
    }
    pub fn run_editor_with_file(&mut self, file_name: &str) {
        self.buf.load_file(file_name);
        self.status.set_file_name(file_name);

        let stdin = stdin();
        // let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        let mut stdout = stdout().into_raw_mode().unwrap();
        write!(stdout, "{}", clear::All).unwrap();
        write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();
        stdout.flush().unwrap();

        self.buf.redraw(&mut stdout);

        for c in stdin.keys() {
            match c {
                Ok(event::Key::Ctrl('c')) => break,
                Ok(event::Key::Ctrl('s')) => {
                    self.buf.save_file();
                }
                Ok(event::Key::PageDown) => {
                    self.buf.scrollup(1);
                    self.buf.redraw(&mut stdout);
                }
                Ok(event::Key::PageUp) => {
                    self.buf.scrolldown(1);
                    self.buf.redraw(&mut stdout);
                }
                Ok(event::Key::Insert) => {
                    self.status.toggle_insert_mode();
                }
                Ok(event::Key::Down) => {
                    if self.buf.cur_y() >= self.buf.begin() + self.buf.window().height() as usize - 1 {
                        self.buf.scrollup(1);
                        self.buf.redraw(&mut stdout);
                    } else {
                        self.buf.set_cur_y(self.buf.cur_y() + 1);
                    }
                    self.buf.update_win_cur();
                    self.buf.redraw_cursor(&mut stdout);
                }
                Ok(event::Key::Up) => {
                    if self.buf.cur_y() > self.buf.begin() {
                        self.buf.set_cur_y(self.buf.cur_y() - 1);
                        self.buf.update_win_cur();
                        self.buf.redraw_cursor(&mut stdout);
                    } else {
                        self.buf.scrolldown(1);
                        self.buf.redraw(&mut stdout);
                    }
                }
                Ok(event::Key::Left) => {
                    if self.buf.cur_x() > 0 {
                        self.buf.set_cur_x(self.buf.cur_x() - 1);
                        let u_x = self.buf.cur_x() as u16;
                        self.buf.window().set_cur_x(u_x);
                        self.buf.redraw_cursor(&mut stdout);
                    } else if self.buf.cur_x() == 0 {
                        if self.buf.cur_y() != 0 {
                            if self.buf.window().cur_y() == 0 {
                                self.buf.scrolldown(1);
                                self.buf.update_win_cur();
                                self.buf.set_cur_x(self.buf.current_line_len() + 1);
                                self.buf.update_win_cur();
                                self.buf.redraw(&mut stdout);
                            } else {
                                self.buf.set_cur_y(self.buf.cur_y() - 1);
                                self.buf.set_cur_x(self.buf.current_line_len() + 1);
                                self.buf.update_win_cur();
                                self.buf.redraw_cursor(&mut stdout);
                            }
                        }
                    }
                }
                Ok(event::Key::Right) => {
                    if self.buf.cur_x() >= self.buf.current_line_len() {
                        if self.buf.window().cur_y() >= self.buf.window().height() - 1 {
                            self.buf.scrollup(1);
                            self.buf.update_win_cur();
                            self.buf.set_cur_x(0);
                            self.buf.update_win_cur();
                            self.buf.redraw(&mut stdout);
                        } else {
                            self.buf.set_cur_y(self.buf.cur_y() + 1);
                            self.buf.set_cur_x(0);
                            self.buf.update_win_cur();
                            self.buf.redraw_cursor(&mut stdout);
                        }
                    } else {
                        self.buf.set_cur_x(self.buf.cur_x() + 1);
                        let u_x = self.buf.cur_x() as u16;
                        self.buf.window().set_cur_x(u_x);
                        self.buf.redraw_cursor(&mut stdout);
                    }
                }
                Ok(event::Key::Delete) => {
                    self.buf.delete_char();
                    self.buf.redraw(&mut stdout);
                }
                Ok(event::Key::Char(c)) => {
                    if c == '\n' {
                        if self.status.insert_mode_flag() {
                            self.buf.insert_newline();
                            self.buf.redraw(&mut stdout);
                        }
                    } else {
                        if self.status.insert_mode_flag() {
                            self.buf.insert_char(c);
                        } else {
                            self.buf.replace_char(c);
                            self.buf.set_cur_y(self.buf.cur_x() + 1);
                        }
                        self.buf.redraw(&mut stdout);
                    }
                }
                _ => {}
            }
            // buf.disp_params(&mut stdout);
            self.status.redraw(&mut stdout);
            write!(
                stdout,
                "{}",
                cursor::Goto(self.buf.window().scr_cur_x(), self.buf.window().scr_cur_y())
            )
            .unwrap();
            stdout.flush().unwrap();
        }
        write!(stdout, "{}", cursor::Show).unwrap();
    }
}
