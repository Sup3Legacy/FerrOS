//! host shell accessed by the serial interface

use super::partition::Partition;

use crate::{data_storage::path::Path, print};
use alloc::string::String;
use alloc::vec::Vec;
pub struct HostShellPartition {}

impl HostShellPartition {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for HostShellPartition {
    fn default() -> Self {
        Self::new()
    }
}

impl Partition for HostShellPartition {
    fn open(&mut self, _path: &Path) -> Option<usize> {
        Some(0)
    }

    fn read(&mut self, _path: &Path, _id: usize, _offset: usize, _size: usize) -> Vec<u8> {
        panic!("not allowed");
    }

    fn write(
        &mut self,
        _path: &Path,
        _id: usize,
        buffer: &[u8],
        _offset: usize,
        _flags: u64,
    ) -> isize {
        let mut sortie = String::new();
        let size = buffer.len();
        for item in buffer.iter() {
            sortie.push(*item as char);
        }
        print!("{}", sortie);
        size as isize
    }

    fn close(&mut self, _path: &Path, _id: usize) -> bool {
        false
    }

    fn duplicate(&mut self, _path: &Path, _id: usize) -> Option<usize> {
        Some(0)
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

    fn give_param(&mut self, _path: &Path, _id: usize, _param: usize) -> usize {
        usize::MAX
    }
}
