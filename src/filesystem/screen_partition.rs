use super::partition::Partition;
use crate::scheduler::process;
use crate::{data_storage::path::Path, errorln};
use crate::{vga::mainscreen, warningln};
use alloc::vec::Vec;

/// Used to define an empty partition
#[derive(Debug)]
pub struct ScreenPartition {}

impl ScreenPartition {
    pub fn new() -> Self {
        unsafe {
            if let Some(_main_screen) = &mut mainscreen::MAIN_SCREEN {
                Self {}
            } else {
                mainscreen::MAIN_SCREEN = Some(mainscreen::MainScreen::new());
                ScreenPartition::new()
            }
        }
    }
}

impl Partition for ScreenPartition {
    fn read(&self, _path: &Path, _offset: usize, _size: usize) -> Vec<u8> {
        panic!("not allowed");
    }

    fn write(&mut self, _path: &Path, _buffer: &[u8], offset: usize, flags: u64) -> isize {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let process = process::get_current();
                let v_screen_id = process.screen;
                let v_screen = main_screen.get_vscreen_mut(&v_screen_id);
                if let Some(screen) = v_screen {
                    screen.write_byte_vec(&_buffer);
                    main_screen.draw();
                    _buffer.len() as isize
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

    fn close(&mut self, path: &Path) -> bool {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let process = process::get_current();
                let v_screen_id = process.screen;
                main_screen.delete_screen(v_screen_id);
            }
        }
        false
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
impl Default for ScreenPartition {
    fn default() -> Self {
        Self::new()
    }
}
