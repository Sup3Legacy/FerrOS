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

impl Partition for HostShellPartition {
    fn read(&self, _path: &Path, _offset: usize, _size: usize) -> Vec<u8> {
        panic!("not allowed");
    }

    fn write(&mut self, _path: &Path, buffer: &[u8], offset: usize, flags: u64) -> isize {
        let mut sortie = String::new();
        let size = buffer.len();
        for i in 0..size {
            sortie.push(buffer[i] as char);
        }
        print!("{}", sortie);
        size as isize
    }

    fn close(&mut self, path: &Path) -> bool {
        false
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
}
