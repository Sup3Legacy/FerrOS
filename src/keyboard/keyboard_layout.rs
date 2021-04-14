//use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyboardLayout, KeyCode, Modifiers};

//pub struct Fr104Key;

//use crate::{print, println};

#[allow(dead_code)]
pub struct KeyBoardStatus {
    maj: bool,
    shift_l: bool,
    shift_r: bool,
    num_lock: bool,
    control: bool,
    alt: bool,
    alt_gr: bool,
    fn_key: bool,
    table_status: [bool; 128],
    layout: Layout,
}

#[derive(Copy, Clone)]
pub enum KeyEvent {
    Character(char),
    SpecialKey(u8),
}

#[derive(Copy, Clone)]
pub enum Effect {
    Nothing,
    Value(KeyEvent),
}

#[derive(Copy, Clone)]
pub enum Layout {
    Fr = 0,
    Us = 1,
}

#[allow(dead_code)]
impl KeyBoardStatus {
    pub fn new() -> Self {
        KeyBoardStatus {
            maj: false,
            shift_l: false,
            shift_r: false,
            num_lock: false,
            control: false,
            alt: false,
            alt_gr: false,
            fn_key: false,
            table_status: [false; 128],
            layout: Layout::Fr,
        }
    }

    pub fn shift_l_down(&mut self) {
        self.shift_l = true;
    }
    pub fn shift_l_up(&mut self) {
        self.shift_l = false;
    }
    pub fn shift_r_down(&mut self) {
        self.shift_r = true;
    }
    pub fn shift_r_up(&mut self) {
        self.shift_r = false;
    }
    pub fn alt_down(&mut self) {
        self.alt = true;
    }
    pub fn alt_up(&mut self) {
        self.alt = false;
    }
    pub fn maj_s(&mut self) {
        self.maj = !self.maj;
    }
    pub fn num_lock(&mut self) {
        self.num_lock = !self.num_lock;
    }
    pub fn fn_up(&mut self) {
        self.fn_key = false;
    }
    pub fn fn_down(&mut self) {
        self.fn_key = true;
    }
    pub fn control_s(&mut self) {
        self.control = !self.control;
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
    }
    pub fn get_layout(&self) -> Layout {
        self.layout
    }
    pub fn change_layout(&mut self) {
        let current_layout = self.get_layout();
        let new_layout = match current_layout {
            Layout::Fr => Layout::Us,
            Layout::Us => Layout::Fr,
        };
        self.set_layout(new_layout);
    }

    pub fn maj(&self) -> bool {
        self.maj || self.shift_l || self.shift_r
    }
    pub fn num(&self) -> bool {
        self.num_lock || self.shift_l || self.shift_r || self.maj
    }
    pub fn function(&self) -> bool {
        self.fn_key
    }
    pub fn shift(&self) -> bool {
        self.shift_r || self.shift_l
    }
    pub fn alt(&self) -> bool {
        self.alt
    }

    pub fn process(&mut self, key: u8) -> Effect {
        if key > 127_u8 {
            self.table_status[(key - 128) as usize] = false;
            match key - 128 {
                54 => self.shift_r_up(),
                42 => self.shift_l_up(),
                56 => self.alt_up(),
                _ => (),
            };
            Effect::Nothing
        } else {
            self.table_status[key as usize] = true;

            let lower_case_layout = match self.layout {
                Layout::Fr => LOWER_CASE_FR_LAYOUT,
                Layout::Us => LOWER_CASE_US_LAYOUT,
            };
            let upper_case_layout = match self.layout {
                Layout::Fr => UPPER_CASE_FR_LAYOUT,
                Layout::Us => UPPER_CASE_US_LAYOUT,
            };
            let num_layout = match self.layout {
                Layout::Fr => NUM_FR_LAYOUT,
                Layout::Us => NUM_US_LAYOUT,
            };
            let alt_layout = match self.layout {
                Layout::Fr => ALT_FR_LAYOUT,
                Layout::Us => ALT_US_LAYOUT,
            };

            if self.num() {
                num_layout[key as usize]
            } else if self.alt {
                alt_layout[key as usize]
            } else if self.maj() {
                upper_case_layout[key as usize]
            } else {
                lower_case_layout[key as usize]
            }
        }
    }
}

