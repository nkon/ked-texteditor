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
            cur_x: 0, // cursor point of the buffer(0-indexed)
            cur_y: 0,
            file_name: String::from(""),
            window: win,
            cache_width: vec![] as Vec<usize>,
            cache_size: vec![] as Vec<usize>,
        }
    }
    pub fn load_file(&mut self, file_name: &str) -> Result<&Self, &str> {
        if let Ok(file) = File::open(file_name) {
            for result in BufReader::new(file).lines() {
                match result {
                    Ok(s) => self.buffer.push(s.clone()),
                    Err(_) => self.buffer.push(String::new()),
                }
            }
            self.file_name = file_name.to_string();
            self.calc_line();
            Ok(self)
        } else {
            Err("Cannot load file")
        }
    }
    pub fn save_file(&mut self) -> Result<&Self, &str> {
        if self.file_name == "" {
            eprintln!("save_file: No File Name");
            Err("No File Name")
        } else {
            let mut file = File::create(self.file_name.clone());
            match &mut file {
                Ok(file) => {
                    for line in &self.buffer {
                        writeln!(file, "{}", line).unwrap();
                    }
                    Ok(self)
                }
                Err(_) => {
                    eprintln!("Cannot create file:{}", self.file_name);
                    Err("Cannot create file")
                }
            }
        }
    }
    pub fn save_file_as(&mut self, file_name: &str) {
        if let Ok(mut file) = File::create(file_name) {
            eprintln!("save_file_as:{}", file_name);
            for line in &self.buffer {
                writeln!(file, "{}", line).unwrap();
            }
            self.file_name = file_name.to_string();
        } else {
            eprintln!("save_file_as:can not write:{}", file_name);
        }
    }
    pub fn new_buffer(&mut self) {
        self.buffer = vec![String::from("")];
    }
    pub fn file_name(&self) -> &str {
        &self.file_name
    }
    pub fn set_file_name(&mut self, file_name: &str) {
        self.file_name = file_name.to_string();
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
        if self.cur_y() >= self.begin + self.window().height() as usize - 1 {
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
        if self.cur_y() > self.begin {
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
        if self.current_line_len() > 0 {
            // insert char on the existing line
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
        } else {
            // insert char on the blank line
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
            let mut line2 = self.buffer[self.cur_y].clone();
            let mut line1 = String::new();
            for _ in 0..self.cur_x {
                line1.push(line2.remove(0));
            }
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
            let mut line: Vec<char> = self.buffer[self.cur_y].clone().chars().collect();
            line.remove(self.cur_x);
            let mut line2 = String::new();
            for c in line {
                line2.push(c);
            }
            self.buffer[self.cur_y] = line2;
        } else if self.current_line_len() == 0 {
            // delete blank line.
            self.buffer.remove(self.cur_y);
            self.cur_x = 0;
        } else if self.current_line_len() == self.cur_x {
            if self.cur_y < self.buffer.len() - 1 {
                // delete NEWLINE at the end of line -> join to the next line.
                let mut line1 = String::from(&self.buffer[self.cur_y]);
                line1.push_str(&self.buffer[self.cur_y + 1]);
                self.buffer.remove(self.cur_y + 1);
                self.buffer[self.cur_y] = line1;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::stdout;
    use termion::raw::IntoRawMode;

    #[test]
    fn load_file_existing_file() {
        let screen = Screen {
            width: 80,
            height: 25,
        };
        let window = Window::new(1, 1, 80, 24, screen);
        let mut buf = EditBuffer::new(window);
        buf.load_file("src/main.rs").unwrap();
        assert_eq!(buf.file_name, "src/main.rs");
    }
    #[test]
    fn load_file_not_exist() {
        let screen = Screen {
            width: 80,
            height: 25,
        };
        let window = Window::new(1, 1, 80, 24, screen);
        let mut buf = EditBuffer::new(window);
        let file_name = "tests/not_exist.txt";
        assert!(buf.load_file(file_name).is_err());
    }

    #[test]
    fn set_cur_x_1() {
        let screen = Screen {
            width: 80,
            height: 25,
        };
        let window = Window::new(1, 1, 80, 24, screen);
        let mut buf = EditBuffer::new(window);
        buf.new_buffer();
        buf.buffer[0] = String::from("12345");
        buf.set_cur_y(0);
        buf.set_cur_x(5);
        assert_eq!(buf.cur_x, 5);
    }

    #[test]
    fn set_cur_x_truncated() {
        let screen = Screen {
            width: 80,
            height: 25,
        };
        let window = Window::new(1, 1, 80, 24, screen);
        let mut buf = EditBuffer::new(window);
        buf.new_buffer();
        buf.buffer[0] = String::from("12345");
        buf.set_cur_y(0);
        buf.set_cur_x(6);
        assert_eq!(buf.cur_x, 5); // truncated.
    }
    #[test]
    fn set_cur_x_wchar() {
        let screen = Screen {
            width: 80,
            height: 25,
        };
        let window = Window::new(1, 1, 80, 24, screen);
        let mut buf = EditBuffer::new(window);
        buf.new_buffer();
        buf.buffer[0] = String::from("あいうえお");
        buf.set_cur_y(0);
        buf.set_cur_x(5);
        assert_eq!(buf.cur_x, 5); // truncated.
    }
    #[test]
    fn set_cur_x_wchar_truncated() {
        let screen = Screen {
            width: 80,
            height: 25,
        };
        let window = Window::new(1, 1, 80, 24, screen);
        let mut buf = EditBuffer::new(window);
        buf.new_buffer();
        buf.buffer[0] = String::from("あいうえお");
        buf.set_cur_y(0);
        buf.set_cur_x(6);
        assert_eq!(buf.cur_x, 5); // truncated.
    }

    #[test]
    fn current_line_len_1() {
        let screen = Screen {
            width: 80,
            height: 25,
        };
        let window = Window::new(1, 1, 80, 24, screen);
        let mut buf = EditBuffer::new(window);
        buf.new_buffer();
        buf.buffer[0] = String::from("12345");
        buf.set_cur_y(0);
        assert_eq!(buf.current_line_len(), 5); // truncated.
    }

    #[test]
    fn current_line_len_wchar() {
        let screen = Screen {
            width: 80,
            height: 25,
        };
        let window = Window::new(1, 1, 80, 24, screen);
        let mut buf = EditBuffer::new(window);
        buf.new_buffer();
        buf.buffer[0] = String::from("あいうえお");
        buf.set_cur_y(0);
        assert_eq!(buf.current_line_len(), 5); // truncated.
    }

    #[test]
    fn current_line_len_wchar_2() {
        let screen = Screen {
            width: 80,
            height: 25,
        };
        let window = Window::new(1, 1, 80, 24, screen);
        let mut buf = EditBuffer::new(window);
        buf.new_buffer();
        buf.buffer[0] = String::from("🍎🍊🍣❤👉");
        buf.set_cur_y(0);
        assert_eq!(buf.current_line_len(), 5); // truncated.
    }
    #[test]
    fn cursor_down_1() {
        let mut stdout = stdout().into_raw_mode().unwrap();
        let screen = Screen {
            width: 80,
            height: 25,
        };
        let window = Window::new(1, 1, 80, 24, screen);
        let mut buf = EditBuffer::new(window);
        buf.new_buffer();
        buf.buffer[0] = String::from("12345");
        buf.buffer.push(String::from(""));
        buf.set_cur_y(0);
        buf.set_cur_x(2);
        buf.cursor_down(&mut stdout);
        assert_eq!(buf.cur_x(), 0);
        assert_eq!(buf.cur_y(), 1);
    }
    #[test]
    fn cursor_down_2() {
        let mut stdout = stdout().into_raw_mode().unwrap();
        let screen = Screen {
            width: 80,
            height: 25,
        };
        let window = Window::new(1, 1, 80, 24, screen);
        let mut buf = EditBuffer::new(window);
        buf.new_buffer();
        buf.buffer[0] = String::from("12345");
        buf.buffer.push(String::from("あいう"));
        buf.set_cur_y(0);
        buf.set_cur_x(5);
        buf.cursor_down(&mut stdout);
        assert_eq!(buf.cur_x(), 3);
        assert_eq!(buf.cur_y(), 1);
    }
    #[test]
    fn cursor_down_3() {
        let mut stdout = stdout().into_raw_mode().unwrap();
        let screen = Screen {
            width: 80,
            height: 25,
        };
        let window = Window::new(1, 1, 80, 24, screen);
        let mut buf = EditBuffer::new(window);
        buf.new_buffer();
        buf.buffer[0] = String::from("あいうえお");
        buf.buffer.push(String::from("1234567890"));
        buf.set_cur_y(0);
        buf.set_cur_x(5);
        buf.cursor_down(&mut stdout);
        assert_eq!(buf.cur_x(), 5);
        assert_eq!(buf.cur_y(), 1);
    }
}
