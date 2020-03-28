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

fn load_file_to_buffer(file_name: &str) -> Vec<String> {
    let mut lines = Vec::<String>::new();

    for result in BufReader::new(File::open(file_name).unwrap()).lines() {
        lines.push(result.unwrap().clone());
    }
    lines
}

struct Window {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

fn draw_buffer_to_window(
    buffer: &Vec<String>,
    from: usize,
    output: &mut termion::raw::RawTerminal<std::io::Stdout>,
    win: &Window,
) {
    write!(output, "{}", clear::All).unwrap();
    write!(output, "{}", cursor::Goto(win.x, win.y)).unwrap();
    for y in 0..win.height - 1 {
        let line = if buffer.len() > from + y as usize {
            &buffer[from + y as usize]
        } else {
            ""
        };
        let end = if line.len() > win.width as usize {
            line.char_indices().nth(win.width as usize).unwrap().0
        } else {
            line.len()
        };
        write!(
            output,
            "{}{}",
            cursor::Goto(1, y as u16 + 1),
            &line[0..end]
        )
        .unwrap();
    }
    write!(output, "{}", cursor::Goto(1, 1)).unwrap();
    output.flush().unwrap();
}

fn run_viewer_with_file(file_name: &str) {
    let buffer = load_file_to_buffer(file_name);

    let stdin = stdin();
    let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    write!(stdout, "{}", clear::All).unwrap();
    write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();
    stdout.flush().unwrap();

    let mut curx = 1;
    let mut cury = 1;


    if let Ok((width, height)) = terminal_size() {
        let win = Window {
            x: 1,
            y: 1,
            width: width,
            height: height,
        };
        let mut begin = 0;

        draw_buffer_to_window(&buffer, begin, &mut stdout, &win);

        for c in stdin.keys() {
            match c {
                Ok(event::Key::Ctrl('c')) => break,
                Ok(event::Key::PageDown) => {
                    if begin < buffer.len() - win.height as usize + 1 {
                        begin = begin + 1;
                        draw_buffer_to_window(&buffer, begin, &mut stdout, &win);
                    }
                }
                Ok(event::Key::PageUp) => {
                    if begin > 0 {
                        begin = begin - 1;
                        draw_buffer_to_window(&buffer, begin, &mut stdout, &win);
                    }
                }
                Ok(event::Key::Down) => {
                    cury += 1;
                    write!(stdout,"{}",cursor::Goto(curx,cury)).unwrap();
                    stdout.flush().unwrap();
                }
                Ok(event::Key::Up) => {
                    cury -= 1;
                    write!(stdout,"{}",cursor::Goto(curx,cury)).unwrap();
                    stdout.flush().unwrap();
                }
                Ok(event::Key::Left) => {
                    curx -= 1;
                    write!(stdout,"{}",cursor::Goto(curx,cury)).unwrap();
                    stdout.flush().unwrap();
                }
                Ok(event::Key::Right) => {
                    curx += 1;
                    write!(stdout,"{}",cursor::Goto(curx,cury)).unwrap();
                    stdout.flush().unwrap();
                }
                _ => {}
            }
        }
        write!(stdout, "{}", termion::cursor::Show).unwrap();
    }
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
        return;
    }
    if matches.free.is_empty() {
        print_usage(&program, opts);
    } else {
        let input_file_name = matches.free[0].clone();
        run_viewer_with_file(&input_file_name);
    }
}
