//! Provides bindings to the different functions in `hardware`, `keyboard`, `sound`, etc.
use super::super::partition::Partition;
use crate::data_storage::path::Path;
use alloc::vec::Vec;

pub mod clock;
pub mod keyboard;
pub mod sound;

pub trait HardwareInterface {
    fn read(&self);
    fn write(&self);
}

pub struct HardWarePartition;

impl Partition for HardWarePartition {
    fn read(&self, _path: &Path, _offset: usize, _size: usize) -> Vec<u8> {
        todo!()
    }

    fn write(&mut self, _path: &Path, _buffer: &[u8], _offset: usize, _flags: u64) -> isize {
        todo!()
    }

    fn close(&mut self, _path: &Path) -> bool {
        todo!()
    }

    fn flush(&self) {
        todo!()
    }

    fn lseek(&self) {
        todo!()
    }

    fn read_raw(&self) {
        todo!()
    }
}
