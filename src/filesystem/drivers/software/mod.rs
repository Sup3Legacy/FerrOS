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
    fn open(&mut self, _path: &Path) -> Option<usize> {
        todo!()
    }

    fn read(&mut self, _path: &Path, _id: usize, _offset: usize, _size: usize) -> Vec<u8> {
        todo!()
    }

    fn write(
        &mut self,
        _path: &Path,
        _id: usize,
        _buffer: &[u8],
        _offset: usize,
        _flags: u64,
    ) -> isize {
        todo!()
    }

    fn close(&mut self, _path: &Path, _id: usize) -> bool {
        todo!()
    }

    fn duplicate(&mut self, _path: &Path, _id: usize) -> Option<usize> {
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

    fn give_param(&mut self, _path: &Path, _id: usize, _param: usize) -> usize {
        usize::MAX
    }
}
