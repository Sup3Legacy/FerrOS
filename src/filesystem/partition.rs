// Each storage element (be it an ATA disk or  a virtual system)
/// needs to implement this trait in order to get integrated into the
/// VFS.
pub trait Partition {
    /// Reads a file
    fn read(&self) -> ();

    /// Writes a file
    fn write(&self) -> ();

    /// Flushes all changes to a file
    fn flush(&self) -> ();

    /// TODO
    fn lseek(&self) -> ();
}
