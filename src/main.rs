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

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&brief));
}

/// Get terminal information and hold.
#[derive(Clone)]
struct Screen {
    width: u16,
    height: u16,
}

/// Display window
#[derive(Clone)]
struct Window {
    x: u16, // left top position of the window, 1-index-ed screen coodinates
    y: u16,
    width: u16,
    height: u16,
    cur_x: u16, // cursor position: relative coodinates on the window, 0-index-ed.
    cur_y: u16,
    screen: Screen, // Screen information is cloned at the initalizing.
}

impl Window {
    /// return cursor x position on the screen coodinate.
    fn scr_cur_x(&self) -> u16 {
        self.cur_x + self.x
    }
    /// return cursor y position on the screen coodinate.
    fn scr_cur_y(&self) -> u16 {
        self.cur_y + self.y
    }
    /// set cursor x position on the window coodinate.
    fn set_cur_x(&mut self, x: u16) {
        if x < self.screen.width {
            self.cur_x = x
        }
    }
    /// set cursor y position on the window coodinate.
    fn set_cur_y(&mut self, y: u16) {
        if y < self.screen.height {
            self.cur_y = y
        }
    }
}

/// Edit buffer. current impriment is Vec<String>
struct EditBuffer {
    buffer: Vec<String>,
    cur_x: usize, // 0-index-ed.
    cur_y: usize,
    window: Window, // Window information is cloned at the initalizing.
}

impl EditBuffer {
    fn new(win: Window) -> Self {
        Self {
            buffer: vec![<String>::new()],
            cur_x: 0,
            cur_y: 0,
            window: win,
        }
    }
    fn load_file(&mut self, file_name: &str) {
        self.buffer.remove(0); // remove the first line which is allocated at the initalizing.
        for result in BufReader::new(File::open(file_name).unwrap()).lines() {
            self.buffer.push(result.unwrap().clone());
        }
    }
    /// return cursor x position on the buffer coodinate.
    fn cur_x(&self) -> usize {
        self.cur_x
    }
    /// set cursor x position on the buffer coodinate.
    fn set_cur_x(&mut self, x: usize) {
        if x < self.buffer[self.cur_y].len() {
            self.cur_x = x;
        } else {
            self.cur_x = self.buffer[self.cur_y].len() - 1;
        }
    }
    /// return cursor y position on the buffer coodinate.
    fn cur_y(&self) -> usize {
        self.cur_y
    }
    /// set cursor y position on the buffer coodinate.
    fn set_cur_y(&mut self, y: usize) {
        if y < self.buffer.len() {
            self.cur_y = y
        }
    }
    fn replace_char(&mut self, ch: char) {
        let mut line: Vec<char> = self.buffer[self.cur_y].clone().chars().collect();
        line[self.cur_x] = ch;
        let mut s = String::new();
        for c in line {
            s.push(c)
        }
        self.buffer[self.cur_y] = s;
    }
    fn set_window(&mut self, win: Window) {
        self.window = win;
    }
    fn window(&mut self) -> &mut Window {
        &mut self.window
    }
    fn redraw(&self, from: usize, output: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write!(output, "{}", clear::All).unwrap();
        write!(output, "{}", cursor::Goto(1, 1)).unwrap();
        for y in 0..self.window.height - 1 {
            let line = if self.buffer.len() > from + y as usize {
                &self.buffer[from + y as usize]
            } else {
                ""
            };
            let end = if line.len() > self.window.width as usize {
                line.char_indices()
                    .nth(self.window.width as usize)
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
}

fn run_viewer_with_file(file_name: &str, win: Window) {
    let mut buf = EditBuffer::new(win.clone());
    buf.load_file(file_name);
    buf.set_window(win);

    let stdin = stdin();
    let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    // let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", clear::All).unwrap();
    write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();
    stdout.flush().unwrap();

    let mut begin = 0;

    //    draw_buffer_to_window(&buf, begin, &mut stdout, &mut win);
    buf.redraw(begin, &mut stdout);

    for c in stdin.keys() {
        match c {
            Ok(event::Key::Ctrl('c')) => break,
            Ok(event::Key::PageDown) => {
                if begin < buf.buffer.len() - buf.window().height as usize + 1 {
                    begin = begin + 1;
                    buf.redraw(begin, &mut stdout);
                }
            }
            Ok(event::Key::PageUp) => {
                if begin > 0 {
                    begin = begin - 1;
                    buf.redraw(begin, &mut stdout);
                }
            }
            Ok(event::Key::Down) => {
                buf.set_cur_y(buf.cur_y() + 1);
                let u_y = buf.cur_y() as u16;
                buf.window().set_cur_y(u_y);
                write!(
                    stdout,
                    "{}",
                    cursor::Goto(buf.window().scr_cur_x(), buf.window().scr_cur_y())
                )
                .unwrap();
                stdout.flush().unwrap();
            }
            Ok(event::Key::Up) => {
                if buf.cur_y() > 0 {
                    buf.set_cur_y(buf.cur_y() - 1);
                    let u_y = buf.cur_y() as u16;
                    buf.window().set_cur_y(u_y);
                    write!(
                        stdout,
                        "{}",
                        cursor::Goto(buf.window().scr_cur_x(), buf.window().scr_cur_y())
                    )
                    .unwrap();
                    stdout.flush().unwrap();
                }
            }
            Ok(event::Key::Left) => {
                if buf.cur_x() > 0 {
                    buf.set_cur_x(buf.cur_x() - 1);
                    let u_x = buf.cur_x() as u16;
                    buf.window().set_cur_x(u_x);
                    write!(
                        stdout,
                        "{}",
                        cursor::Goto(buf.window().scr_cur_x(), buf.window().scr_cur_y())
                    )
                    .unwrap();
                    stdout.flush().unwrap();
                }
            }
            Ok(event::Key::Right) => {
                buf.set_cur_x(buf.cur_x() + 1);
                let u_x = buf.cur_x() as u16;
                buf.window().set_cur_x(u_x);
                write!(
                    stdout,
                    "{}",
                    cursor::Goto(buf.window().scr_cur_x(), buf.window().scr_cur_y())
                )
                .unwrap();
                stdout.flush().unwrap();
            }
            Ok(event::Key::Char(c)) => {
                buf.replace_char(c);
                buf.redraw(begin, &mut stdout);
            }
            _ => {}
        }
    }
    write!(stdout, "{}", termion::cursor::Show).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        std::process::exit(0);
    }
    if matches.free.is_empty() {
        print_usage(&program, opts);
        std::process::exit(0);
    }
    let input_file_name = matches.free[0].clone();
    if let Ok((width, height)) = terminal_size() {
        let screen = Screen {
            width: width,
            height: height,
        };
        let editor_win = Window {
            x: 1,
            y: 1,
            width: screen.width,
            height: screen.height,
            cur_x: 0,
            cur_y: 0,
            screen: screen,
        };
        run_viewer_with_file(&input_file_name, editor_win);
    }
}
