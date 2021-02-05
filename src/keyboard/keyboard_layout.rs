//use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyboardLayout, KeyCode, Modifiers};

//pub struct Fr104Key;

#[allow(dead_code)]
pub struct KeyBoardStatus {
    maj : bool,
    shift_l : bool,
    shift_r : bool,
    num_lock : bool,
    control : bool,
    alt : bool,
    fn_key : bool,
    table_status : [bool; 128]

}

pub enum Effect {
    Nothing,
    Value(KeyEvent)
}

pub enum KeyEvent {
    Character(char),
    SpecialKey(u8)
}

#[allow(dead_code)]
impl KeyBoardStatus {
    pub fn new() -> Self {
        KeyBoardStatus {
            maj : false,
            shift_l : false,
            shift_r : false,
            num_lock : false,
            control : false,
            alt : false,
            fn_key : false,
            table_status : [false; 128]
        }
    }

    pub fn shift_l_down(&mut self) { self.shift_l = true }
    pub fn shift_l_up(&mut self) { self.shift_l = false}
    pub fn shift_r_down(&mut self) { self.shift_r = true }
    pub fn shift_r_up(&mut self) { self.shift_r = false}
    pub fn alt_down(&mut self) {    self.alt = true     }
    pub fn alt_up(&mut self) {    self.alt = false    }
    pub fn maj_s(&mut self) {    self.maj = !self.maj}
    pub fn num_lock(&mut self) { self.num_lock = !self.num_lock  }
    pub fn fn_up(&mut self) {  self.fn_key = false }
    pub fn fn_down(&mut self) {  self.fn_key = true  }
    pub fn control_s(&mut self) {self.control = !self.control}

    pub fn maj(&self) -> bool { self.maj || self.shift_l || self.shift_r    }
    pub fn num(&self) -> bool { self.num_lock || self.shift_l || self.shift_r   }
    pub fn function(&self) -> bool {  self.fn_key }
    pub fn shift(&self) -> bool {   self.shift_r || self.shift_l    }
    pub fn alt(&self) -> bool { self.alt    }

