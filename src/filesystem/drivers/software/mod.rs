//! Provides bindings to the different functions in `hardware`, `keyboard`, `sound`, etc.
use super::super::partition::Partition;
use crate::data_storage::path::Path;
use alloc::vec::Vec;

pub trait SoftWareInterface {
    fn read(&self);
    fn write(&self);
}

pub struct SoftwarePartition;

impl Partition for SoftwarePartition {
    fn read(&self, _path: Path, _offset: usize, _size: usize) -> Vec<u8> {
        todo!()
    }

    fn write(&mut self, _path: Path, _buffer: Vec<u8>) -> usize {
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
