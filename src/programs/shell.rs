use core::str::SplitWhitespace;

use crate::keyboard::keyboard_interaction;
use alloc::collections::BTreeMap;
use alloc::{string::String, vec::Vec};
use lazy_static::lazy_static;

use super::ascii_fluid;

use crate::{print, println};

#[derive(Debug, Clone)]
pub struct ShellErr {
    message: String,
}

/// ShellCommand is the wrapper around each command callable from the shell
///
/// *Attributes*
/// - `keyword` keyword through which one can call the command
/// - `help` help message displayd when the execution of the command returns an error
/// - `function` main function of the command
#[derive(Clone, Debug)]
pub struct ShellCommand {
    pub keyword: String,
    pub help: String,
    pub function: fn(Vec<String>) -> Result<(), ShellErr>,
}

pub fn _test1(_a: Vec<String>) -> Result<(), ShellErr> {
    println!("test");
    Ok(())
}

#[allow(unreachable_code)]
pub fn ascii(_: Vec<String>) -> Result<(), ShellErr> {
    ascii_fluid::main();
    Ok(())
}

lazy_static! {
    /// Main BTreeMap. Contains the bindings `keyword` <=> `command`
    pub static ref COMMANDS: BTreeMap<String, ShellCommand> = {
        let mut commands = BTreeMap::new();
        let test_command = ShellCommand {
            keyword: String::from("test"),
            help: String::new(),
            function: _test1,
        };
        commands.insert(String::from("test"), test_command);
        let test_command = ShellCommand {
            keyword: String::from("ascii"),
            help: String::new(),
            function: ascii,
        };
        commands.insert(String::from("ascii"), test_command);
        commands
    };
}

/// Entry function of the shell
///
/// TODO : clean it and make it more general
pub fn main_shell() -> () {
    let _utilisateur = keyboard_interaction::get_input("pseudo : ", false);
    println!();
    let _mpd = keyboard_interaction::get_input("mdp : ", true);
    _main_loop();
}

/// Main Read-Evaluate-Print loop of the shell.
///
/// The user can write comands.
/// The first word is the keywords, which indicates which (software-defined) programed is called
pub fn _main_loop() -> ! {
    loop {
        let a = keyboard_interaction::get_input(">> ", false);
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

/// Temporary function.
/// Will be modified or removed in the future
pub fn _parse_input_into_vec<'a>(s: &'a String) -> SplitWhitespace<'a> {
    s.split_whitespace()
}
