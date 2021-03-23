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
    fn read(&self, path: Path, offset: usize, size: usize) -> Vec<u8> {
        todo!()
    }

    fn write(&self, path: Path, buffer: Vec<u8>) -> usize {
        todo!()
    }

    fn flush(&self) -> () {
        todo!()
    }

    fn lseek(&self) -> () {
        todo!()
    }

    fn read_raw(&self) -> () {
        todo!()
    }
}
