use super::partition::Partition;
use crate::scheduler::process;
use crate::vga::virtual_screen::VirtualScreenLayer;
use crate::{data_storage::path::Path, errorln};
use crate::{vga::mainscreen, warningln};
use alloc::vec::Vec;

/// Used to define an empty partition
#[derive(Debug)]
pub struct ScreenPartition {
    pub screen_id: mainscreen::VirtualScreenID,
}

impl ScreenPartition {
    pub fn new(row_top: usize, col_left: usize, height: usize, width: usize, layer: usize) -> Self {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                Self {
                    screen_id: main_screen.new_screen(
                        row_top,
                        col_left,
                        height,
                        width,
                        VirtualScreenLayer(layer),
                    ),
                }
            } else {
                mainscreen::MAIN_SCREEN = Some(mainscreen::MainScreen::new());
                ScreenPartition::new(row_top, col_left, height, width, layer)
            }
        }
    }
}

impl Partition for ScreenPartition {
    fn read(&self, _path: Path, _offset: usize, _size: usize) -> Vec<u8> {
        panic!("not allowed");
    }

    fn write(&self, _path: Path, _buffer: Vec<u8>) -> usize {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let process = process::get_current();
                let v_screen_id = process.screen;
                let v_screen = main_screen.get_vscreen_mut(&v_screen_id);
                if let Some(screen) = v_screen {
                    screen.write_byte_vec(&_buffer);
                    main_screen.draw();
                    _buffer.len()
                } else {
                    warningln!(
                        "Attempted to write in non-existing virtualscreen : {:?}",
                        v_screen_id
                    );
                    0
                }
            } else {
                errorln!("Mainscreen not initialized!");
                0
            }
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
