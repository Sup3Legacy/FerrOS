use super::super::partition::Partition;
use x86_64::instructions::port::Port;

pub struct Raw {
    port: u8,    // Should be a generic port.
    read: bool,  // Whether this raw periph can be read from
    write: bool, // Whether it can be written to
}

impl Partition for Raw {
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
}
