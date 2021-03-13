use super::super::partition::Partition;
use core::sync::atomic::{AtomicU64, Ordering};
#[derive(Copy, Clone, Debug)]
struct RamDiskID(u64);

/// Just an ID, because why not keeping track of all the RAM-Disk
/// existing at any given time.
impl RamDiskID {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let new = NEXT_ID.fetch_add(1, Ordering::Relaxed); // Maybe better to reallow previous numbers
        Self(new)
    }
}

/// A RAM-Disk is a data structure stored in RAM
/// (as opposed to a ras peripheral or a disk drive)
///
/// Thanks to the `Partition` trait, its behaviour is totally transparent
/// to the VFS and can be used for certain latency- and throughput-critical
/// operations as a process' `stdin`, `stdout` and `stderr`.
pub struct RamDisk {}

/// This interfaces enables a RAM-Disk to get used alongside every other device.
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
