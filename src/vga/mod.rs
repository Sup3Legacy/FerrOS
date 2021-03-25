//! Implements all functions related to the display.
//! Currently uses an older version, directly writing onto the
//! physical display.


use lazy_static::lazy_static;
use spin::Mutex;





pub mod mainscreen;
pub mod video_mode;
pub mod virtual_screen;

use mainscreen::MainScreen;

lazy_static! {
    /// Main screen structure.
    pub static ref SCREEN: Mutex<MainScreen> = Mutex::new(MainScreen::new());
}

pub fn draw_screen() {
    SCREEN.lock().draw()
}
