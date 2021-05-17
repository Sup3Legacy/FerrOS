use super::super::partition::{IoError, Partition};
use crate::filesystem::descriptor::OpenFileTable;
use crate::filesystem::fsflags::OpenFlags;

use crate::debug;
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
        let mut res = Self {
            infos: BTreeMap::new(),
        };
        res.infos.insert(
            String::from("heap"),
            ProcInfoDriver::new(String::from("heap"), heap_proc),
        );
        res
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
    fn open(&mut self, path: &Path, _flags: OpenFlags) -> Option<usize> {
        Some(0)
    }

    #[allow(clippy::if_same_then_else)]
    #[allow(clippy::len_zero)]
    fn read(&mut self, oft: &OpenFileTable, size: usize) -> Result<Vec<u8>, IoError> {
        let sliced = oft.get_path().clone().slice();
        debug!("proc got {:?},n {:?}", oft.get_path(), sliced);
        if sliced.len() == 2 {
            if let Ok(proc) = sliced[0].parse::<usize>() {
                if let Ok(pi) = self.get_info(&sliced[1]) {
                    let func = pi.function;
                    let mut res = func(proc);
                    // this utter mess makes sure what we hand away complies to the requested offset and size
                    res.truncate(oft.get_offset() + size);
                    res.reverse();
                    res.truncate(core::cmp::max(res.len() - oft.get_offset(), 0));
                    res.reverse();
                    return Ok(res);
                } else {
                    return Ok(Vec::new());
                }
            }
        } else if sliced.len() == 1 {
            // Means we access the directory of a process
            let keys = self.infos.keys();
            let mut res_array = Vec::new();
            for s in keys {
                let info_bytes = s.bytes();
                for b in info_bytes {
                    res_array.push(b)
                }
                res_array.push(b' ');
            }
            res_array.truncate(oft.get_offset() + size);
            res_array.reverse();
            res_array.truncate(core::cmp::max(res_array.len() - oft.get_offset(), 0));
            res_array.reverse();
            return Ok(res_array);
        } else if sliced.len() == 0 {
            // Means we access the main proc directory
            // We want to build the array of all alive processes
            let mut res_array = Vec::new();
            for (id, proc) in unsafe { scheduler::process::ID_TABLE.as_ref().iter().enumerate() } {
                match proc.state {
                    process::State::SlotAvailable => (),
                    _ => {
                        let id = format!("{}", id);
                        for b in id.bytes() {
                            res_array.push(b)
                        }
                        res_array.push(b' ');
                    }
                }
            }
            res_array.truncate(oft.get_offset() + size);
            res_array.reverse();
            res_array.truncate(core::cmp::max(res_array.len() - oft.get_offset(), 0));
            res_array.reverse();
            return Ok(res_array);
        } else {
            panic!("Oscoure");
        }
        todo!()
    }

    fn write(&mut self, _oft: &OpenFileTable, _buffer: &[u8]) -> isize {
        warningln!("User-program attempted to write in proc.");
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

/// Drives a single file in a `proc/pid` repertory
pub struct ProcInfoDriver {
    /// Name of the virtual file
    keyword: String,
    /// Handling function
    function: fn(usize) -> Vec<u8>,
}

impl ProcInfoDriver {
    pub fn new(kwd: String, func: fn(usize) -> Vec<u8>) -> Self {
        Self {
            keyword: kwd,
            function: func,
        }
    }
}

fn heap_proc(proc: usize) -> Vec<u8> {
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