macro_rules! layout {
    ( ; $( $k:literal $c:literal ),* ; $( $special:tt )* ) => {
        {
            let mut l: [Effect; 128] = [Effect::Nothing; 128];
            layout!(l ; $( $k $c ),* ; $( $special )*)
        }
    };
    ( $l:expr ; $( $k:literal $c:literal ),* ; $( $special:tt )* ) => {
        {
            $(
                #[allow(unused_assigments)]
                {
                    $l[$k] = Effect::Value(KeyEvent::Character($c));
                }
            )*
            layout!($l ; $( $special )*)
        }
    };
    ( $l:expr ; $( $k:literal $sk:literal ),* ) => {
        {
            $(
                #[allow(unused_assigments)]
                {
                    $l[$k] = Effect::Value(KeyEvent::SpecialKey($sk));
                }
            )*
            $l
        }
    };
}

// FR Layout

const LOWER_CASE_FR_LAYOUT: [Effect; 128] = layout!(
    ; 2 '&', 3 'é', 4 '"', 5 '\'', 6 '(', 7 '-', 8 'è', 9 '_', 10 'ç', 11 'à', 12 ')', 13 '=',
      16 'a', 17 'z', 18 'e', 19 'r', 20 't', 21 'y', 22 'u', 23 'i', 24 'o', 25 'p', 26 '^', 27 '$', 28 '\n',
      30 'q', 31 's', 32 'd', 33 'f', 34 'g', 35 'h', 36 'j', 37 'k', 38 'l', 39 'm', 40 '%',
      44 'w', 45 'x', 46 'c', 47 'v', 48 'b', 49 'n', 50 ',', 51 ';', 52 '/', 53 '!',
      57 ' '
    ; 14 0, 15 1);

const UPPER_CASE_FR_LAYOUT: [Effect; 128] = layout!(
    ; 2 '1', 3 '2', 4 '3', 5 '4', 6 '5', 7 '6', 8 '7', 9 '8', 10 '9', 11 '0', 12 '°', 13 '+',
      16 'A', 17 'Z', 18 'E', 19 'R', 20 'T', 21 'Y', 22 'U', 23 'I', 24 'O', 25 'P', 26 '¨', 27 '£', 28 '\n',
      30 'Q', 31 'S', 32 'D', 33 'F', 34 'G', 35 'H', 36 'J', 37 'K', 38 'L', 39 'M', 40 '%',
      44 'W', 45 'X', 46 'C', 47 'V', 48 'B', 49 'N', 50 '?', 51 '.', 52 ':', 53 '§',
      57 ' '
    ; 14 0, 15 1);

// TODO
const ALT_FR_LAYOUT: [Effect; 128] = layout!(
    ; 2 '&', 3 'é', 4 '"', 5 '\'', 6 '(', 7 '-', 8 'è', 9 '_', 10 'ç', 11 'à', 12 ')', 13 '=',
      16 'a', 17 'z', 18 'e', 19 'r', 20 't', 21 'y', 22 'u', 23 'i', 24 'o', 25 'p', 26 '^', 27 '$', 28 '\n',
      30 'q', 31 's', 32 'd', 33 'f', 34 'g', 35 'h', 36 'j', 37 'k', 38 'l', 39 'm', 40 '%',
      44 'w', 45 'x', 46 'c', 47 'v', 48 'b', 49 'n', 50 '?', 51 '.', 52 '/', 53 '=',
      57 ' '
    ; 14 0, 15 1);

// TODO
const NUM_FR_LAYOUT: [Effect; 128] = layout!(
    ; 2 '&', 3 'é', 4 '"', 5 '\'', 6 '(', 7 '-', 8 'è', 9 '_', 10 'ç', 11 'à', 12 ')', 13 '=',
      16 'a', 17 'z', 18 'e', 19 'r', 20 't', 21 'y', 22 'u', 23 'i', 24 'o', 25 'p', 26 '^', 27 '$', 28 '\n',
      30 'q', 31 's', 32 'd', 33 'f', 34 'g', 35 'h', 36 'j', 37 'k', 38 'l', 39 'm', 40 '%',
      44 'w', 45 'x', 46 'c', 47 'v', 48 'b', 49 'n', 50 '?', 51 '.', 52 '/', 53 '=',
      57 ' '
    ; 14 0, 15 1);

// US Layout

