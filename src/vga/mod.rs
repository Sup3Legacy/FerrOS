use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

pub mod video_mode;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

lazy_static! {
    pub static ref SCREEN: Mutex<Screen> = Mutex::new(Screen {
        col_pos: 0,
        row_pos: 0,
        color: ColorCode::new(Color::LightGreen, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut BUFFER) },
    });
}

/// crate-wide `println` macro.
#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

/// crate-wide `print` macro. It enables any program to write to the VGA interface
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

pub fn write_back() {
    SCREEN.lock().write_byte(b'\r');
}

pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        SCREEN.lock().write_fmt(args).unwrap();
    });
}

pub(crate) fn _print_at(row: usize, col: usize, s: &str) {
    interrupts::without_interrupts(|| {
        SCREEN.lock().write_to_pos(row, col, s);
    });
}

pub fn init() {
    #[allow(unused_must_use)]
    interrupts::without_interrupts(|| {
        SCREEN.lock()._clear();
    });
}

pub fn move_cursor_down() {
    interrupts::without_interrupts(|| {
        SCREEN.lock()._move_cursor_down();
    });
}

pub fn move_cursor_right() {
    interrupts::without_interrupts(|| {
        SCREEN.lock()._move_cursor_right();
    });
}

pub fn move_cursor_up() {
    interrupts::without_interrupts(|| {
        SCREEN.lock()._move_cursor_up();
    });
}

pub fn move_cursor_left() {
    interrupts::without_interrupts(|| {
        SCREEN.lock()._move_cursor_left();
    });
}


/// The 16 colors available in VGA mode
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

/// A ColorCode is the data of a foreground color and a background one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VgaError<'a>(&'a str);

impl ColorCode {
    /// This creates a ColorCode given a foreground color and a background color
    ///
    /// # Arguments
    /// * `foreground` - A color for the foreground
    /// * `background` - A color for the background
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(ColorCode::new(Color::Blue, Color::Black), 1);
    /// ```
    ///
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// This is the base element, stored in the screen buffer.
///
/// # Fields
/// * `code` - ASCII code of the character
/// * `color` - color code of the character, 8-bit integer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct CHAR {
    code: u8,
    color: ColorCode,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct BUFFER {
    characters: [CHAR; BUFFER_WIDTH * BUFFER_HEIGHT],
}

pub struct Screen {
    pub col_pos: usize,
    pub row_pos: usize,
    pub color: ColorCode,
    pub buffer: &'static mut BUFFER,
}

#[allow(dead_code)]
impl Screen {
    fn _move_cursor_up(&mut self){
        if self.row_pos == 0 {
            self.scroll_down();
        }
        else {
            self.row_pos -= 1;
        }
        self.set_cursor();
    }
    
    fn _move_cursor_left(&mut self){
        if self.col_pos == 0 {
            self.col_pos = BUFFER_WIDTH - 1;
            self._move_cursor_up();
        }
        else {
            self.col_pos -= 1;
        }
        self.set_cursor();
    }

    fn _move_cursor_down(&mut self){
        if self.row_pos >= BUFFER_HEIGHT - 1 {
            self.scroll_up();
        } else {
            self.row_pos += 1;
        }
        self.set_cursor();
    }

