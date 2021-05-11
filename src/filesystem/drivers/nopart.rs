use crate::data_storage::path::Path;
use crate::filesystem::partition::Partition;
use alloc::vec::Vec;

/// Used to define an empty partition
#[derive(Debug)]
pub struct NoPart {}

impl NoPart {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Partition for NoPart {
    fn read(&self, _path: &Path, _offset: usize, _size: usize) -> Vec<u8> {
        todo!()
    }

    fn write(&mut self, _path: &Path, _buffer: &[u8], _offset: usize, _flags: u64) -> isize {
        todo!()
    }

    fn close(&mut self, _path: &Path) -> bool {
        todo!()
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
}
