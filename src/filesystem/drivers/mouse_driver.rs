use super::super::partition::{IoError, Partition};
use crate::filesystem::descriptor::OpenFileTable;
use crate::filesystem::fsflags::OpenFlags;
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
    fn open(&mut self, path: &Path, _flags: OpenFlags) -> Option<usize> {
        if path.len() != 0 {
            None
        } else {
            Some(0)
        }
    }

    fn read(&mut self, _oft: &OpenFileTable, _size: usize) -> Result<Vec<u8>, IoError> {
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
        Ok(res)
    }

    fn write(&mut self, _oft: &OpenFileTable, _buffer: &[u8]) -> isize {
        warningln!("User-program attempted to write in mouse.");
        -1
    }

    fn close(&mut self, _oft: &OpenFileTable) -> bool {
        false
    }

    /*fn duplicate(&mut self, _path: &Path, _id: usize) -> Option<usize> {
        Some(0)
    }*/

    fn lseek(&self) {
        todo!()
    }

    fn flush(&self) {
        todo!()
    }

    fn read_raw(&self) {
        todo!()
    }

    fn give_param(&mut self, _oft: &OpenFileTable, _param: usize) -> usize {
        usize::MAX
    }
}
