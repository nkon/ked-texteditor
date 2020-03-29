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

use ked::*;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&brief));
}

struct StatusBar {
    file_name: String,
    insert_mode_flag: bool,
    window: Window,
}

impl StatusBar {
    fn redraw(&mut self, output: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        let mut bar = String::from("");
        for _ in 0..self.window.width() {
            bar.push(' ');
        }
        bar.replace_range(0..self.file_name.len(), &self.file_name);
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
    fn toggle_insert_mode(&mut self) {
        if self.insert_mode_flag {
            self.insert_mode_flag = false;
        } else {
            self.insert_mode_flag = true;
        }
    }
    fn set_file_name(&mut self, file_name: &str) {
        self.file_name = String::from(file_name);
    }
}

fn run_editor_with_file(file_name: &str, win: Window, mut status: StatusBar) {
    let mut buf = EditBuffer::new(win.clone());
    buf.load_file(file_name);
    buf.set_window(win);
    status.set_file_name(file_name);

    let stdin = stdin();
    // let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", clear::All).unwrap();
    write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();
    stdout.flush().unwrap();

    buf.redraw(&mut stdout);

    for c in stdin.keys() {
        match c {
            Ok(event::Key::Ctrl('c')) => break,
            Ok(event::Key::Ctrl('s')) => {
                buf.save_file();
            }
            Ok(event::Key::PageDown) => {
                buf.scrollup(1);
                buf.redraw(&mut stdout);
            }
            Ok(event::Key::PageUp) => {
                buf.scrolldown(1);
                buf.redraw(&mut stdout);
            }
            Ok(event::Key::Insert) => {
                status.toggle_insert_mode();
            }
            Ok(event::Key::Down) => {
                if buf.cur_y() >= buf.begin() + buf.window().height() as usize - 1 {
                    buf.scrollup(1);
                    buf.redraw(&mut stdout);
                } else {
                    buf.set_cur_y(buf.cur_y() + 1);
                }
                buf.update_win_cur();
                buf.redraw_cursor(&mut stdout);
            }
            Ok(event::Key::Up) => {
                if buf.cur_y() > buf.begin() {
                    buf.set_cur_y(buf.cur_y() - 1);
                    buf.update_win_cur();
                    buf.redraw_cursor(&mut stdout);
                } else {
                    buf.scrolldown(1);
                    buf.redraw(&mut stdout);
                }
            }
            Ok(event::Key::Left) => {
                if buf.cur_x() > 0 {
                    buf.set_cur_x(buf.cur_x() - 1);
                    let u_x = buf.cur_x() as u16;
                    buf.window().set_cur_x(u_x);
                    buf.redraw_cursor(&mut stdout);
                } else if buf.cur_x() == 0 {
                    if buf.cur_y() != 0 {
                        if buf.window().cur_y() == 0 {
                            buf.scrolldown(1);
                            buf.update_win_cur();
                            buf.set_cur_x(buf.current_line_len() + 1);
                            buf.update_win_cur();
                            buf.redraw(&mut stdout);
                        } else {
                            buf.set_cur_y(buf.cur_y() - 1);
                            buf.set_cur_x(buf.current_line_len() + 1);
                            buf.update_win_cur();
                            buf.redraw_cursor(&mut stdout);
                        }
                    }
                }
            }
            Ok(event::Key::Right) => {
                if buf.cur_x() >= buf.current_line_len() {
                    if buf.window().cur_y() >= buf.window().height() - 1 {
                        buf.scrollup(1);
                        buf.update_win_cur();
                        buf.set_cur_x(0);
                        buf.update_win_cur();
                        buf.redraw(&mut stdout);
                    } else {
                        buf.set_cur_y(buf.cur_y() + 1);
                        buf.set_cur_x(0);
                        buf.update_win_cur();
                        buf.redraw_cursor(&mut stdout);
                    }
                } else {
                    buf.set_cur_x(buf.cur_x() + 1);
                    let u_x = buf.cur_x() as u16;
                    buf.window().set_cur_x(u_x);
                    buf.redraw_cursor(&mut stdout);
                }
            }
            Ok(event::Key::Delete) => {
                buf.delete_char();
                buf.redraw(&mut stdout);
            }
            Ok(event::Key::Char(c)) => {
                if c == '\n' {
                    if status.insert_mode_flag {
                        buf.insert_newline();
                        buf.redraw(&mut stdout);
                    }
                } else {
                    if status.insert_mode_flag {
                        buf.insert_char(c);
                    } else {
                        buf.replace_char(c);
                        buf.set_cur_y(buf.cur_x() + 1);
                    }
                    buf.redraw(&mut stdout);
                }
            }
            _ => {}
        }
        // buf.disp_params(&mut stdout);
        status.redraw(&mut stdout);
        write!(
            stdout,
            "{}",
            cursor::Goto(buf.window().scr_cur_x(), buf.window().scr_cur_y())
        )
        .unwrap();
        stdout.flush().unwrap();
    }
    write!(stdout, "{}", cursor::Show).unwrap();
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
        let editor_win = Window::new(1, 1, screen.width, screen.height - 2, screen);
        let status_win = Window::new(1, height - 1, screen.width, 2, screen);
        let status_bar = StatusBar {
            file_name: String::from(""),
            insert_mode_flag: true,
            window: status_win,
        };
        run_editor_with_file(&input_file_name, editor_win, status_bar);
    }
}
