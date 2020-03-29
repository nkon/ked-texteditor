use crate::*;

/// Display window
#[derive(Clone)]
pub struct Window {
    x: u16, // left top position of the window, 1-index-ed screen coodinates
    y: u16,
    width: u16,
    height: u16,
    cur_x: u16, // cursor position: relative coodinates on the window, 0-index-ed.
    cur_y: u16,
    screen: Screen, // Screen information is cloned at the initalizing.
}

impl Window {
    pub fn new(x: u16, y: u16, width: u16, height: u16, screen: Screen) -> Self {
        Window {
            x: x,
            y: y,
            width: width,
            height: height,
            cur_x: 0,
            cur_y: 0,
            screen: screen,
        }
    }
    /// return cursor x position on the screen coodinate.
    pub fn scr_cur_x(&self) -> u16 {
        self.cur_x + self.x
    }
    /// return cursor y position on the screen coodinate.
    pub fn scr_cur_y(&self) -> u16 {
        self.cur_y + self.y
    }
    /// set cursor x position on the window coodinate.
    pub fn set_cur_x(&mut self, x: u16) {
        if x < self.width {
            self.cur_x = x
        }
    }
    /// set cursor y position on the window coodinate.
    pub fn set_cur_y(&mut self, y: u16) {
        if y < self.height {
            self.cur_y = y
        }
    }
    pub fn height(&mut self) -> u16 {
        self.height
    }
    pub fn width(&mut self) -> u16 {
        self.width
    }
    pub fn cur_x(&mut self) -> u16 {
        self.cur_x
    }
    pub fn cur_y(&mut self) -> u16 {
        self.cur_y
    }
    pub fn screen(&mut self) -> &Screen {
        &self.screen
    }
}
