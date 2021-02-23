use lazy_static::lazy_static;
use vga::colors::Color16;
use vga::writers::{Graphics640x480x16, GraphicsWriter};

lazy_static! {
    /// OS-wide video-mode VGA handler.
    ///
    /// It is by default in `640x480x16` mode, but we might add a feature to change it on the fly from the OS.
    ///
    /// The default background color is black.
    pub static ref VIDEOMODE : Graphics640x480x16 = {
        let mode = Graphics640x480x16::new();
        //mode.set_mode();
        mode.clear_screen(Color16::Black);
        mode
    };
}

/// Basic initialization function for the video-mode VGA handler.
///
/// It contains some example code for lines as well as characters.
pub fn init() -> () {
    VIDEOMODE.set_mode(); // enables the video-mode (instead of the text mode)
    VIDEOMODE.draw_line((80, 60), (80, 420), Color16::White);
    VIDEOMODE.draw_line((80, 60), (540, 60), Color16::White);
    VIDEOMODE.draw_line((80, 420), (540, 420), Color16::White);
    VIDEOMODE.draw_line((540, 420), (540, 60), Color16::White);
    VIDEOMODE.draw_line((80, 90), (540, 90), Color16::White);
    for (offset, character) in "Hello World!".chars().enumerate() {
        VIDEOMODE.draw_character(270 + offset * 8, 72, character, Color16::White)
    }
}
