use super::super::partition::Partition;

use crate::scheduler;
use crate::scheduler::process;
use crate::{data_storage::path::Path, warningln};

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Drives the `proc` repertory
pub struct ProcDriver {
    infos: BTreeMap<String, ProcInfoDriver>,
}

pub struct ErrProc();

impl ProcDriver {
    pub fn new() -> Self {
        Self {
            infos: BTreeMap::new(),
        }
    }
    pub fn get_info(&self, id: &str) -> Result<&ProcInfoDriver, ErrProc> {
        if let Some(res) = self.infos.get(id) {
            return Ok(res);
        }
        Err(ErrProc())
    }
}
impl Default for ProcDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl Partition for ProcDriver {
    fn open(&mut self, path: &Path) -> Option<usize> {
        if path.len() != 0 {
            None
        } else {
            Some(0)
        }
    }

    #[allow(clippy::if_same_then_else)]
    #[allow(clippy::len_zero)]
    fn read(&mut self, path: &Path, _id: usize, _offset: usize, _size: usize) -> Vec<u8> {
        let sliced = path.slice();
        if sliced.len() == 2 {
            if let Ok(proc) = sliced[0].parse::<usize>() {
                if let Ok(pi) = self.get_info(&sliced[1]) {
                    let func = pi.function;
                    return func(proc);
                } else {
                    return Vec::new();
                }
            }
        } else if sliced.len() == 1 {
            // Means we access the directory of a process
        } else if sliced.len() == 0 {
            // Means we access the main proc directory
        } else {
            panic!("Oscoure");
        }
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
        warningln!("User-program attempted to write in proc.");
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

    fn give_param(&mut self, _path: &Path, _id: usize, _param: usize) -> usize {
        usize::MAX
    }
}

/// Drives a single file in a `proc/pid` repertory
pub struct ProcInfoDriver {
    /// Name of the virtual file
    keyword: String,
    /// Handling function
    function: fn(usize) -> Vec<u8>,
}

fn heap(proc: usize) -> Vec<u8> {
    if proc as u64 >= scheduler::PROCESS_MAX_NUMBER {
        return Vec::new();
    }
    let process = unsafe { process::get_process(proc) };
    let str = format!("{} {}", process.heap_address, process.heap_size);
    str.as_bytes().to_vec()
}

fn screen(_proc: usize) -> Vec<u8> {
    todo!()
}
