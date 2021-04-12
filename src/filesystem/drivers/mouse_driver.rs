use core::convert::TryInto;

use super::super::partition::Partition;
use crate::hardware::mouse;
use crate::{data_storage::path::Path, warningln};

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use x86_64::instructions::port::Port;

pub struct MouseDriver;

impl MouseDriver {
    pub fn new() -> Self {
        Self
    }
}

impl Partition for MouseDriver {
    fn read(&self, _path: Path, _offset: usize, _size: usize) -> Vec<u8> {
        // The number of packets to be written into the buffer
        let packet_number = _size / 3;
        let mut res = Vec::new();
        for i in 0..packet_number {
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

    fn write(&self, _path: Path, _buffer: Vec<u8>) -> usize {
        warningln!("User-program attempted to write in mouse.");
        0
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
