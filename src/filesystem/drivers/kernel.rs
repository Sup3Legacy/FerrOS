use super::super::partition::Partition;
use x86_64::instructions::port::Port;

pub struct Kernel {}

impl Partition for Kernel {
    fn read(&self) -> () {
        todo!()
    }

    fn write(&self) -> () {
        todo!()
    }

    fn lseek(&self) -> () {
        todo!()
    }

    fn flush(&self) {
        todo!()
    }

    fn read_raw(&self) -> () {
        todo!()
    }
}
