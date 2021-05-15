#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]

use alloc::vec::Vec;
use x86_64::instructions::port::Port;

use crate::data_storage::screen::Coord;
use crate::{debug, println, warningln};

/// COPY OF THE ONE IN MOD
/// A ColorCode is the data of a foreground color and a background one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ColorCode(pub u8);

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

impl CHAR {
    pub fn new(code: u8, color: ColorCode) -> Self {
        Self { code, color }
    }
}

/// This is the structure holding the layer index
/// of a process.
/// A higher layer index means the virtual screen will be
/// displayed more on the foreground.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualScreenLayer(pub usize);

impl VirtualScreenLayer {
    pub fn new(layer: usize) -> Self {
        Self(layer)
    }
}

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
    pub fn resize(&mut self, size: Coord) {
        let blank = CHAR {
            code: b' ',
            color: self.color,
        };
        let mut new_buffer = Vec::new();
        let _old_height = self.height;
        let _old_width = self.width;
        self.height = size.get_row();
        self.width = size.get_col();
        for _ in 0..self.height {
            let mut line = Vec::new();
            for _ in 0..self.width {
                line.push(blank);
            }
            new_buffer.push(line);
        }
        let _old = self.buffer.clone();
        self.buffer = new_buffer;
        self.row_pos = 0;
        self.col_pos = 0;
        // TODO write back the old buffer into the new one
        /*
        for i in 0..old_height {
            for j in 0..old_width {
                self.write_byte();
            }
        }
        */
    }
    pub fn replace(&mut self, location: Coord) {
        self.position = location;
    }
    pub fn get_char(&self, row: usize, col: usize) -> CHAR {
        self.buffer[row][col]
    }
    pub fn get_size(&self) -> Coord {
        Coord::new(self.width, self.height)
    }
    pub fn get_position(&self) -> Coord {
        self.position
    }
    /// Writes a byte on the screen.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.col_pos = 0,
            b'\x1b' => (), // Escape code
            _ => {
                if self.col_pos == self.width {
                    self.new_line()
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
        self.row_pos += 1; // + (self.col_pos / self.width)
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
        let char_vec = s.chars().collect::<Vec<char>>();
        let len = char_vec.len();
        let mut i = 0;
        while i < len {
            let byte = char_vec[i] as u8;
            match byte as u8 {
                b'\n' => self.new_line(),
                b'\r' => self.col_pos = 0,
                b'\x1b' => {
                    // Escape code
                    let mut end = i;
                    for j in i..len {
                        if char::is_alphabetic(char_vec[j]) {
                            end = j;
                            break;
                        }
                    }
                    assert!(end > i);
                    self.handle_escaped(&char_vec[i..=end]);
                    i = end
                }
                _ => {
                    if self.col_pos == self.width {
                        self.new_line()
                    }
                    self.buffer[self.row_pos][self.col_pos] = CHAR {
                        code: byte as u8,
                        color: self.color,
                    };
                    self.col_pos += 1;
                }
            };
            //self.set_cursor();
            i += 1;
        }
    }

    fn handle_escaped(&mut self, code: &[char]) {
        // Handle escape code
        let escaped_length = code.len();
        let terminator = code[escaped_length - 1];
        debug!("Got escaped code : {:?}", code);
        assert_eq!(code[0] as u8, b'\x1b');
        assert!(char::is_alphabetic(terminator));
        match terminator {
            'A' => {
                let n = code[2] as usize;
                if n >= self.row_pos {
                    self.row_pos = 0;
                } else {
                    self.row_pos -= n;
                }
            }
            'B' => {
                let n = code[2] as usize;
                if n + self.row_pos >= self.height - 1 {
                    self.row_pos = self.height - 1;
                } else {
                    self.row_pos += n;
                }
            }
            'C' => {
                let n = code[2] as usize;
                if n + self.col_pos >= self.width - 1 {
                    self.col_pos = self.width - 1;
                } else {
                    self.col_pos += n;
                }
            }
            'D' => {
                let n = code[2] as usize;
                if n >= self.col_pos {
                    self.col_pos = 0;
                } else {
                    self.col_pos -= n;
                }
            }
            'E' => (),
            'F' => (),
            'G' => (),
            'H' => (),
            'I' => (),
            'J' => {
                let n = code[2];
                match n as u8 {
                    0 => (),
                    1 => (),
                    2 => self._clear(),
                    _ => warningln!("Unknown J (clear screen) code : {}", n),
                }
            }
            'K' => (),
            'S' => (),
            'T' => (),
            'm' => {
                let n = code[2];
                match n as u8 {
                    1..=16 => {
                        // Change foreground color
                        let mut col = self.color.0;
                        col &= 0b11110000;
                        col += n as u8 - 1;
                        self.color = ColorCode(col)
                    }
                    21..=36 => {
                        // Change background color
                        let mut col = self.color.0;
                        col &= 0b00001111;
                        col += (n as u8 - 21) << 4;
                        self.color = ColorCode(col)
                    }
                    _ => warningln!("Unknown colour code : {}", n),
                }
            }
            _ => warningln!("Could not read escape code {:?}", code),
        }
    }

    pub fn write_byte_vec(&mut self, s: &[u8]) -> usize {
        let l = s.len();
        for byte in s {
            match byte {
                // useless match ?
                0x20..=0x7e | b'\n' | b'\r' => self.write_byte(*byte as u8),
                _ => self.write_byte(*byte as u8),
            }
        }
        l
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
    pub fn new(color: ColorCode, position: Coord, size: Coord, layer: VirtualScreenLayer) -> Self {
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
            println!("Row out of bounds");
            return;
        }
        if col >= self.width {
            println!("Col out of bounds");
            return;
        }
        self.row_pos = row;
        self.col_pos = col;
        self.write_string(s);
        self.row_pos = old_row;
        self.col_pos = old_col;
    }

    pub fn delete(&mut self) {}
}
