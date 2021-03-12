pub trait Partition {
    fn open(&self) -> ();
    fn close(&self) -> ();
    fn read(&self) -> ();
    fn write(&self) -> ();
    fn lseek(&self) -> ();
}