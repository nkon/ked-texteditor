use termion::*;
use std::io::{Write, stdout, stdin};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

const MESSAGE: &str = "Hello, world!";

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}{}", clear::All, cursor::Hide).unwrap();
    write!(stdout, "{}", cursor::Goto(1,1)).unwrap();
    stdout.flush().unwrap();

    for c in stdin.keys(){
        match c{
            Ok(event::Key::Char('m')) => {
                if let Ok((width, height)) = terminal_size() {
                    let x = width / 2 - (MESSAGE.len() / 2) as u16;
                    let y = height / 2;
                    write!(stdout, "{}{}{}{}{}{}",
                        clear::All,
                        cursor::Goto(x,y),
                        color::Fg(color::Blue),
                        style::Bold,
                        MESSAGE,
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
