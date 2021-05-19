//! FIFO used for inter-process communication

use super::super::partition::{IoError, Partition};
use crate::data_storage::path::Path;
use crate::filesystem::descriptor::OpenFileTable;
use crate::filesystem::fsflags::OpenFlags;
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

    pub fn read(&mut self, size: usize, b: bool) -> Result<Vec<u8>, IoError> {
        let mut data = Vec::new();
        for _i in 0..size {
            match self.data.pop() {
                Err(PopError) => {
                    if b {
                        if data.is_empty() {
                            crate::warningln!("FiFo empty, now killing");
                            return Err(IoError::Kill);
                        } else {
                            crate::warningln!("FiFo gave end and is alone {}", data.len());
                            return Ok(data);
                        }
                    } else {
                        if !data.is_empty(){
                            crate::warningln!("FiFo gave end and not alone {}", data.len());
                        }
                        return Ok(data);
                    }
                }
                Ok(d) => data.push(d),
            }
        }
        crate::warningln!("FiFo not empty {}", data.len());
        Ok(data)
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

    pub fn len(&self) -> usize {
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
    fn open(&mut self, path: &Path, _fs: OpenFlags) -> Option<usize> {
        if !path.is_empty() {
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

    fn read(&mut self, oft: &OpenFileTable, size: usize) -> Result<Vec<u8>, IoError> {
        if !oft.get_path().is_empty() {
            return Err(IoError::Kill);
        }
        match &mut self.data[oft.get_id()] {
            None => Err(IoError::Kill),
            Some(fifo) => fifo.read(size, oft.get_amount() == 1),
        }
    }

    fn write(&mut self, oft: &OpenFileTable, buffer: &[u8]) -> isize {
        if !oft.get_path().is_empty() {
            return 0;
        }
        match &mut self.data[oft.get_id()] {
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

    /*fn duplicate(&mut self, _path: &Path, _id: usize) -> Option<usize> {
        None
    }*/

    fn close(&mut self, oft: &OpenFileTable) -> bool {
        if !oft.get_path().is_empty() {
            panic!("closed an unexisting file in fifo")
        }

        match &self.data[oft.get_id()] {
            None => crate::warningln!("Empty fifo"),
            Some(v) => crate::warningln!("Fifo of length {}", v.len()),
        }
        self.data[oft.get_id()] = None;
        false
    }

    fn give_param(&mut self, oft: &OpenFileTable, param: usize) -> usize {
        if !oft.get_path().is_empty() {
            panic!("give param to unexisting file")
        }
        match &mut self.data[oft.get_id()] {
            None => 0,
            Some(fifo) => fifo.give_param(param),
        }
    }
}
