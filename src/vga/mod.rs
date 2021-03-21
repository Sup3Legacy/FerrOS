//! Implements all functions related to the display.
//! Currently uses an older version, directly writing onto the
//! physical display.
use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

use crate::{print, println};

pub mod mainscreen;
pub mod video_mode;
pub mod virtual_screen;

use mainscreen::MainScreen;

/// Main screen structure.
lazy_static! {
    pub static ref SCREEN: Mutex<MainScreen> = Mutex::new(MainScreen::new());
}

pub fn draw_screen() {
    SCREEN.lock().draw()
}
