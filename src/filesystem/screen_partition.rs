use super::partition::Partition;
use crate::scheduler::process;
use crate::{data_storage::path::Path, errorln};
use crate::{debug, vga::mainscreen, vga::virtual_screen::VirtualScreenLayer, warningln};
use alloc::string::String;
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

    fn open(&mut self, _path: &Path) -> usize {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let s = main_screen.new_screen(0, 0, 25, 80, VirtualScreenLayer::new(0));
                s.as_usize()
            } else {
                panic!("Mainscreen uninitialised")
            }
        }
    }


    fn read(&mut self, _path: &Path, _id: usize, _offset: usize, _size: usize) -> Vec<u8> {
        panic!("not allowed");
    }

    fn write(&mut self, _path: &Path, id: usize, buffer: &[u8], offset: usize, flags: u64) -> isize {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let v_screen_id = mainscreen::VirtualScreenID::forge(id);
                let v_screen = main_screen.get_vscreen_mut(&v_screen_id);
                if let Some(screen) = v_screen {
                    screen.write_string(&(String::from_utf8_lossy(buffer)));
                    main_screen.draw();
                    buffer.len() as isize
                } else {
                    warningln!(
                        "Attempted to write in non-existing virtualscreen : {:?}",
                        v_screen_id
                    );
                    panic!("exit");
                    0
                }
            } else {
                errorln!("Mainscreen not initialized!");
                0
            }
        }
    }

    fn close(&mut self, path: &Path, id: usize) -> bool {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let v_screen_id = mainscreen::VirtualScreenID::forge(id);
                main_screen.delete_screen(v_screen_id);
            }
        }
        false
    }

    fn duplicate(&mut self, path: &Path, id: usize) -> Option<usize> {
        Some(id)
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
