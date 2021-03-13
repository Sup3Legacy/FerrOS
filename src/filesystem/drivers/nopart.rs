use crate::filesystem::partition::Partition;

/// Used to define an empty partition
#[derive(Debug)]
pub struct NoPart {}

impl NoPart {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Partition for NoPart {
    fn open(&self) -> () {
        todo!()
    }

    fn close(&self) -> () {
        todo!()
    }

    fn read(&self) -> () {
        todo!()
    }

    fn write(&self) -> () {
        todo!()
    }

    fn lseek(&self) -> () {
        todo!()
    }
}
