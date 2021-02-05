//use pc_keyboard::{Keyboard, Modifiers, KeyCode, DecodedKey};
use crossbeam_queue::{ArrayQueue, PopError};
use alloc::boxed::Box;
use lazy_static::lazy_static;
use alloc::string::String;
use crate::{println, print};
use crate::keyboard::keyboard_layout;
const TAILLE: usize = 80;

struct DoubleFile {
    debut : usize,
    fin : usize,
    tableau : [char; TAILLE],
    boucle : bool
}

impl DoubleFile {
    fn push(&mut self, byte : char) {
        if self.boucle && (self.debut == self.fin) {
            self.tableau[self.fin] = byte;
            self.debut = (self.debut + 1)%TAILLE;
            self.fin = (self.fin + 1)%TAILLE;
        } else {
            self.tableau[self.fin] = byte;
            self.fin = (self.fin + 1)%TAILLE;
            self.boucle = true;
        }
    }

    fn pop_end(&mut self) {
        if (self.boucle && (self.debut == self.fin)) || self.fin != self.debut {
            self.fin = (self.fin + TAILLE - 1)%TAILLE;
            self.boucle = false;
        }
    }

    fn print_first(&mut self, cache : bool) -> bool {
        if self.debut != self.fin || self.boucle {
            print!("{}",if cache {0xfe as char} else {self.tableau[self.debut]});
            self.debut = (self.debut + 1)%TAILLE;
            self.boucle = false;
            true
        } else {
            false
        }
    }

    fn sortie(&mut self) -> [char; TAILLE] {
        let mut s  = [0xfe as char; TAILLE];
        if self.debut < self.fin {
            for i in self.debut..self.fin {
                s[i - self.debut] = self.tableau[i];
            }
        } else if self.debut != self.fin || self.boucle {
            for i in self.debut..TAILLE {
                s[i - self.debut] = self.tableau[i];
            };
            for i in 0..self.fin {
                s[i + TAILLE - self.debut] = self.tableau[i];
            }
        };
        s
    }
}

pub fn get_input(cache : bool) -> String {
    let mut stack = String::new();
    loop {
        match {crate::keyboard::get_top_value()} {
            Ok(a) => {
                match a {
                    keyboard_layout::KeyEvent::Character('\n') => {
                        if stack.len() != 0 {
                        println!("");
                        break stack}
                        },

                    keyboard_layout::KeyEvent::Character('\x08') => {
                        stack.pop();
                        if cache {
                            print!("\r");
                            for i in 0..stack.len() {
                                print!(" ");
                            }
                            print!(" \r");
                            for i in 0..stack.len() {
                                print!("{}", 0x1e as char);
                            }
                        } else {
                            print!("\r{} \r{}",stack, stack);
                        }

                    },

                    keyboard_layout::KeyEvent::Character(character) => {
                        stack.push(character);
                        if cache {
                            print!("{}", 0x1e as char);
                        } else {
                            print!("{}", character);
                        }
                        },

                    keyboard_layout::KeyEvent::SpecialKey(0) => {
                        stack.pop();
                        if cache {
                            print!("\r");
                            for i in 0..stack.len() {
                                print!(" ");
                            }
                            print!(" \r");
                            for i in 0..stack.len() {
                                print!("{}", 0x1e as char);
                            }
                        } else {
                            print!("\r{} \r{}",stack, stack);
                        }
                        },
                    keyboard_layout::KeyEvent::SpecialKey(key) => (),
                }
            },

            Err(_) => ()
        }
    }
}

