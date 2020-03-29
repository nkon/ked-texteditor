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
        let status_bar = StatusBar::new(status_win);
        let mut editor = Editor::new(editor_win, status_bar);
        editor.run_editor_with_file(&input_file_name);
    }
}
