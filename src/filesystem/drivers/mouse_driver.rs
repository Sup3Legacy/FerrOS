use super::super::partition::Partition;
use crate::hardware::mouse;
use crate::{data_storage::path::Path, warningln};

use alloc::vec::Vec;

pub struct MouseDriver;

impl MouseDriver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MouseDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl Partition for MouseDriver {
    fn open(&mut self, _path: &Path) -> usize {
        0
    }

    fn read(&mut self, _path: &Path, _id: usize, _offset: usize, _size: usize) -> Vec<u8> {
        // The number of packets to be written into the buffer
        let packet_number = _size / 3;
        let mut res = Vec::new();
        for _ in 0..packet_number {
            if let Some(packet) = mouse::get_packet() {
                let (p1, p2, p3) = packet.to_bytes();
                res.push(p1);
                res.push(p2);
                res.push(p3);
            } else {
                break;
            }
        }
        res
    }

    fn write(&mut self, _path: &Path, _id: usize, _buffer: &[u8], offset: usize, _flags: u64) -> isize {
        warningln!("User-program attempted to write in mouse.");
        -1
    }

    fn close(&mut self, _path: &Path, _id: usize) -> bool {
        false
    }

    fn duplicate(&mut self, _path: &Path, _id: usize) -> Option<usize> {
        Some(0)
    }

    fn lseek(&self) {
        todo!()
    }

    fn flush(&self) {
        todo!()
    }

    fn read_raw(&self) {
        todo!()
    }
}
