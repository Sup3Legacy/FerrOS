use super::super::partition::Partition;
use crate::data_storage::path::Path;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
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
/// (as opposed to a raw peripheral or a disk drive)
///
/// We simply store each file as a <TO DO> and keep track of it
/// via a BTreeMap, addressed by it's path
///
/// Thanks to the `Partition` trait, its behaviour is totally transparent
/// to the VFS and can be used for certain latency- and throughput-critical
/// operations as a process' `stdin`, `stdout` and `stderr`.
pub struct RamDisk {
    id: RamDiskID,
    files: BTreeMap<Path, [u8; 256]>, // TODO generalize MemFile structure
}

/// This interfaces enables a RAM-Disk to get used alongside every other device.
impl Partition for RamDisk {
    fn open(&mut self, _path: &Path) -> Option<usize> {
        todo!()
    }

    fn read(&mut self, _path: &Path, _id: usize, _offset: usize, _size: usize) -> Vec<u8> {
        todo!()
    }

    fn write(
        &mut self,
        _path: &Path,
        _id: usize,
        _buffer: &[u8],
        _offset: usize,
        _flags: u64,
    ) -> isize {
        todo!()
    }

    fn close(&mut self, _path: &Path, _id: usize) -> bool {
        todo!()
    }

    fn duplicate(&mut self, _path: &Path, _id: usize) -> Option<usize> {
        todo!()
    }

    fn flush(&self) {
        todo!()
    }

    fn lseek(&self) {
        todo!()
    }

    fn read_raw(&self) {
        todo!()
    }

    fn give_param(&mut self, _path: &Path, _id: usize, _param: usize) -> usize {
        usize::MAX
    }
}
