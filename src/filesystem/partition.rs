//! Defines the `Partition` trait, implemented by all drivers and uniting them into a `VFS`!

use super::descriptor::OpenFileTable;
use super::fsflags::OpenFlags;
use crate::data_storage::path::Path;
use alloc::vec::Vec;

pub enum IoError {
    Continue,
    Kill,
    Sleep,
}

/// Each storage element (be it an ATA disk or  a virtual system)
/// needs to implement this trait in order to get integrated into the
/// VFS.
pub trait Partition {
    fn open(&mut self, path: &Path, flags: OpenFlags) -> Option<usize>;
    /// Reads a file
    /// Takes as a parameter the path to the file, the offset and the size
    /// Returns the read buffer
    fn read(&mut self, oft: &OpenFileTable, size: usize) -> Result<Vec<u8>, IoError>;

    /// Writes a file
    /// Might wanna add some flags...
    fn write(&mut self, oft: &OpenFileTable, buffer: &[u8]) -> isize;

    /// Flushes all changes to a file
    fn flush(&self);

    /// TODO
    fn lseek(&self);

    /// This is the function the kernel reads through.
    fn read_raw(&self);

    /// Duplicate file ?
    //    fn duplicate(&mut self, path: &Path, id: usize) -> Option<usize>;

    /// Close
    fn close(&mut self, oft: &OpenFileTable) -> bool;

    /// Param
    fn give_param(&mut self, oft: &OpenFileTable, param: usize) -> usize;
}
