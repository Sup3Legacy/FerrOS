use crate::data_storage::path::Path;
use crate::filesystem::descriptor::OpenFileTable;
use crate::filesystem::fsflags::OpenFlags;
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
    fn open(&mut self, _path: &Path, _flags: OpenFlags) -> Option<usize> {
        todo!()
    }

    fn read(&mut self, _oft: &OpenFileTable, _size: usize) -> Vec<u8> {
        todo!()
    }

    fn write(&mut self, _oft: &OpenFileTable, _buffer: &[u8]) -> isize {
        todo!()
    }

    fn close(&mut self, _oft: &OpenFileTable) -> bool {
        todo!()
    }

    /*fn duplicate(&mut self, _oft: &OpenFileTable) -> Option<usize> {
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
