use super::super::partition::Partition;
use crate::sound;
use crate::{data_storage::path::Path, print, warningln};

use alloc::vec::Vec;

pub struct SoundDriver;

impl SoundDriver {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SoundDriver {
    fn default() -> Self {
        Self::new()
    }
}

fn u8slice_to_u64(slice: &[u8]) -> u64 {
    assert_eq!(slice.len(), 8);
    let mut res = slice[7] as u64;
    res <<= 8;
    res += slice[6] as u64;
    res <<= 8;
    res += slice[5] as u64;
    res <<= 8;
    res += slice[4] as u64;
    res <<= 8;
    res += slice[3] as u64;
    res <<= 8;
    res += slice[2] as u64;
    res <<= 8;
    res += slice[1] as u64;
    res <<= 8;
    res += slice[0] as u64;
    res
}

impl Partition for SoundDriver {
    fn read(&self, _path: &Path, _offset: usize, _size: usize) -> Vec<u8> {
        warningln!("User-program attempted to read in sound.");
        Vec::new()
    }

    fn write(&mut self, _path: &Path, _buffer: &[u8], offset: usize, flags: u64) -> isize {
        let sound_number = _buffer.len() / (3 * 8);
        // Each sound packet is 3 u64
        print!("Got a sound, {}", _buffer.len());
        for i in 0..sound_number {
            let tone = u8slice_to_u64(&_buffer[i * (3 * 8)..i * (3 * 8) + 8]);
            let length = u8slice_to_u64(&_buffer[i * (3 * 8) + 8..i * (3 * 8) + 16]);
            let begin = u8slice_to_u64(&_buffer[i * (3 * 8) + 16..i * (3 * 8) + 24]);
            sound::add_sound(tone, length, begin);
            print!("Added a sound");
        }
        (sound_number * (3 * 8)) as isize
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
