use crate::data_storage::path::Path;
use alloc::vec::Vec;

/// Each storage element (be it an ATA disk or  a virtual system)
/// needs to implement this trait in order to get integrated into the
/// VFS.
pub trait Partition {
    /// Reads a file
    /// Takes as a parameter the path to the file, the offset and the size
    /// Returns the read buffer
    fn read(&self, path: &Path, offset: usize, size: usize) -> Vec<u8>;

    /// Writes a file
    /// Might wanna add some flags...
    fn write(&mut self, path: &Path, buffer: &[u8], offset: usize, flags: u64) -> isize;

    /// Flushes all changes to a file
    fn flush(&self);

    /// TODO
    fn lseek(&self);

    /// This is the function the kernel reads through.
    fn read_raw(&self);

    /// Close
    fn close(&mut self, path: &Path) -> bool;
}
