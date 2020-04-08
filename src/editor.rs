use std::io::{stdin, stdout, Write};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::*;

use std::str;

use crate::*;

enum AfterPrompt {
    SaveFileAs,
}

pub struct Editor {
    buf: EditBuffer,
    status: StatusBar,
    prompt: Prompt,
    input: String,
    prompt_mode: bool,
    after_prompt: Option<AfterPrompt>,
}

impl Editor {
    pub fn new(win: Window, status: StatusBar, prompt: Prompt) -> Self {
        Editor {
            buf: EditBuffer::new(win),
            status: status,
            prompt: prompt,
            input: String::from(""),
            prompt_mode: false,
            after_prompt: None,
        }
    }
    pub fn run_editor_with_new_buffer(&mut self, debug_mode: bool) {
        self.buf.new_buffer();
        self.status.set_file_name("[NEW FILE]");
        self.run_editor(debug_mode);
    }
    pub fn run_editor_with_new_file(&mut self, file_name: &str, debug_mode: bool) {
        self.status.set_file_name(file_name);
        self.run_editor(debug_mode);
    }
    pub fn run_editor_with_file(&mut self, file_name: &str, debug_mode: bool) {
        self.buf.load_file(file_name);
        self.status.set_file_name(file_name);
        self.run_editor(debug_mode);
    }
    fn run_editor(&mut self, debug_mode: bool) {
        let stdin = stdin();
        // let mut stdout = stdout().into_raw_mode().unwrap();
        let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        write!(stdout, "{}", clear::All).unwrap();
        write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();
        stdout.flush().unwrap();

        self.buf.redraw(&mut stdout);
        self.status.redraw(&mut stdout);
        write!(
            stdout,
            "{}",
            cursor::Goto(self.buf.window().scr_cur_x(), self.buf.window().scr_cur_y())
        )
        .unwrap();
        write!(stdout, "{}", cursor::Show).unwrap();
        stdout.flush().unwrap();
        for c in stdin.keys() {
            if self.prompt_mode == false {
                match c {
                    Ok(event::Key::Ctrl('c')) => break,
                    Ok(event::Key::Ctrl('s')) => {
                        self.buf.save_file();
                    }
                    Ok(event::Key::Ctrl('a')) => {
                        self.prompt_mode = true;
                        self.after_prompt = Some(AfterPrompt::SaveFileAs);
                        self.prompt.redraw(&mut stdout);
                    }
                    Ok(event::Key::PageDown) => {
                        self.buf.scrollup(1);
                        self.buf.redraw(&mut stdout);
                    }
                    Ok(event::Key::PageUp) => {
                        self.buf.scrolldown(1);
                        self.buf.redraw(&mut stdout);
                    }
                    Ok(event::Key::Insert) => {
                        self.status.toggle_insert_mode();
                    }
                    Ok(event::Key::Down) => {
                        self.buf.cursor_down(&mut stdout);
                    }
                    Ok(event::Key::Up) => {
                        self.buf.cursor_up(&mut stdout);
                    }
                    Ok(event::Key::Left) => {
                        self.buf.cursor_left(&mut stdout);
                    }
                    Ok(event::Key::Right) => {
                        self.buf.cursor_right(&mut stdout);
                    }
                    Ok(event::Key::Delete) => {
                        self.buf.delete_char();
                        self.buf.redraw(&mut stdout);
                    }
                    Ok(event::Key::Char(c)) => {
                        if c == '\n' {
                            if self.status.insert_mode_flag() {
                                self.buf.insert_newline();
                                self.buf.redraw(&mut stdout);
                            }
                        } else {
                            if self.status.insert_mode_flag() {
                                self.buf.insert_char(c);
                            } else {
                                self.buf.replace_char(c);
                                self.buf.set_cur_y(self.buf.cur_x() + 1);
                            }
                            self.buf.redraw(&mut stdout);
                        }
                    }
                    _ => {}
                }
                if debug_mode {
                    self.buf.disp_params(&mut stdout);
                }
                self.status.redraw(&mut stdout);
                write!(
                    stdout,
                    "{}",
                    cursor::Goto(self.buf.window().scr_cur_x(), self.buf.window().scr_cur_y())
                )
                .unwrap();
                stdout.flush().unwrap();
            } else {
                match c {
                    Ok(event::Key::Ctrl('c')) => {
                        self.prompt_mode = false;
                    }
                    Ok(event::Key::Backspace) => {
                        self.prompt.backspace();
                        self.prompt.redraw(&mut stdout);
                    }
                    Ok(event::Key::Char(c)) => {
                        if c == '\n' {
                            self.prompt_mode = false;
                            self.input = String::from(self.prompt.result());
                            self.prompt.clear(&mut stdout);
                            self.status.redraw(&mut stdout);
                            write!(
                                stdout,
                                "{}",
                                cursor::Goto(
                                    self.buf.window().scr_cur_x(),
                                    self.buf.window().scr_cur_y()
                                )
                            )
                            .unwrap();
                            stdout.flush().unwrap();
                            match &mut self.after_prompt {
                                Some(x) => {
                                    match x {
                                        AfterPrompt::SaveFileAs => {
                                            self.buf.save_file_as(&self.input);
                                        }
                                    }
                                    self.after_prompt = None;
                                }
                                None => {}
                            }
                        } else {
                            self.prompt.push(c);
                            self.prompt.redraw(&mut stdout);
                            stdout.flush().unwrap();
                        }
                    }
                    _ => {}
                }
                stdout.flush().unwrap();
            }
        }
        write!(stdout, "{}", cursor::Show).unwrap();
    }
    pub fn run_script(&mut self, script: &Vec<MacroCommand>) {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode().unwrap();
        // let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        write!(stdout, "{}", clear::All).unwrap();
        write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();
        stdout.flush().unwrap();

        self.buf.redraw(&mut stdout);
        self.status.redraw(&mut stdout);

        for cmd in script {
            match cmd.name.as_str() {
                "new_buffer" => {
                    self.buf.new_buffer();
                    self.status.set_file_name("[NEW FILE]");
                }
                "open_file" => {
                    self.buf.load_file(&cmd.argstr);
                    self.status.set_file_name(&cmd.argstr);
                }
                "save_file_as" => {
                    self.buf.save_file_as(&cmd.argstr);
                }
                "cursor_up" => {
                    self.buf.cursor_up(&mut stdout);
                }
                "cursor_down" => {
                    self.buf.cursor_down(&mut stdout);
                }
                "cursor_left" => {
                    self.buf.cursor_left(&mut stdout);
                }
                "cursor_right" => {
                    self.buf.cursor_right(&mut stdout);
                }
                "insert_char" => {
                    self.buf.insert_char(cmd.argstr.chars().nth(0).unwrap());
                    self.buf.redraw(&mut stdout);
                }
                "break" => {
                    break;
                }
                _ => {}
            }
            self.status.redraw(&mut stdout);
            write!(
                stdout,
                "{}",
                cursor::Goto(self.buf.window().scr_cur_x(), self.buf.window().scr_cur_y())
            )
            .unwrap();
            stdout.flush().unwrap();
        }
    }
}