    pub fn process(&mut self, key : u8) -> Effect {
        if key > (127 as u8) {
            self.table_status[(key-128) as usize] = false;
            match convert(key - 128) {
                Key::ShiftR => self.shift_r_up(),
                Key::ShiftL => self.shift_l_up(),
                _ => ()
            };
            Effect::Nothing
        } else if !self.table_status[key as usize] {
            self.table_status[key as usize] = true;
            match convert(key) {
                Key::Key1 => {
                    if self.num() {Effect::Value(KeyEvent::Character('1'))
                    } else {
                        Effect::Value(KeyEvent::Character('&'))
                    }
                },

                Key::Key2 => {
                    if self.num() {Effect::Value(KeyEvent::Character('2'))
                    } else {
                        Effect::Value(KeyEvent::Character('é'))
                    }
                },

                Key::Key3 => {
                    if self.num() {Effect::Value(KeyEvent::Character('3'))
                    } else {
                        Effect::Value(KeyEvent::Character('"'))
                    }
                },
                
                Key::Key4 => {
                    if self.num() {Effect::Value(KeyEvent::Character('4'))
                    } else {
                        Effect::Value(KeyEvent::Character('\''))
                    }
                },

                Key::Key5 => {
                    if self.num() {Effect::Value(KeyEvent::Character('5'))
                    } else {
                        Effect::Value(KeyEvent::Character('('))
                    }
                },

                Key::Key6 => {
                    if self.num() {Effect::Value(KeyEvent::Character('6'))
                    } else {
                        Effect::Value(KeyEvent::Character('§'))
                    }
                },

                Key::Key7 => {
                    if self.num() {Effect::Value(KeyEvent::Character('7'))
                    } else {
                        Effect::Value(KeyEvent::Character('è'))
                    }
                },

                Key::Key8 => {
                    if self.num() {Effect::Value(KeyEvent::Character('8'))
                    } else {
                        Effect::Value(KeyEvent::Character('!'))
                    }
                },

                Key::Key9 => {
                    if self.num() {Effect::Value(KeyEvent::Character('9'))
                    } else {
                        Effect::Value(KeyEvent::Character('ç'))
                    }
                },

                Key::Key0 => {
                    if self.num() {Effect::Value(KeyEvent::Character('0'))
                    } else {
                        Effect::Value(KeyEvent::Character('à'))
                    }
                },

                Key::LetA => {
                    if self.maj() {Effect::Value(KeyEvent::Character('A'))
                    } else {
                        Effect::Value(KeyEvent::Character('a'))
                    }
                },
                Key::LetB => {
                    if self.maj() {Effect::Value(KeyEvent::Character('B'))
                    } else {
                        Effect::Value(KeyEvent::Character('b'))
                    }
                },
                Key::LetC => {
                    if self.maj() {Effect::Value(KeyEvent::Character('C'))
                    } else {
                        Effect::Value(KeyEvent::Character('c'))
                    }
                },
                Key::LetD => {
                    if self.maj() {Effect::Value(KeyEvent::Character('D'))
                    } else {
                        Effect::Value(KeyEvent::Character('d'))
                    }
                },
                Key::LetE => {
                    if self.maj() {Effect::Value(KeyEvent::Character('E'))
                    } else {
                        Effect::Value(KeyEvent::Character('e'))
                    }
                },
                Key::LetF => {
                    if self.maj() {Effect::Value(KeyEvent::Character('F'))
                    } else {
                        Effect::Value(KeyEvent::Character('f'))
                    }
                },
                Key::LetG => {
                    if self.maj() {Effect::Value(KeyEvent::Character('G'))
                    } else {
                        Effect::Value(KeyEvent::Character('g'))
                    }
                },
                Key::LetH => {
                    if self.maj() {Effect::Value(KeyEvent::Character('H'))
                    } else {
                        Effect::Value(KeyEvent::Character('h'))
                    }
                },
                Key::LetI => {
                    if self.maj() {Effect::Value(KeyEvent::Character('I'))
                    } else {
                        Effect::Value(KeyEvent::Character('i'))
                    }
                },
                Key::LetJ => {
                    if self.maj() {Effect::Value(KeyEvent::Character('J'))
                    } else {
                        Effect::Value(KeyEvent::Character('j'))
                    }
                },
                Key::LetK => {
                    if self.maj() {Effect::Value(KeyEvent::Character('K'))
                    } else {
                        Effect::Value(KeyEvent::Character('k'))
                    }
                },
                Key::LetL => {
                    if self.maj() {Effect::Value(KeyEvent::Character('L'))
                    } else {
                        Effect::Value(KeyEvent::Character('l'))
                    }
                },
                Key::LetM => {
                    if self.maj() {Effect::Value(KeyEvent::Character('M'))
                    } else {
                        Effect::Value(KeyEvent::Character('m'))
                    }
                },
                Key::LetN => {
                    if self.maj() {Effect::Value(KeyEvent::Character('N'))
                    } else {
                        Effect::Value(KeyEvent::Character('n'))
                    }
                },
                Key::LetO => {
                    if self.maj() {Effect::Value(KeyEvent::Character('O'))
                    } else {
                        Effect::Value(KeyEvent::Character('o'))
                    }
                },
                Key::LetP => {
                    if self.maj() {Effect::Value(KeyEvent::Character('P'))
                    } else {
                        Effect::Value(KeyEvent::Character('p'))
                    }
                },
                Key::LetQ => {
                    if self.maj() {Effect::Value(KeyEvent::Character('Q'))
                    } else {
                        Effect::Value(KeyEvent::Character('q'))
                    }
                },
                Key::LetR => {
                    if self.maj() {Effect::Value(KeyEvent::Character('R'))
                    } else {
                        Effect::Value(KeyEvent::Character('r'))
                    }
                },
                Key::LetS => {
                    if self.maj() {Effect::Value(KeyEvent::Character('S'))
                    } else {
                        Effect::Value(KeyEvent::Character('s'))
                    }
                },
                Key::LetT => {
                    if self.maj() {Effect::Value(KeyEvent::Character('T'))
                    } else {
                        Effect::Value(KeyEvent::Character('t'))
                    }
                },
                Key::LetU => {
                    if self.maj() {Effect::Value(KeyEvent::Character('U'))
                    } else {
                        Effect::Value(KeyEvent::Character('u'))
                    }
                },
                Key::LetV => {
                    if self.maj() {Effect::Value(KeyEvent::Character('V'))
                    } else {
                        Effect::Value(KeyEvent::Character('v'))
                    }
                },
                Key::LetW => {
                    if self.maj() {Effect::Value(KeyEvent::Character('W'))
                    } else {
                        Effect::Value(KeyEvent::Character('w'))
                    }
                },
                Key::LetX => {
                    if self.maj() {Effect::Value(KeyEvent::Character('X'))
                    } else {
                        Effect::Value(KeyEvent::Character('x'))
                    }
                },
                Key::LetY => {
                    if self.maj() {Effect::Value(KeyEvent::Character('Y'))
                    } else {
                        Effect::Value(KeyEvent::Character('y'))
                    }
                },
                Key::LetZ => {
                    if self.maj() {Effect::Value(KeyEvent::Character('Z'))
                    } else {
                        Effect::Value(KeyEvent::Character('z'))
                    }
                },

                Key::ShiftR => {
                    self.shift_r_down();
                    Effect::Nothing
                },

                Key::ShiftL => {
                    self.shift_l_down();
                    Effect::Nothing
                },

                Key::Maj => {
                    self.maj_s();
                    Effect::Nothing
                },

                Key::Enter => {
                    Effect::Value(KeyEvent::Character('\n'))
                }

                Key::BackSpace => {
                    Effect::Value(KeyEvent::SpecialKey(0))
                }
                _ => Effect::Nothing
            }
        } else {
            Effect::Nothing
        }

    }

}

