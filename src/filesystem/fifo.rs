use super::partition::Partition;

use crate::data_storage::path::Path;
use alloc::vec::Vec;
use crossbeam_queue::{ArrayQueue, PopError, PushError};

struct FiFoPartitionInner {
    data: ArrayQueue<u8>,
}

impl FiFoPartitionInner {
    pub fn new() -> Self {
        Self {
            data: ArrayQueue::new(1024),
        }
    }

    pub fn read(&mut self, size: usize) -> Vec<u8> {
        let mut data = Vec::new();
        for _i in 0..size {
            match self.data.pop() {
                Err(PopError) => return data,
                Ok(d) => data.push(d),
            }
        }
        data
    }

    pub fn write(&mut self, buffer: &[u8]) -> isize {
        let mut amount = 0;
        for item in buffer.iter() {
            match self.data.push(*item) {
                Ok(()) => amount += 1,
                Err(PushError(_d)) => return amount,
            }
        }
        amount
    }

    pub fn give_param(&mut self, p: usize) -> usize {
        for _i in 0..p {
            match self.data.pop() {
                Err(PopError) => return self.data.len(),
                Ok(_d) => (),
            }
        }
        self.data.len()
    }
}

pub struct FiFoPartition {
    data: Vec<Option<FiFoPartitionInner>>,
}

impl FiFoPartition {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}

impl Default for FiFoPartition {
    fn default() -> Self {
        Self::new()
    }
}

impl Partition for FiFoPartition {
    fn open(&mut self, path: &Path) -> Option<usize> {
        if path.len() != 0 {
            return None;
        }
        for i in 0..self.data.len() {
            if self.data[i].is_none() {
                self.data[i] = Some(FiFoPartitionInner::new());
                return Some(i);
            }
        }
        self.data.push(Some(FiFoPartitionInner::new()));
        Some(self.data.len() - 1)
    }

    fn read(&mut self, path: &Path, id: usize, _offset: usize, size: usize) -> Vec<u8> {
        if path.len() != 0 {
            return Vec::new();
        }
        match &mut self.data[id] {
            None => Vec::new(),
            Some(fifo) => fifo.read(size),
        }
    }

    fn write(
        &mut self,
        path: &Path,
        id: usize,
        buffer: &[u8],
        _offset: usize,
        _flags: u64,
    ) -> isize {
        if path.len() != 0 {
            return 0;
        }
        match &mut self.data[id] {
            None => 0,
            Some(fifo) => fifo.write(buffer),
        }
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

    fn duplicate(&mut self, _path: &Path, _id: usize) -> Option<usize> {
        None
    }

    fn close(&mut self, path: &Path, id: usize) -> bool {
        if path.len() != 0 {
            panic!("closed an unexisting file in fifo")
        }
        self.data[id] = None;
        false
    }

    fn give_param(&mut self, path: &Path, id: usize, param: usize) -> usize {
        if path.len() != 0 {
            panic!("give param to unexisting file")
        }
        match &mut self.data[id] {
            None => 0,
            Some(fifo) => fifo.give_param(param),
        }
    }
}