// TODO
const LOWER_CASE_US_LAYOUT: [Effect; 128] = layout!(
    ; 2 '&', 3 'é', 4 '"', 5 '\'', 6 '(', 7 '-', 8 'è', 9 '_', 10 'ç', 11 'à', 12 ')', 13 '=',
      16 'a', 17 'z', 18 'e', 19 'r', 20 't', 21 'y', 22 'u', 23 'i', 24 'o', 25 'p', 26 '^', 27 '$', 28 '\n',
      30 'q', 31 's', 32 'd', 33 'f', 34 'g', 35 'h', 36 'j', 37 'k', 38 'l', 39 'm', 40 '%',
      44 'w', 45 'x', 46 'c', 47 'v', 48 'b', 49 'n', 50 '?', 51 '.', 52 '/', 53 '=',
      57 ' '
    ; 14 0, 15 1);

// TODO
const UPPER_CASE_US_LAYOUT: [Effect; 128] = layout!(
    ; 2 '&', 3 'é', 4 '"', 5 '\'', 6 '(', 7 '-', 8 'è', 9 '_', 10 'ç', 11 'à', 12 ')', 13 '=',
      16 'a', 17 'z', 18 'e', 19 'r', 20 't', 21 'y', 22 'u', 23 'i', 24 'o', 25 'p', 26 '^', 27 '$', 28 '\n',
      30 'q', 31 's', 32 'd', 33 'f', 34 'g', 35 'h', 36 'j', 37 'k', 38 'l', 39 'm', 40 '%',
      44 'w', 45 'x', 46 'c', 47 'v', 48 'b', 49 'n', 50 '?', 51 '.', 52 '/', 53 '=',
      57 ' '
    ; 14 0, 15 1);

// TODO
const ALT_US_LAYOUT: [Effect; 128] = layout!(
    ; 2 '&', 3 'é', 4 '"', 5 '\'', 6 '(', 7 '-', 8 'è', 9 '_', 10 'ç', 11 'à', 12 ')', 13 '=',
      16 'a', 17 'z', 18 'e', 19 'r', 20 't', 21 'y', 22 'u', 23 'i', 24 'o', 25 'p', 26 '^', 27 '$', 28 '\n',
      30 'q', 31 's', 32 'd', 33 'f', 34 'g', 35 'h', 36 'j', 37 'k', 38 'l', 39 'm', 40 '%',
      44 'w', 45 'x', 46 'c', 47 'v', 48 'b', 49 'n', 50 '?', 51 '.', 52 '/', 53 '=',
      57 ' '
    ; 14 0, 15 1);

// TODO
const NUM_US_LAYOUT: [Effect; 128] = layout!(
    ; 2 '&', 3 'é', 4 '"', 5 '\'', 6 '(', 7 '-', 8 'è', 9 '_', 10 'ç', 11 'à', 12 ')', 13 '=',
      16 'a', 17 'z', 18 'e', 19 'r', 20 't', 21 'y', 22 'u', 23 'i', 24 'o', 25 'p', 26 '^', 27 '$', 28 '\n',
      30 'q', 31 's', 32 'd', 33 'f', 34 'g', 35 'h', 36 'j', 37 'k', 38 'l', 39 'm', 40 '%',
      44 'w', 45 'x', 46 'c', 47 'v', 48 'b', 49 'n', 50 '?', 51 '.', 52 '/', 53 '=',
      57 ' '
    ; 14 0, 15 1);

