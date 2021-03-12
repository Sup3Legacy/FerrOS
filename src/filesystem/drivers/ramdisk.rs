use core::sync::atomic::{AtomicU64, Ordering};
use super::super::partition::Partition;
#[derive(Copy, Clone, Debug)]
struct RamDiskID(u64);

impl RamDiskID {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let new = NEXT_ID.fetch_add(1, Ordering::Relaxed); // Maybe better to reallow previous numbers
        Self(new)
    }
}

pub struct RamDisk {}

impl Partition for RamDisk {
    fn open(&self) -> () {
        todo!()
    }

    fn close(&self) -> () {
        todo!()
    }

    fn read(&self) {
        todo!()
    }
    
    fn write(&self) {
        todo!()
    }

    fn lseek(&self) -> () {
        todo!()
    }
}