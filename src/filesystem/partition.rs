use crate::data_storage::path::Path;
use alloc::vec::Vec;

/// Each storage element (be it an ATA disk or  a virtual system)
/// needs to implement this trait in order to get integrated into the
/// VFS.
pub trait Partition {
    fn open(&mut self, path: &Path) -> Option<usize>;
    /// Reads a file
    /// Takes as a parameter the path to the file, the offset and the size
    /// Returns the read buffer
    fn read(&mut self, path: &Path, id: usize, offset: usize, size: usize) -> Vec<u8>;

    /// Writes a file
    /// Might wanna add some flags...
    fn write(&mut self, path: &Path, id: usize, buffer: &[u8], offset: usize, flags: u64) -> isize;

    /// Flushes all changes to a file
    fn flush(&self);

    /// TODO
    fn lseek(&self);

    /// This is the function the kernel reads through.
    fn read_raw(&self);

    /// Duplicate file ?
    fn duplicate(&mut self, path: &Path, id: usize) -> Option<usize>;

    /// Close
    fn close(&mut self, path: &Path, id: usize) -> bool;

    /// Param
    fn give_param(&mut self, path: &Path, id: usize, param: usize) -> usize;
}
