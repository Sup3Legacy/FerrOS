use core::str::SplitWhitespace;

use crate::keyboard::keyboard_interraction;
use alloc::collections::BTreeMap;
use alloc::{string::String, vec::Vec};
use lazy_static::lazy_static;

use crate::{print, println};

#[derive(Debug, Clone)]
pub struct ShellErr {
    message: String,
}
#[derive(Clone, Debug)]
pub struct ShellCommand {
    pub keyword: String,
    pub help: String,
    pub function: fn(Vec<String>) -> Result<(), ShellErr>,
}

pub fn _test1(a: Vec<String>) -> Result<(), ShellErr> {
    println!("test");
    Ok(())
}

lazy_static! {
    pub static ref COMMANDS: BTreeMap<String, ShellCommand> = {
        let mut commands = BTreeMap::new();
        let test_command = ShellCommand {
            keyword: String::from("test"),
            help: String::new(),
            function: _test1,
        };
        commands.insert(String::from("test"), test_command);
        commands
    };
}

pub fn main_shell() -> () {
    println!("nom d'utilisateur : {}", 0xfe as char);
    let _utilisateur = keyboard_interraction::get_input(false);
    println!();
    println!("mot de passe : ");
    let _mpd = keyboard_interraction::get_input(true);
    _main_loop();
}

pub fn _main_loop() -> ! {
    loop {
        let a = keyboard_interraction::get_input(false);
        let mut it = _parse_input_into_vec(&a);
        match it.next() {
            Some(a) => match COMMANDS.get(a) {
                Some(command) => {
                    let func = command.function;
                    match func(it.map(|x| String::from(x)).collect::<Vec<String>>()) {
                        Ok(()) => (),
                        _ => println!("{}", command.help),
                    }
                }
                None => println!("No such command."),
            },
            None => println!("Empty command."),
        }
    }
}

pub fn _parse_input_into_vec<'a>(s: &'a String) -> SplitWhitespace<'a> {
    s.split_whitespace()
}