/*
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Key1 = 2,
    Key2 = 3,
    Key3 = 4,
    Key4 = 5,
    Key5 = 6,
    Key6 = 7,
    Key7 = 8,
    Key8 = 9,
    Key9 = 10,
    Key0 = 11,
    UpZero = 12,
    Min = 13,
    BackSpace = 14,

    Tab = 15,
    Let0_0 = 16,
    Let0_1 = 17,
    Let0_2 = 18,
    Let0_3 = 19,
    Let0_4 = 20,
    Let0_5 = 21,
    Let0_6 = 22,
    Let0_7 = 23,
    Let0_8 = 24,
    Let0_9 = 25,
    Accent = 26,
    Dolar = 27,
    Enter = 28,

    Maj = 58,
    Let1_0 = 30,
    Let1_1 = 31,
    Let1_2 = 32,
    Let1_3 = 33,
    Let1_4 = 34,
    Let1_5 = 35,
    Let1_6 = 36,
    Let1_7 = 37,
    Let1_8 = 38,
    Let1_9 = 39,
    Pourcent = 40,
    Tild = 43,

    ShiftL = 42,
    Ineg = 41,
    Let2_0 = 44,
    Let2_1 = 45,
    Let2_2 = 46,
    Let2_3 = 47,
    Let2_4 = 48,
    Let2_5 = 49,
    Let2_6 = 50,
    Dot = 51,
    Slash = 52,
    Equal = 53,
    ShiftR = 54,

    Alt = 56,

    Space = 57,

    ArrowU = 75,
    ArrowD = 77,
    ArrowL = 72,
    ArrowR = 80,
    Unknown = 0,
}

impl KeyboardLayout for Fr104Key {
    pub fn map_keycode(keycode: KeyCode, modifiers: &Modifiers) -> DecodedKey {
        match keycode {
            KeyCode::BackTick => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('²')
                } else {
                    DecodedKey::Unicode('²')
                }
            }
            KeyCode::Escape => DecodedKey::Unicode(0x1B.into()),
            KeyCode::Key1 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('1')
                } else {
                    DecodedKey::Unicode('&')
                }
            }
            KeyCode::Key2 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('2')
                } else {
                    DecodedKey::Unicode('é')
                }
            }
            KeyCode::Key3 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('3')
                } else {
                    DecodedKey::Unicode('"')
                }
            }
            KeyCode::Key4 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('4')
                } else {
                    DecodedKey::Unicode('\'')
                }
            }
            KeyCode::Key5 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('5')
                } else {
                    DecodedKey::Unicode('(')
                }
            }
            KeyCode::Key6 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('6')
                } else {
                    DecodedKey::Unicode('-')
                }
            }
            KeyCode::Key7 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('7')
                } else {
                    DecodedKey::Unicode('è')
                }
            }
            KeyCode::Key8 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('8')
                } else {
                    DecodedKey::Unicode('_')
                }
            }
            KeyCode::Key9 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('9')
                } else {
                    DecodedKey::Unicode('ç')
                }
            }
            KeyCode::Key0 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('0')
                } else {
                    DecodedKey::Unicode('à')
                }
            }
            KeyCode::Minus => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('°')
                } else {
                    DecodedKey::Unicode(')')
                }
            }
            KeyCode::Equals => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('+')
                } else {
                    DecodedKey::Unicode('=')
                }
            }
            KeyCode::Backspace => DecodedKey::Unicode(0x08.into()),
            KeyCode::Tab => DecodedKey::Unicode(0x09.into()),
            KeyCode::Q => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('A')
                } else {
                    DecodedKey::Unicode('a')
                }
            }
            KeyCode::W => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('Z')
                } else {
                    DecodedKey::Unicode('z')
                }
            }
            KeyCode::E => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('E')
                } else {
                    DecodedKey::Unicode('e')
                }
            }
            KeyCode::R => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('R')
                } else {
                    DecodedKey::Unicode('r')
                }
            }
            KeyCode::T => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('T')
                } else {
                    DecodedKey::Unicode('t')
                }
            }
            KeyCode::Y => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('Y')
                } else {
                    DecodedKey::Unicode('y')
                }
            }
            KeyCode::U => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('U')
                } else {
                    DecodedKey::Unicode('u')
                }
            }
            KeyCode::I => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('I')
                } else {
                    DecodedKey::Unicode('i')
                }
            }
            KeyCode::O => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('O')
                } else {
                    DecodedKey::Unicode('o')
                }
            }
            KeyCode::P => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('P')
                } else {
                    DecodedKey::Unicode('p')
                }
            }
            KeyCode::BracketSquareLeft => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('\"')
                } else {
                    DecodedKey::Unicode('^')
                }
            }
            KeyCode::BracketSquareRight => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('£')
                } else {
                    DecodedKey::Unicode('$')
                }
            }
            KeyCode::BackSlash => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('|')
                } else {
                    DecodedKey::Unicode('\\')
                }
            }
            KeyCode::A => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('Q')
                } else {
                    DecodedKey::Unicode('q')
                }
            }
            KeyCode::S => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('S')
                } else {
                    DecodedKey::Unicode('s')
                }
            }
            KeyCode::D => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('D')
                } else {
                    DecodedKey::Unicode('d')
                }
            }
            KeyCode::F => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('F')
                } else {
                    DecodedKey::Unicode('f')
                }
            }
            KeyCode::G => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('G')
                } else {
                    DecodedKey::Unicode('g')
                }
            }
            KeyCode::H => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('H')
                } else {
                    DecodedKey::Unicode('h')
                }
            }
            KeyCode::J => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('J')
                } else {
                    DecodedKey::Unicode('j')
                }
            }
            KeyCode::K => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('K')
                } else {
                    DecodedKey::Unicode('k')
                }
            }
            KeyCode::L => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('L')
                } else {
                    DecodedKey::Unicode('l')
                }
            }
            KeyCode::SemiColon => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('M')
                } else {
                    DecodedKey::Unicode('m')
                }
            }
            KeyCode::Quote => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('%')
                } else {
                    DecodedKey::Unicode('ù')
                }
            }
            // Enter gives LF, not CRLF or CR
            KeyCode::Enter => DecodedKey::Unicode(10.into()),
            KeyCode::Z => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('W')
                } else {
                    DecodedKey::Unicode('w')
                }
            }
            KeyCode::X => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('X')
                } else {
                    DecodedKey::Unicode('x')
                }
            }
            KeyCode::C => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('C')
                } else {
                    DecodedKey::Unicode('c')
                }
            }
            KeyCode::V => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('V')
                } else {
                    DecodedKey::Unicode('v')
                }
            }
            KeyCode::B => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('B')
                } else {
                    DecodedKey::Unicode('b')
                }
            }
            KeyCode::N => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('N')
                } else {
                    DecodedKey::Unicode('n')
                }
            }
            KeyCode::M => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('?')
                } else {
                    DecodedKey::Unicode(',')
                }
            }
            KeyCode::Comma => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('.')
                } else {
                    DecodedKey::Unicode(';')
                }
            }
            KeyCode::Fullstop => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('/')
                } else {
                    DecodedKey::Unicode('!')
                }
            }
            KeyCode::Slash => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('§')
                } else {
                    DecodedKey::Unicode('!')
                }
            }
            KeyCode::Spacebar => DecodedKey::Unicode(' '),
            KeyCode::DeLet0_2 => DecodedKey::Unicode(127.into()),
            KeyCode::NumpadSlash => DecodedKey::Unicode('/'),
            KeyCode::NumpadStar => DecodedKey::Unicode('*'),
            KeyCode::NumpadMinus => DecodedKey::Unicode('-'),
            KeyCode::Numpad7 => {
                if modifiers.numlock {
                    DecodedKey::Unicode('7')
                } else {
                    DecodedKey::RawKey(KeyCode::Home)
                }
            }
            KeyCode::Numpad8 => {
                if modifiers.numlock {
                    DecodedKey::Unicode('8')
                } else {
                    DecodedKey::RawKey(KeyCode::ArrowUp)
                }
            }
            KeyCode::Numpad9 => {
                if modifiers.numlock {
                    DecodedKey::Unicode('9')
                } else {
                    DecodedKey::RawKey(KeyCode::PageUp)
                }
            }
            KeyCode::NumpadPlus => DecodedKey::Unicode('+'),
            KeyCode::Numpad4 => {
                if modifiers.numlock {
                    DecodedKey::Unicode('4')
                } else {
                    DecodedKey::RawKey(KeyCode::ArrowLeft)
                }
            }
            KeyCode::Numpad5 => DecodedKey::Unicode('5'),
            KeyCode::Numpad6 => {
                if modifiers.numlock {
                    DecodedKey::Unicode('6')
                } else {
                    DecodedKey::RawKey(KeyCode::ArrowRight)
                }
            }
            KeyCode::Numpad1 => {
                if modifiers.numlock {
                    DecodedKey::Unicode('1')
                } else {
                    DecodedKey::RawKey(KeyCode::End)
                }
            }
            KeyCode::Numpad2 => {
                if modifiers.numlock {
                    DecodedKey::Unicode('2')
                } else {
                    DecodedKey::RawKey(KeyCode::ArrowDown)
                }
            }
            KeyCode::Numpad3 => {
                if modifiers.numlock {
                    DecodedKey::Unicode('3')
                } else {
                    DecodedKey::RawKey(KeyCode::PageDown)
                }
            }
            KeyCode::Numpad0 => {
                if modifiers.numlock {
                    DecodedKey::Unicode('0')
                } else {
                    DecodedKey::RawKey(KeyCode::Insert)
                }
            }
            KeyCode::NumpadPeriod => {
                if modifiers.numlock {
                    DecodedKey::Unicode('.')
                } else {
                    DecodedKey::Unicode(127.into())
                }
            }
            KeyCode::NumpadEnter => DecodedKey::Unicode(10.into()),
            k => DecodedKey::RawKey(k),
        }
    }
}
*/
