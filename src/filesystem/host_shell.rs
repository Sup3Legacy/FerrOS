//! host shell accessed by the serial interface

use super::partition::{IoError, Partition};
use crate::filesystem::descriptor::OpenFileTable;
use crate::filesystem::fsflags::OpenFlags;

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
    fn open(&mut self, _path: &Path, _flags: OpenFlags) -> Option<usize> {
        Some(0)
    }

    fn read(&mut self, _oft: &OpenFileTable, _size: usize) -> Result<Vec<u8>, IoError> {
        panic!("not allowed");
    }

    fn write(&mut self, _oft: &OpenFileTable, buffer: &[u8]) -> isize {
        let mut sortie = String::new();
        let size = buffer.len();
        for item in buffer.iter() {
            sortie.push(*item as char);
        }
        print!("{}", sortie);
        size as isize
    }

    fn close(&mut self, _oft: &OpenFileTable) -> bool {
        false
    }

    /*fn duplicate(&mut self, _oft: &OpenFileTable) -> Option<usize> {
        Some(0)
    }*/

    fn lseek(&self) {
        panic!("not allowed");
    }

    fn flush(&self) {
        panic!("not allowed");
    }

    fn read_raw(&self) {
        panic!("not allowed");
    }

    fn give_param(&mut self, _oft: &OpenFileTable, _param: usize) -> usize {
        usize::MAX
    }
}
