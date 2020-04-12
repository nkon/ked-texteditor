use std::io::Write;
use termion::*;

use std::str;

use crate::*;

pub struct Prompt {
    prompt: String,
    result: String,
    window: Window,
    cur_x: usize,
}

impl Prompt {
    pub fn new(window: Window) -> Self {
        Prompt {
            prompt: String::from(""),
            result: String::from(""),
            window: window,
            cur_x: 0,
        }
    }
    pub fn clear(&mut self, output: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        self.result = String::from("");
        for _ in 0..self.window.width() {
            self.result.push(' ');
        }
        write!(
            output,
            "{}{}{}{}",
            cursor::Goto(self.window.x(), self.window.y()),
            self.result,
            cursor::Goto(self.window.scr_cur_x(), self.window.scr_cur_y()),
            cursor::Show,
        )
        .unwrap();
        output.flush().unwrap();
        self.result = String::from("");
        self.cur_x = 0;
        self.window.set_cur_x(0);
    }
    pub fn set_prompt(&mut self, prompt_str: &str) {
        self.prompt = String::from(prompt_str);
        self.cur_x = self.prompt.len();
        self.window.set_cur_x(self.cur_x as u16);
    }
    pub fn backspace(&mut self) {
        self.result.pop();
    }
    pub fn push(&mut self, c: char) {
        self.result.push(c);
        self.cur_x += 1;
        let cur_x = self.window.cur_x() + 1;
        self.window.set_cur_x(cur_x);
    }
    pub fn redraw(&mut self, output: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write!(
            output,
            "{}{}{}{}{}",
            cursor::Goto(self.window.x(), self.window.y()),
            self.prompt,
            self.result,
            cursor::Goto(self.window.scr_cur_x(), self.window.scr_cur_y()),
            cursor::Show,
        )
        .unwrap();
        output.flush().unwrap();
    }
    pub fn result(&mut self) -> &str {
        eprintln!("prompt.rs:result:{}", self.result);
        &self.result
    }
    pub fn window(&mut self) -> &mut Window {
        &mut self.window
    }
}
