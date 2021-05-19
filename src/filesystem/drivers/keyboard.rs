use super::super::partition::{IoError, Partition};

use crate::{data_storage::path::Path, warningln};

use crate::filesystem::descriptor::OpenFileTable;
use crate::filesystem::fsflags::OpenFlags;
use crate::keyboard::get_top_key_event;
use alloc::vec::Vec;

pub struct KeyBoard;

impl KeyBoard {
    pub fn new() -> Self {
        Self
    }
}

impl Default for KeyBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl Partition for KeyBoard {
    fn open(&mut self, path: &Path, _flags: OpenFlags) -> Option<usize> {
        if !path.is_empty(){
            None
        } else {
            Some(0)
        }
    }

    fn read(&mut self, _oft: &OpenFileTable, size: usize) -> Result<Vec<u8>, IoError> {
        // The number of packets to be written into the buffer
        let mut res = Vec::new();
        for _ in 0..size {
            if let Ok(k) = get_top_key_event() {
                res.push(k);
            } else {
                return Ok(res);
            }
        }
        Ok(res)
    }

    fn write(&mut self, _oft: &OpenFileTable, _buffer: &[u8]) -> isize {
        warningln!("User-program attempted to write in keyboard.");
        -1
    }

    fn close(&mut self, _oft: &OpenFileTable) -> bool {
        false
    }

    /*fn duplicate(&mut self, _path: &Path, _id: usize) -> Option<usize> {
        Some(0)
    }*/

    fn lseek(&self) {
        todo!()
    }

    fn flush(&self) {
        todo!()
    }

    fn read_raw(&self) {
        todo!()
    }

    fn give_param(&mut self, _oft: &OpenFileTable, _param: usize) -> usize {
        usize::MAX
    }
}
