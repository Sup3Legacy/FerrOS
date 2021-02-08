use crate::keyboard::keyboard_interraction;
use alloc::string::String;

use crate::{print, println};

pub fn main_shell() -> () {
    println!("nom d'utilisateur : {}", 0xfe as char);
    let _utilisateur = keyboard_interraction::get_input(false);
    println!();
    println!("mot de passe : ");
    let _mpd = keyboard_interraction::get_input(true);
}