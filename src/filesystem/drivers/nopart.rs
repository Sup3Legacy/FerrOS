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
    fn read(&self, _path: Path, _offset: usize, _size: usize) -> Vec<u8> {
        todo!()
    }

    fn write(&self, _path: Path, _buffer: Vec<u8>) -> usize {
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
