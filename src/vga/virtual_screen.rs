use alloc::vec::Vec;
use x86_64::{addr::VirtAddrNotValid, instructions::port::Port};

use crate::data_storage::screen::Coord;

/// COPY OF THE ONE IN MOD
/// A ColorCode is the data of a foreground color and a background one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ColorCode(u8);

/// COPY OF THE ONE IN MOD
/// This is the base element, stored in the screen buffer.
///
/// # Fields
/// * `code` - ASCII code of the character
/// * `color` - color code of the character, 8-bit integer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct CHAR {
    code: u8,
    color: ColorCode,
}

/// This is the structure holding the layer index
/// of a process.
/// A higher layer index means the virtual screen will be
/// displayed more on the foreground.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualScreenLayer(usize);

/// This is the virtual screen assigned to a process
#[derive(Debug, Clone, Hash, PartialEq)]
pub struct VirtualScreen {
    width: usize,
    height: usize,
    col_pos: usize,
    row_pos: usize,
    position: Coord,
    color: ColorCode,
    layer: VirtualScreenLayer,
    /// First coordinate is the row
    buffer: Vec<Vec<CHAR>>,
}

impl VirtualScreen {
    /// Writes a byte on the screen.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.row_pos = 0,
            _ => {
                if self.col_pos + self.col_pos * self.width == self.width * self.height - 1 {
                    if self.col_pos == 0 {
                        self.new_line();
                        panic!("too many words");
                    }
                    self.scroll_up();
                }
                self.buffer[self.row_pos][self.col_pos] = CHAR {
                    code: byte,
                    color: self.color,
                };
                self.col_pos += 1;
            }
        };
        //self.set_cursor();
    }

    /// FOR NOW DISABLED
    /// Moves the cursor given the information in the `Screen` struct.
    fn set_cursor(&mut self) {
        let pos = self.row_pos * self.width + self.col_pos;
        let mut port1 = Port::new(0x3D4);
        let mut port2 = Port::new(0x3D5);
        unsafe {
            port1.write(0x0F_u8);
            port2.write((pos & 0xFF) as u8);
            port1.write(0x0E_u8);
            port2.write(((pos >> 8) & 0xFF) as u8)
        }
    }
    /// This functions positions the character pointer to the following line.
    /// If the screens overflows, it get scrolled up.
    fn new_line(&mut self) {
        self.row_pos += 1 + (self.col_pos / self.width);
        self.col_pos = 0;
        while self.row_pos >= self.height {
            self.scroll_up();
            self.clear_bottom();
            self.row_pos = self.height - 1;
        }
    }

    /// This function scrolls the entire screen by one row upwards.
    fn scroll_up(&mut self) {
        for row in 1..self.height {
            for col in 0..self.width {
                self.buffer[row - 1][col] = self.buffer[row][col];
            }
        }
    }

    /// This function wipes the last line of the screen.
    fn clear_bottom(&mut self) {
        let blank = CHAR {
            code: b' ',
            color: self.color,
        };
        for col in 0..self.width {
            self.buffer[self.height - 1][col] = blank;
        }
    }

    /// This function writes a string on the screen, starting at the current position of the cursor.
    ///
    /// # Arguments
    /// * `s : &str` - the string to print.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.chars() {
            match byte as u8 {
                // useless match ?
                0x20..=0x7e | b'\n' | b'\r' => self.write_byte(byte as u8),
                _ => self.write_byte(byte as u8),
            }
        }
    }

    /// This function writes a string on the screen of the given color, starting at the current position of the cursor.
    ///
    /// # Arguments
    /// * `s : &str` - the string to print
    /// * `col : ColorCode` - the color in which the string will be printed
    pub fn _write_string_color(&mut self, s: &str, col: ColorCode) {
        let old_color = self.color;
        self.set_color(col);
        self.write_string(s);
        self.set_color(old_color);
        //println!("s = {}", s.bytes().len());
    }

    /// Initializes a new screen, with a given color and buffer.
    fn new(color: ColorCode, position: Coord, size: Coord, layer: VirtualScreenLayer) -> Self {
        let blank = CHAR { code: b' ', color };
        let col_size = size.get_col();
        let row_size = size.get_row();
        let mut buffer = Vec::new();
        for _ in 0..row_size {
            let mut new = Vec::new();
            for _ in 0..col_size {
                new.push(blank);
            }
            buffer.push(new);
        }
        Self {
            col_pos: 0,
            row_pos: 0,
            color,
            buffer,
            width: col_size,
            height: row_size,
            position,
            layer,
        }
    }

    /// The function changes the color of the cursor (the color which the next characters will be printed in)
    ///
    /// # Arguments
    /// * `color : ColorCode` - the color to be given to the cursor
    pub fn set_color(&mut self, color: ColorCode) {
        self.color = color;
    }

    /// This function clears the screen.
    ///
    /// # Result
    /// * The screen is cleared, and `Ok(()) : Result<(), VgaError<'_>>` is returned.
    pub fn _clear(&mut self) {
        let blank = CHAR {
            code: b' ',
            color: self.color,
        };
        for row in 0..self.height {
            for col in 0..self.width {
                self.buffer[row][col] = blank;
            }
        }
        self.col_pos = 0;
        self.row_pos = 0;
    }

    /// This function writes a given string at a given position on the screen.
    ///
    /// # Arguments
    /// * `row : usize` : row to which the string should be printed
    /// * `col : usize` : column to which the string should be printed
    /// * `s : &str` : the string that should be printed
    pub fn write_to_pos(&mut self, row: usize, col: usize, s: &str) {
        let old_row = self.row_pos;
        let old_col = self.col_pos;
        if row >= self.height {
            //println!("Row out of bounds");
            return;
        }
        if col >= self.width {
            //println!("Col out of bounds");
            return;
        }
        self.row_pos = row;
        self.col_pos = col;
        self.write_string(s);
        self.row_pos = old_row;
        self.col_pos = old_col;
    }
}
