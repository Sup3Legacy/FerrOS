use super::super::partition::Partition;

use crate::{data_storage::path::Path, warningln};

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
    fn open(&mut self, path: &Path) -> Option<usize> {
        if path.len() != 0 {
            None
        } else {
            Some(0)
        }
    }

    fn read(&mut self, _path: &Path, _id: usize, _offset: usize, size: usize) -> Vec<u8> {
        // The number of packets to be written into the buffer
        let mut res = Vec::new();
        for _ in 0..size {
            if let Ok(k) = get_top_key_event() {
                res.push(k);
            } else {
                return res;
            }
        }
        res
    }

    fn write(
        &mut self,
        _path: &Path,
        _id: usize,
        _buffer: &[u8],
        _offset: usize,
        _flags: u64,
    ) -> isize {
        warningln!("User-program attempted to write in keyboard.");
        -1
    }

    fn close(&mut self, _path: &Path, _id: usize) -> bool {
        false
    }

    fn duplicate(&mut self, _path: &Path, _id: usize) -> Option<usize> {
        Some(0)
    }

    fn lseek(&self) {
        todo!()
    }

    fn flush(&self) {
        todo!()
    }

    fn read_raw(&self) {
        todo!()
    }

    fn give_param(&mut self, _path: &Path, _id: usize, _param: usize) -> usize {
        usize::MAX
    }
}
