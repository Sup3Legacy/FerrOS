use super::partition::Partition;
use crate::data_storage::screen::Coord;

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
    fn open(&mut self, path: &Path) -> Option<usize> {
        if path.len() != 0 {
            return None;
        }
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let s = main_screen.new_screen(0, 0, 25, 80, VirtualScreenLayer::new(0));
                Some(s.as_usize())
            } else {
                crate::debug!("no main screen");
                None
            }
        }
    }

    fn read(&mut self, _path: &Path, _id: usize, _offset: usize, _size: usize) -> Vec<u8> {
        panic!("not allowed");
    }

    fn write(
        &mut self,
        _path: &Path,
        id: usize,
        buffer: &[u8],
        _offset: usize,
        _flags: u64,
    ) -> isize {
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
                }
            } else {
                errorln!("Mainscreen not initialized!");
                0
            }
        }
    }

    fn close(&mut self, _path: &Path, id: usize) -> bool {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let v_screen_id = mainscreen::VirtualScreenID::forge(id);
                main_screen.delete_screen(v_screen_id)
            } else {
                panic!("should not happen")
            }
        }
    }

    fn duplicate(&mut self, _path: &Path, id: usize) -> Option<usize> {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                main_screen.duplicated(mainscreen::VirtualScreenID::forge(id))
            }
        }
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

    fn give_param(&mut self, _path: &Path, id: usize, param: usize) -> usize {
        unsafe {
            if let Some(main_screen) = &mut mainscreen::MAIN_SCREEN {
                let v_screen_id = mainscreen::VirtualScreenID::forge(id);
                if param >> 63 == 1 {
                    main_screen.resize_vscreen(
                        &v_screen_id,
                        Coord::new(param & 0xFF, (param >> 32) & 0xFF),
                    )
                } else {
                    main_screen.replace_vscreen(&v_screen_id, Coord::new(param & 0xFF, param >> 32))
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