    fn _move_cursor_right(&mut self){
        if self.col_pos >= BUFFER_WIDTH - 1 {
            self.col_pos = 0;
            self._move_cursor_down();
        }
        else {
            self.col_pos += 1;
        }
        self.set_cursor();
    }
    
    
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.col_pos = 0,
            b'\x08' => {
                self._move_cursor_left();
                self.buffer.characters[self.row_pos * BUFFER_WIDTH + self.col_pos] = CHAR {
                    code: b' ',
                    color: self.color,
                };
            }
            b'\x7F' => {
                self.buffer.characters[self.row_pos * BUFFER_WIDTH + self.col_pos] = CHAR {
                    code: b' ',
                    color: self.color,
                };
            }
            _ => {
//                 if self.col_pos + self.row_pos * BUFFER_WIDTH >= BUFFER_WIDTH * BUFFER_HEIGHT - 1 {
//                     if self.row_pos == 0 {
//                         self.new_line();
//                     }
//                     self.scroll_up();
//                 }
                self.buffer.characters[self.row_pos * BUFFER_WIDTH + self.col_pos] = CHAR {
                    code: byte,
                    color: self.color,
                };
                self._move_cursor_right();
            }
        };
        self.set_cursor();
    }

    fn set_cursor(&mut self) {
        let pos = self.row_pos * BUFFER_WIDTH + self.col_pos;
        let mut port1 = Port::new(0x3D4);
        let mut port2 = Port::new(0x3D5);
        unsafe {
            port1.write(0x0F as u8);
            port2.write((pos & 0xFF) as u8);
            port1.write(0x0E as u8);
            port2.write(((pos >> 8) & 0xFF) as u8)
        }
    }
    /// This functions positions the character pointer to the following line.
    /// If the screens overflows, it get scrolled up.
    fn new_line(&mut self) -> () {
        self.row_pos += 1 + (self.col_pos / BUFFER_WIDTH);
        self.col_pos = 0;
        while self.row_pos >= BUFFER_HEIGHT {
            self.scroll_up();
            self.clear_row(BUFFER_HEIGHT-1);
            self.row_pos = BUFFER_HEIGHT - 1;
        }
    }
    /// This function scrolls the entire screen by one row upwards.
    fn scroll_up(&mut self) -> () {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.characters[(row - 1) * BUFFER_WIDTH + col] =
                    self.buffer.characters[row * BUFFER_WIDTH + col];
            }
        };
        self.clear_row(BUFFER_HEIGHT - 1);
    }
            
    fn scroll_down(&mut self) -> () {
        for row in (0..BUFFER_HEIGHT-1).rev() {
            for col in 0..BUFFER_WIDTH {
                self.buffer.characters[(row + 1) * BUFFER_WIDTH + col] =
                    self.buffer.characters[row * BUFFER_WIDTH + col];
            }
        }
        self.clear_row(0);
    }

        
    fn clear_row(&mut self, row: usize) -> () {
        let blank = CHAR {
            code: b' ',
            color: self.color,
        };
        // perhaps a way to directly fidle in memory to clean all at once ?
        for col in 0..BUFFER_WIDTH {
            self.buffer.characters[row * BUFFER_WIDTH + col] = blank;
        }
    }
    
    
    pub fn write_string(&mut self, s: &str) {
        for byte in s.chars() {
            match byte as u8 {
                0x20..=0x7e | b'\n' | b'\r' => self.write_byte(byte as u8),
                _ => self.write_byte(byte as u8),
            }
        }
    }

    pub fn _write_string_color(&mut self, s: &str, col: ColorCode) -> () {
        let old_color = self.color;
        self.set_color(col);
        self.write_string(s);
        self.set_color(old_color);
        println!("s = {}", s.bytes().len());
    }
    fn _new(color: ColorCode, buffer: &'static mut BUFFER) -> Self {
        Screen {
            col_pos: 0,
            row_pos: 0,
            color,
            buffer,
        }
    }
    pub fn set_color(&mut self, color: ColorCode) -> () {
        self.color = color;
    }
    pub fn _clear(&mut self) -> Result<(), VgaError<'_>> {
        let blank = CHAR {
            code: b' ',
            color: self.color,
        };
        // perhaps a way to fidle in memory to clean all at once ?
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.characters[row * BUFFER_WIDTH + col] = blank;
            }
        }
        self.col_pos = 0;
        self.row_pos = 0;
        Ok(())
    }
    pub fn write_to_pos(&mut self, row: usize, col: usize, s: &str) {
        let old_row = self.row_pos;
        let old_col = self.col_pos;
        if row >= BUFFER_HEIGHT {
            println!("Row out of bounds");
            return;
        }
        if col >= BUFFER_WIDTH {
            println!("Col out of bounds");
            return;
        }
        self.row_pos = row;
        self.col_pos = col;
        self.write_string(s);
        self.row_pos = old_row;
        self.col_pos = old_col;
    }
}

impl fmt::Write for Screen {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[test_case]
fn print_without_panic() {
    println!("abcdefghijklmnopqrstuvwwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ!@#$%^&*()_+-=~`|?/>.<,:;\"'}}{{[]\\");
    println!("éèàáöçµ®¢ŒÆÆæœ©®ßÁ§ÐðÏïŒœøØ¶°¦¬”»“«ÖöÓóÍíÚúÜüÞþËëÉåÅäÄ¡¹²³£¤€¸¼½¾˘’‘¥̣÷×");
    print!("42");
}

#[test_case]
fn check_print_output() {
    let s = "Yolo";

    let row = SCREEN.lock().row_pos;
    let col = SCREEN.lock().col_pos;
    let pos = row * BUFFER_WIDTH + col;

    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = SCREEN.lock().buffer.characters[pos + i];
        assert_eq!(char::from(screen_char.code), c);
    }
}
