use core::fmt;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(transparent)]
pub struct BUFFER {
    characters : [[CHAR; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct SCREEN {
    col_pos : usize,
    row_pos : usize,
    color : ColorCode,
    buffer : &'static mut BUFFER
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct CHAR {
    code : u8,
    color : ColorCode
}

impl SCREEN {
    pub fn write_byte(&mut self, byte : u8) {
        match byte {
            b'\n' => self.new_line(),
            _ => {
                    if self.col_pos >= BUFFER_WIDTH {
                        self.new_line();
                    }
                    self.buffer.characters[self.row_pos][self.col_pos] = CHAR {code : byte, color : self.color};
                    self.col_pos += 1;
            }
        };
    }
    fn new_line(&mut self) -> () {
        self.col_pos = 0;
        self.row_pos += 1;
        if self.row_pos >= BUFFER_HEIGHT {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    self.buffer.characters[row - 1][col] = self.buffer.characters[row][col];
                }
            }
            self.clear_bottom();
            self.row_pos = BUFFER_HEIGHT - 1;
        }
    }
    fn clear_bottom(&mut self) -> () {
        let blank = CHAR {
            code : b' ',
            color : self.color
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.characters[BUFFER_HEIGHT - 1][col] = blank;
        }
    }
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }

        }
    }
    pub fn write_string_color(&mut self, s : &str, col : ColorCode) -> () {
        let old_color = self.color;
        self.set_color(col);
        self.write_string(s);
        self.set_color(old_color);
    }
    pub fn new(color : ColorCode, buffer : &'static mut BUFFER) -> Self {
        SCREEN {col_pos : 0, row_pos : 0, color : color, buffer : buffer}
    }
    pub fn set_color(&mut self, color : ColorCode) -> () {
        self.color = color;
    }
}

impl fmt::Write for SCREEN {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}