use pc_keyboard::{Keyboard, Modifiers, KeyCode, DecodedKey};
use alloc::boxed::Box;
use lazy_static::lazy_static;
use crate::{println, print};

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

pub fn get_input(cache : bool) -> [char; TAILLE] {
    let mut stack = DoubleFile {
        debut : 0,
        fin : 0,
        tableau : [' '; TAILLE],
        boucle : false
    };
    loop {
        match crate::keyboard::get_top_value() {
            Ok(a) => {
                match a {
                    DecodedKey::Unicode('\n') => {
                        if stack.fin != 0 {
                        let sortie = stack.sortie();
                        while stack.debut != stack.fin {
                            stack.print_first(cache);
                        };
                        println!("");
                        break sortie}
                        },

                    DecodedKey::Unicode('\x08') => {
                        stack.pop_end();
                        for i in 1..TAILLE {
                            print!(" ")
                        };
                        print!("\r");
                        let deb = stack.debut;
                        let fin = stack.fin;
                        let boucle = stack.boucle;
                        while stack.debut != stack.fin {
                            stack.print_first(cache);
                        };
                        stack.debut = deb;
                        stack.fin = fin;
                        stack.boucle = boucle;
                        print!("\r")
                    },

                    DecodedKey::Unicode(character) => {
                        stack.push(character);
                        let deb = stack.debut;
                        let fin = stack.fin;
                        let boucle = stack.boucle;
                        while stack.debut != stack.fin {
                            stack.print_first(cache);
                        };
                        stack.debut = deb;
                        stack.fin = fin;
                        stack.boucle = boucle;
                        print!("\r")
                        },
                    DecodedKey::RawKey(KeyCode::Delete) => stack.pop_end(),
                    DecodedKey::RawKey(key) => (),
                }
            },

            Err(_) => ()
        }
    }
}

