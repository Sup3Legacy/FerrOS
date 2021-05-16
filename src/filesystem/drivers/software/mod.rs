//! Provides bindings to the different functions in `hardware`, `keyboard`, `sound`, etc.
use super::super::partition::{IoError, Partition};
use crate::data_storage::path::Path;
use crate::filesystem::descriptor::OpenFileTable;
use crate::filesystem::fsflags::OpenFlags;

use alloc::vec::Vec;

pub trait SoftWareInterface {
    fn read(&self);
    fn write(&self);
}

pub struct SoftwarePartition;

impl Partition for SoftwarePartition {
    fn open(&mut self, _path: &Path, _flags: OpenFlags) -> Option<usize> {
        todo!()
    }

    fn read(&mut self, _oft: &OpenFileTable, _size: usize) -> Result<Vec<u8>, IoError> {
        todo!()
    }

    fn write(&mut self, _oft: &OpenFileTable, _buffer: &[u8]) -> isize {
        todo!()
    }

    fn close(&mut self, _oft: &OpenFileTable) -> bool {
        todo!()
    }

    /*fn duplicate(&mut self, _path: &Path, _id: usize) -> Option<usize> {
        todo!()
    }*/

    fn flush(&self) {
        todo!()
    }

    fn lseek(&self) {
        todo!()
    }

    fn read_raw(&self) {
        todo!()
    }

    fn give_param(&mut self, _oft: &OpenFileTable, _param: usize) -> usize {
        usize::MAX
    }
}
