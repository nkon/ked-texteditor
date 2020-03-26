use termion::*;
use std::io::{Write, stdout, stdin};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use getopts::Options;
use std::env;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&brief));
}

fn run_viewer_with_file(file_name: &str) {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}{}", clear::All, cursor::Hide).unwrap();
    write!(stdout, "{}", cursor::Goto(1,1)).unwrap();
    stdout.flush().unwrap();

    for c in stdin.keys(){
        match c{
            Ok(event::Key::Char('m')) => {
                if let Ok((width, height)) = terminal_size() {
                    let x = width / 2 - (file_name.len() / 2) as u16;
                    let y = height / 2;
                    write!(stdout, "{}{}{}{}{}{}",
                        clear::All,
                        cursor::Goto(x,y),
                        color::Fg(color::Blue),
                        style::Bold,
                        file_name,
                        style::Reset,
                    ).unwrap();
                    stdout.flush().unwrap();
                }
            },
            Ok(event::Key::Ctrl('c')) => break,
            _ => {},
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
        Ok(m) => { m },
        Err(f) => {panic!(f.to_string())}
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
