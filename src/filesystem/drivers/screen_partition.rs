//! Give a process access to a screen.

use super::super::partition::{IoError, Partition};
use crate::data_storage::screen::Coord;
use crate::filesystem::descriptor::OpenFileTable;
use crate::filesystem::fsflags::OpenFlags;
use crate::{data_storage::path::Path, errorln};
use crate::{vga::mainscreen, vga::virtual_screen::VirtualScreenLayer, warningln};
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
    fn open(&mut self, path: &Path, _flags: OpenFlags) -> Option<usize> {
        if path.len() != 0 {
            return None;
        }
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let s = main_screen.new_screen(0, 0, 0, 0, VirtualScreenLayer::new(0));
                Some(s.as_usize())
            } else {
                crate::debug!("no main screen");
                None
            }
        }
    }

    fn read(&mut self, _oft: &OpenFileTable, _size: usize) -> Result<Vec<u8>, IoError> {
        Err(IoError::Kill)
    }

    fn write(&mut self, oft: &OpenFileTable, buffer: &[u8]) -> isize {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let v_screen_id = mainscreen::VirtualScreenID::forge(oft.get_id());
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
                }
            } else {
                errorln!("Mainscreen not initialized!");
                0
            }
        }
    }

    fn close(&mut self, oft: &OpenFileTable) -> bool {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let v_screen_id = mainscreen::VirtualScreenID::forge(oft.get_id());
                main_screen.delete_screen(v_screen_id)
            } else {
                panic!("should not happen")
            }
        }
    }

    /*fn duplicate(&mut self, _path: &Path, id: usize) -> Option<usize> {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                main_screen.duplicated(mainscreen::VirtualScreenID::forge(id))
            }
        }
        Some(id)
    }*/

    fn lseek(&self) {
        panic!("not allowed");
    }

    fn flush(&self) {
        panic!("not allowed");
    }

    fn read_raw(&self) {
        panic!("not allowed");
    }

    fn give_param(&mut self, oft: &OpenFileTable, param: usize) -> usize {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let v_screen_id = mainscreen::VirtualScreenID::forge(oft.get_id());
                if param >> 63 == 1 {
                    let x = param & 0xFF;
                    let y = (param >> 32) & 0xFF;
                    crate::debug!("resize {} {}", x, y);
                    main_screen.resize_vscreen(&v_screen_id, Coord::new(x, y))
                } else {
                    let x = param & 0xFF;
                    let y = (param >> 32) & 0xFF;
                    crate::debug!("move {} {}", x, y);
                    main_screen.replace_vscreen(&v_screen_id, Coord::new(x, y))
                }
                0
            } else {
                usize::MAX
            }
        }
    }
}
impl Default for ScreenPartition {
    fn default() -> Self {
        Self::new()
    }
}
