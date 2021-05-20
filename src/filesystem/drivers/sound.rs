use super::super::partition::{IoError, Partition};
use crate::filesystem::descriptor::OpenFileTable;
use crate::filesystem::fsflags::OpenFlags;
use crate::sound;
use crate::{data_storage::path::Path};

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
    fn open(&mut self, path: &Path, _flags: OpenFlags) -> Option<usize> {
        if !path.is_empty() {
            None
        } else {
            Some(0)
        }
    }

    fn read(&mut self, _oft: &OpenFileTable, _size: usize) -> Result<Vec<u8>, IoError> {
        Err(IoError::Kill)
    }

    fn write(&mut self, _oft: &OpenFileTable, buffer: &[u8]) -> isize {
        let sound_number = buffer.len() / (3 * 8);
        // Each sound packet is 3 u64
        for i in 0..sound_number {
            let tone = u8slice_to_u64(&buffer[i * (3 * 8)..i * (3 * 8) + 8]);
            let length = u8slice_to_u64(&buffer[i * (3 * 8) + 8..i * (3 * 8) + 16]);
            let begin = u8slice_to_u64(&buffer[i * (3 * 8) + 16..i * (3 * 8) + 24]);
            sound::add_sound(tone, length, begin);
        }
        (sound_number * (3 * 8)) as isize
    }

    fn close(&mut self, _oft: &OpenFileTable) -> bool {
        false
    }

    /*fn duplicate(&mut self, _oft: &OpenFileTable) -> Option<usize> {
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
