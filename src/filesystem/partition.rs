
// Each storage element (be it an ATA disk or  a virtual system)
/// needs to implement this trait in order to get integrated into the 
/// VFS.
pub trait Partition {
    /// Opens a file from the given partition. It returns -1 (error)
    /// or he number of a file descriptor.
    fn open(&self) -> ();

    /// Closes the file represented by the file descriptor given as 
    /// argument.
    /// returns -1 if the execution fails, e.g. if the file hadd already
    /// been closed
    fn close(&self) -> ();

    /// Reads a certain amount of bytes from a file, designated by its file
    /// descriptor, into the given buffer. It returns the number of bytes
    /// that have effectlively been written.
    fn read(&self) -> ();

    /// Writes a certain amount of bytes from a buffer to the file.
    ///
    /// /!\ the write action itself isn't done until the file is closed.
    /// In the meantime, the new data is placed into a memory buffer.
    fn write(&self) -> ();

    /// TODO
    fn lseek(&self) -> ();
}
