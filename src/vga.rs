use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;


lazy_static! { 
    pub static ref SCREEN : Mutex<Screen> = Mutex::new(Screen{
        col_pos : 0,
        row_pos : 0,
        color : ColorCode::new(Color::LightGreen, Color::Black),
        buffer : unsafe { &mut *(0xb8000 as *mut BUFFER) },
    });
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

pub fn write_back() {
    SCREEN.lock().write_byte(b'\r');
}

pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {SCREEN.lock().write_fmt(args).unwrap();});
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
    code : u8,
    color : ColorCode
}

#[derive(Debug)]
#[repr(transparent)]
pub struct BUFFER {
    characters : [[CHAR; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Screen {
    pub col_pos : usize,
    pub row_pos : usize,
    pub color : ColorCode,
    pub buffer : &'static mut BUFFER
}



impl Screen {
    pub fn write_byte(&mut self, byte : u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.col_pos = 0,
            _ => {
                    if self.col_pos >= BUFFER_WIDTH {
                        self.new_line();
                    }
                    self.buffer.characters[self.row_pos][self.col_pos] = CHAR {code : byte, color : self.color};
                    self.col_pos += 1;
            }
        };
    }
    /// This functions positions the character pointer to the following line.
    /// If the screens overflows, it get scrolled up.
    fn new_line(&mut self) -> () {
        self.col_pos = 0;
        self.row_pos += 1;
        if self.row_pos >= BUFFER_HEIGHT {
            self.scroll_up();
            self.clear_bottom();
            self.row_pos = BUFFER_HEIGHT - 1;
        }
    }
    /// This function scrolls the entire screen by one row upwards.
    fn scroll_up(&mut self) -> () {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.characters[row - 1][col] = self.buffer.characters[row][col];
            }
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
    pub fn _write_string_color(&mut self, s : &str, col : ColorCode) -> () {
        let old_color = self.color;
        self.set_color(col);
        self.write_string(s);
        self.set_color(old_color);
    }
    fn _new(color : ColorCode, buffer : &'static mut BUFFER) -> Self {
        Screen {col_pos : 0, row_pos : 0, color : color, buffer : buffer}
    }
    pub fn set_color(&mut self, color : ColorCode) -> () {
        self.color = color;
    }
    pub fn _clear(&mut self) -> Result<(), VgaError<'_>> {
        let blank = CHAR {
            code : b' ',
            color : self.color
        };
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.characters[row][col] = blank;
            }
        }
        self.col_pos = 0;
        self.row_pos = 0;
        Ok(())
    }
}

impl fmt::Write for Screen {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}