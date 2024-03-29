//use pc_keyboard::{Keyboard, Modifiers, KeyCode, DecodedKey};
use crate::keyboard::keyboard_layout;
use crate::{print, println};
use alloc::string::String;

/*
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
}*/

pub fn get_input(debut: &str, cache: bool) -> String {
    println!(":D");
    let mut stack = String::new();
    println!(":D :D");
    print!("{}", debut);
    loop {
        if let Ok(a) = crate::keyboard::get_top_value() {
            match a {
                keyboard_layout::KeyEvent::Character('\n') => {
                    if !stack.is_empty() {
                        println!();
                        break stack;
                    }
                }

                keyboard_layout::KeyEvent::Character('\x08') => {
                    stack.pop();
                    if !cache {
                        print!("\r{}{} \r{}{}", debut, stack, debut, stack);
                    }
                }

                keyboard_layout::KeyEvent::Character(character) => {
                    stack.push(character);
                    if !cache {
                        print!("{}", character);
                    }
                }

                keyboard_layout::KeyEvent::SpecialKey(0) => {
                    stack.pop();
                    if !cache {
                        print!("\r{}{} \r{}{}", debut, stack, debut, stack);
                    }
                }
                keyboard_layout::KeyEvent::SpecialKey(_) => (),
            }
        }
    }
}
