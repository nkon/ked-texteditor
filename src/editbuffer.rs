use std::io::Write;
use termion::*;

use std::fs::File;
use std::io::{BufRead, BufReader};

use std::str;

use unicode_width::*;

use crate::*;

/// Edit buffer. current impriment is Vec<String>
pub struct EditBuffer {
    buffer: Vec<String>,
    begin: usize, // line to start display
    cur_x: usize, // 0-index-ed buffer coodinates. char counting.
    cur_y: usize,
    file_name: String,
    window: Window, // Window information is cloned at the initalizing.
    cache_width: Vec<usize>,
    cache_size: Vec<usize>,
}

impl EditBuffer {
    pub fn new(win: Window) -> Self {
        Self {
            buffer: vec![] as Vec<String>,
            begin: 0,
            cur_x: 0,
            cur_y: 0,
            file_name: String::from(""),
            window: win,
            cache_width: vec![] as Vec<usize>,
            cache_size: vec![] as Vec<usize>,
        }
    }
    pub fn load_file(&mut self, file_name: &str) {
        for result in BufReader::new(File::open(file_name).unwrap()).lines() {
            match result {
                Ok(s) => self.buffer.push(s.clone()),
                Err(_) => self.buffer.push(String::new()),
            }
        }
        self.file_name = String::from(file_name);
        self.calc_line();
    }
    pub fn save_file(&mut self) {
        if let Ok(mut file) = File::create(self.file_name.clone()) {
            for line in &self.buffer {
                writeln!(file, "{}", line).unwrap();
            }
        }
    }
    /// return cursor x position on the buffer coodinate.
    pub fn cur_x(&self) -> usize {
        self.cur_x
    }
    /// set cursor x position on the buffer coodinate.
    pub fn set_cur_x(&mut self, x: usize) {
        if x <= self.current_line_len() {
            self.cur_x = x;
        } else {
            if self.current_line_len() > 0 {
                self.cur_x = self.current_line_len();
            } else {
                self.cur_x = 0;
            }
        }
    }
    pub fn update_win_cur(&mut self) {
        self.calc_line();
        self.set_cur_x(self.cur_x);
        let mut cursor_x = 0;
        for i in 0..self.cur_x {
            cursor_x += self.cache_width[i];
        }
        self.window.set_cur_x(cursor_x as u16);
        if self.cur_y >= self.begin {
            self.window.set_cur_y((self.cur_y - self.begin) as u16);
        }
    }
    /// return cursor y position on the buffer coodinate.
    pub fn cur_y(&self) -> usize {
        self.cur_y
    }
    /// set cursor y position on the buffer coodinate.
    pub fn set_cur_y(&mut self, y: usize) {
        if y < self.buffer.len() {
            self.cur_y = y
        }
        self.calc_line();
    }
    pub fn begin(&self) -> usize {
        self.begin
    }
    pub fn current_line_len(&self) -> usize {
        self.buffer[self.cur_y].chars().count()
    }
    pub fn scrollup(&mut self, n: usize) {
        if self.begin < self.buffer.len() - self.window.height() as usize + n {
            self.begin += n;
            self.set_cur_y(self.cur_y + n);
        }
    }
    pub fn scrolldown(&mut self, n: usize) {
        if self.begin >= n {
            self.begin -= n;
            if self.cur_y >= n {
                self.set_cur_y(self.cur_y - n);
            }
        }
    }
    pub fn cursor_down(&mut self, output: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        if self.cur_y() >= self.begin() + self.window().height() as usize - 1 {
            self.scrollup(1);
            self.redraw(output);
        } else {
            self.set_cur_y(self.cur_y() + 1);
        }
        self.update_win_cur();
        self.redraw_cursor(output);
    }
    fn calc_line(&mut self) {
        self.cache_size = vec![];
        self.cache_width = vec![];
        for uni_c in self.buffer[self.cur_y].chars() {
            self.cache_size.push(uni_c.len_utf8());
            self.cache_width.push(uni_c.width().unwrap());
        }
        self.cache_width.push(0); // dummy for newline
        self.cache_size.push(0); // dummy for newline
    }
    pub fn cursor_up(&mut self, output: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        if self.cur_y() > self.begin() {
            self.set_cur_y(self.cur_y() - 1);
            self.update_win_cur();
            self.redraw_cursor(output);
        } else {
            self.scrolldown(1);
            self.redraw(output);
        }
    }
    pub fn cursor_left(&mut self, output: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        if self.cur_x() > 0 {
            // move to prev char
            self.set_cur_x(self.cur_x() - 1);
            let mut cursor_x = 0;
            for i in 0..self.cur_x {
                cursor_x += self.cache_width[i];
            }
            self.window().set_cur_x(cursor_x as u16);
            self.redraw_cursor(output);
        } else if self.cur_x() == 0 {
            // cursor is top of the line
            if self.cur_y() != 0 {
                if self.window().cur_y() == 0 {
                    // scroll down and goto end of prev line
                    self.scrolldown(1);
                    self.update_win_cur();
                    self.set_cur_x(self.current_line_len());
                    self.update_win_cur();
                    self.redraw(output);
                } else {
                    // goto end of prev line
                    self.set_cur_y(self.cur_y() - 1);
                    self.set_cur_x(self.current_line_len());
                    self.update_win_cur();
                    self.redraw_cursor(output);
                }
            }
        }
    }
    pub fn cursor_right(&mut self, output: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        if self.cur_x() >= self.current_line_len() {
            // cursor is end of the line
            if self.window().cur_y() >= self.window().height() - 1 {
                // scroll up and goto top of next line
                self.scrollup(1);
                self.set_cur_x(0);
                self.update_win_cur();
                self.redraw(output);
            } else {
                if self.cur_y < self.buffer.len() - 1 {
                    // goto top of next line
                    self.set_cur_y(self.cur_y() + 1);
                    self.set_cur_x(0);
                    self.update_win_cur();
                    self.redraw_cursor(output);
                }
            }
        } else {
            // move to next char
            self.set_cur_x(self.cur_x() + 1);
            let mut cursor_x = 0;
            for i in 0..self.cur_x {
                cursor_x += self.cache_width[i];
            }
            self.window().set_cur_x(cursor_x as u16);
            self.redraw_cursor(output);
        }
    }
    pub fn replace_char(&mut self, ch: char) {
        self.set_cur_x(self.cur_x);
        if self.current_line_len() > 0 {
            let mut line: Vec<char> = self.buffer[self.cur_y].clone().chars().collect();
            line[self.cur_x] = ch;
            let mut s = String::new();
            for c in line {
                s.push(c)
            }
            self.buffer[self.cur_y] = s;
            self.window.set_cur_x(self.cur_x as u16);
            self.window.set_cur_y(self.cur_y as u16);
        }
    }
    pub fn insert_char(&mut self, ch: char) {
        self.set_cur_x(self.cur_x);
        if self.current_line_len() > 0 {  // insert char on the existing line
            let mut line: Vec<char> = self.buffer[self.cur_y].clone().chars().collect();
            line.insert(self.cur_x, ch);
            let mut s = String::new();
            for c in line {
                s.push(c)
            }
            self.cur_x += 1;
            self.buffer[self.cur_y] = s;
            self.calc_line();
            let mut cursor_x = 0;
            for i in 0..self.cur_x {
                cursor_x += self.cache_width[i];
            }
            self.window.set_cur_x(cursor_x as u16);
            self.window.set_cur_y(self.cur_y as u16);
        } else {  // insert char on the blank line
            self.buffer[self.cur_y] = String::new();
            self.buffer[self.cur_y].push(ch);
            self.cur_x = 1;
            self.calc_line();
            self.window.set_cur_x(self.cur_x as u16);
            self.window.set_cur_y(self.cur_y as u16);
        }
    }
    pub fn insert_newline(&mut self) {
        if self.current_line_len() > self.cur_x {
            // insert NEWLINE between existing line.
            let line1 = String::from(&self.buffer[self.cur_y][0..self.cur_x]);
            let line2 = String::from(&self.buffer[self.cur_y][self.cur_x..]);
            self.buffer[self.cur_y] = line1;
            self.buffer.insert(self.cur_y + 1, line2);
            self.cur_x = 0;
            self.cur_y += 1;
        } else if self.current_line_len() == 0 {
            // insert NEWLINE on the blank line.
            self.buffer.insert(self.cur_y + 1, String::from(""));
            self.cur_x = 0;
            self.cur_y += 1;
        } else if self.current_line_len() == self.cur_x {
            // append NEW line.
            self.buffer.insert(self.cur_y + 1, String::from(""));
            self.cur_x = 0;
            self.cur_y += 1;
        }
        self.update_win_cur();
    }
    pub fn delete_char(&mut self) {
        if self.current_line_len() > self.cur_x {
            // delete char between existing line.
            let mut line1 = String::from(&self.buffer[self.cur_y][0..self.cur_x]);
            let line2 = String::from(&self.buffer[self.cur_y][self.cur_x + 1..]);
            line1.push_str(&line2);
            self.buffer[self.cur_y] = line1;
        } else if self.current_line_len() == 0 {
            // delete blank line.
            self.buffer.remove(self.cur_y);
            self.cur_x = 0;
        } else if self.current_line_len() == self.cur_x {
            // delete NEWLINE at the end of line -> join to the next line.
            let mut line1 = String::from(&self.buffer[self.cur_y]);
            line1.push_str(&self.buffer[self.cur_y + 1]);
            self.buffer.remove(self.cur_y + 1);
            self.buffer[self.cur_y] = line1;
        }
        self.update_win_cur();
    }
    pub fn set_window(&mut self, win: Window) {
        self.window = win;
    }
    pub fn window(&mut self) -> &mut Window {
        &mut self.window
    }
    pub fn redraw(&mut self, output: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write!(output, "{}", clear::All).unwrap();
        write!(output, "{}", cursor::Goto(1, 1)).unwrap();
        for y in 0..self.window.height() as usize {
            let line = if self.buffer.len() > self.begin + y {
                &self.buffer[self.begin + y]
            } else {
                ""
            };
            let end = if line.len() > self.window.width() as usize {
                line.char_indices()
                    .nth(self.window.width() as usize)
                    .unwrap()
                    .0
            } else {
                line.len()
            };
            write!(output, "{}{}", cursor::Goto(1, y as u16 + 1), &line[0..end]).unwrap();
        }
        write!(
            output,
            "{}",
            cursor::Goto(self.window.scr_cur_x(), self.window.scr_cur_y())
        )
        .unwrap();
        output.flush().unwrap();
    }
    pub fn redraw_cursor(&self, output: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write!(
            output,
            "{}",
            cursor::Goto(self.window.scr_cur_x(), self.window.scr_cur_y())
        )
        .unwrap();
        output.flush().unwrap();
    }
    pub fn disp_params(&mut self, output: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write!(output, "{}", cursor::Goto(60, 1)).unwrap();
        let win_x = self.window.screen().width;
        let win_y = self.window.screen().height;
        write!(output, "screen({},{})", win_x, win_y).unwrap();
        write!(output, "{}", cursor::Goto(60, 2)).unwrap();
        write!(
            output,
            "win cur({},{})        ",
            self.window.cur_x(),
            self.window.cur_y()
        )
        .unwrap();
        write!(output, "{}", cursor::Goto(60, 3)).unwrap();
        write!(
            output,
            "buf cur({},{}) begin={}        ",
            self.cur_x, self.cur_y, self.begin
        )
        .unwrap();

        write!(
            output,
            "{}",
            cursor::Goto(self.window.scr_cur_x(), self.window.scr_cur_y())
        )
        .unwrap();
        output.flush().unwrap();
    }
}
