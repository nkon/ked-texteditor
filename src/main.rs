use std::io::{stdin, stdout, Write};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
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
    left_top: u16,
    right_top: u16,
    width: u16,
    height: u16,
}

fn draw_buffer_to_window(buffer: Vec<String>, input: &std::io::Stdin, output: &mut termion::raw::RawTerminal<std::io::Stdout>, win: Window) {
    let height = win.height;
    for y in 0..win.height-1 {
        write!(output, "{}{}", cursor::Goto(1, y as u16 + 1), buffer[y as usize]).unwrap();
    }
    output.flush().unwrap();
}

fn run_viewer_with_file(file_name: &str) {
    let buffer = load_file_to_buffer(file_name);

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}{}", clear::All, cursor::Hide).unwrap();
    write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();
    stdout.flush().unwrap();

    if let Ok((width, height)) = terminal_size() {
        let mut win = Window{left_top: 1, right_top: 1, width: width, height: height};

        draw_buffer_to_window(buffer, &stdin, &mut stdout, win);

        for c in stdin.keys() {
            match c {
                Ok(event::Key::Ctrl('c')) => break,
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
