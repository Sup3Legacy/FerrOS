use crate::data_storage::path::Path;
use super::partition::Partition;
use crate::vga::mainscreen;
use crate::vga::video_mode::VirtualScreenLayer;

/// Used to define an empty partition
#[derive(Debug)]
pub struct ScreenPartition {
    screen_id = VirtualScreenID;
}

impl ScreenPartition {
    pub const fn new(row_top: usize, col_left: usize, height: usize, width: usize, layer: usize) -> Self {
        if let mut Some(main_screen) = mainscreen::MAIN_SCREEN {
            Self {
                screen_id : main_screen.new_screen(row_top, col_left, height, width, VirtualScreenLayer(layer)),
            }
        } else {
            mainscreen::MAIN_SCREEN = Some(mainscreen::MainScreen::new())
            ScreenPartition::new(row_top, col_left, height, width)
        }
    }
}

impl Partition for ScreenPartition {
    fn read(&self, _path: Path, _offset: usize, _size: usize) -> Vec<u8> {
        panic!("not allowed");
    }

    fn write(&self, _path: Path, _buffer: Vec<u8>) -> usize {
        if let mut Some(main_screen) = mainscreen::MAIN_SCREEN {
            if let mut Some(screen) = main_screen.get_screen(self.screen_id) {
                screen.write_byte_vec(_buffer);
                _buffer.size()
            } else {
                0
            }
        } else {
            panic!("should never happen")
        }
    }

    fn lseek(&self) {
        panic!("not allowed");
    }

    fn flush(&self) {
        panic!("not allowed");
    }

    fn read_raw(&self) {
        panic!("not allowed");
    }
}