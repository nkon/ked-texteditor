use termion::*;
use getopts::Options;
use std::env;
use std::path::Path;
use serde::{Deserialize, Serialize};

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
    opts.optflag("d", "debug", "debug mode");
    opts.optopt("s", "script", "run script", "FILE");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        std::process::exit(0);
    }
    if let Ok((width, height)) = terminal_size() {
        let screen = Screen {
            width: width,
            height: height,
        };
        let editor_win = Window::new(1, 1, screen.width, screen.height - 2, screen);
        let status_win = Window::new(1, height - 1, screen.width, 1, screen);
        let prompt_win = Window::new(1, height, screen.width, 1, screen);
        let status_bar = StatusBar::new(status_win);
        let prompt_box = Prompt::new(prompt_win);
        let mut editor = Editor::new(editor_win, status_bar, prompt_box);

        if matches.opt_present("s") {
            let script_file = matches.opt_str("s").unwrap();
            let reader = std::io::BufReader::new(std::fs::File::open(script_file).unwrap());
            let s: Vec<MacroCommand> = serde_json::from_reader(reader).unwrap();
            editor.run_script(&s);
        } else if matches.free.is_empty() {
            editor.run_editor_with_new_buffer(matches.opt_present("d"));
        } else {
            let input_file_name = matches.free[0].clone();
            if Path::new(&input_file_name).exists() {
                editor.run_editor_with_file(&input_file_name, matches.opt_present("d"));
            } else {
                editor.run_editor_with_new_file(&input_file_name, matches.opt_present("d"));
            }
        }
    }
}