fn convert(key : u8) -> Key {
    if key < 128 {
        TABLE_CODE[key as usize]
    } else {
        panic!("Should not occur Keyboard_Layout")
    }
}





static TABLE_CODE : [Key; 128] = [
Key::Unknown, Key::Unknown, Key::Key1, Key::Key2, Key::Key3, Key::Key4, Key::Key5, Key::Key6, Key::Key7, Key::Key8,
Key::Key9, Key::Key0, Key::UpZero, Key::Min, Key::BackSpace, Key::Tab, Key::LetA, Key::LetZ, Key::LetE, Key::LetR,
Key::LetT, Key::LetY, Key::LetU, Key::LetI, Key::LetO, Key::LetP, Key::Accent, Key::Dolar, Key::Enter, Key::Unknown,
Key::LetQ, Key::LetS, Key::LetD, Key::LetF, Key::LetG, Key::LetH, Key::LetJ, Key::LetK, Key::LetL, Key::LetM,
Key::Pourcent, Key::Ineg, Key::ShiftL, Key::Tild, Key::LetW, Key::LetX, Key::LetC, Key::LetV, Key::LetB, Key::LetN,
Key::Comma, Key::Dot, Key::Slash, Key::Equal, Key::ShiftR, Key::Unknown, Key::Unknown, Key::Space, Key::Maj, Key::Unknown,
Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown,
Key::Unknown, Key::ArrowL, Key::Unknown, Key::Unknown, Key::Unknown, Key::ArrowU, Key::Unknown, Key::ArrowD, Key::Unknown, Key::Unknown,
Key::ArrowR, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown,
Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown,
Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown,
Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown,
Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown, Key::Unknown];


#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Key1 = 2, Key2 = 3, Key3 = 4, Key4 = 5, Key5 = 6, Key6 = 7, Key7 = 8, Key8 = 9, Key9 = 10, Key0 = 11, UpZero = 12, Min = 13,
    BackSpace = 14,

    Tab = 15, LetA = 16, LetZ = 17, LetE = 18, LetR = 19, LetT = 20, LetY = 21, LetU = 22, LetI = 23, LetO = 24, LetP = 25,
    Accent = 26, Dolar = 27, Enter = 28,
    
    Maj = 58, LetQ = 30, LetS = 31, LetD = 32, LetF = 33, LetG = 34, LetH = 35, LetJ = 36, LetK = 37, LetL = 38, LetM = 39,
    Pourcent = 40, Tild = 43,

    ShiftL = 42, Ineg = 41, LetW = 44, LetX = 45, LetC = 46, LetV = 47, LetB = 48, LetN = 49, Comma = 50, Dot = 51, Slash = 52,
    Equal = 53, ShiftR = 54,

    Space = 57,
    
    ArrowU = 75,
    ArrowD = 77,
    ArrowL = 72,
    ArrowR = 80,
    Unknown = 0,

}


/*
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
            KeyCode::Delete => DecodedKey::Unicode(127.into()),
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