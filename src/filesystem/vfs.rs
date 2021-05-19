//! Heart of the kernel-user interraction. Defines the logic used by the `VFS`

#![allow(clippy::upper_case_acronyms)]

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::descriptor::OpenFileTable;
use super::fsflags::OpenFlags;
use super::partition::{IoError, Partition};

use crate::{data_storage::path::Path, warningln};

#[derive(Debug)]
pub struct ErrVFS();

/// Root of the partition-tree
pub struct VFS {
    depth: usize,
    subfiles: PartitionNode,
}

enum PartitionNode {
    Node(BTreeMap<String, VFS>),
    Leaf(Box<dyn Partition>),
}

impl Partition for VFS {
    fn open(&mut self, path: &Path, flags: OpenFlags) -> Option<usize> {
        let sliced = path.slice();
        warningln!("O {:?} {:?} depth: {}", sliced, path, self.depth);
        match &mut self.subfiles {
            PartitionNode::Leaf(part) => {
                let path2 = Path::from_sliced(&sliced[self.depth..]);
                part.open(&path2, flags)
            }
            PartitionNode::Node(map) => {
                if self.depth == sliced.len()
                    || (sliced.len() == self.depth + 1 && sliced[self.depth].is_empty())
                {
                    Some(0)
                } else {
                    match map.get_mut(&sliced[self.depth]) {
                        None => None,
                        Some(next) => next.open(path, flags),
                    }
                }
            }
        }
    }

    fn read(&mut self, oft: &OpenFileTable, size: usize) -> Result<Vec<u8>, IoError> {
        let sliced = oft.get_path().slice();
        match &mut self.subfiles {
            PartitionNode::Leaf(part) => {
                let path = Path::from_sliced(&sliced[self.depth..]);
                part.read(&oft.with_new_path(path), size)
            }
            PartitionNode::Node(map) => {
                if self.depth == sliced.len()
                    || (sliced.len() == self.depth + 1 && sliced[self.depth].is_empty())
                {
                    let mut v = Vec::new();
                    for key in map.keys().skip(oft.get_offset() / 28) {
                        if v.len() + 28 > size {
                            return Ok(v);
                        }
                        for l in key.bytes() {
                            v.push(l)
                        }
                        for _ in key.len()..28 {
                            v.push(b' ')
                        }
                        v.push(b'\n');
                    }
                    Ok(v)
                } else {
                    match map.get_mut(&sliced[self.depth]) {
                        None => Err(IoError::Kill),
                        Some(next) => next.read(oft, size),
                    }
                }
            }
        }
    }

    fn write(&mut self, oft: &OpenFileTable, buffer: &[u8]) -> isize {
        let sliced = oft.get_path().slice();
        match &mut self.subfiles {
            PartitionNode::Leaf(part) => {
                let path = Path::from_sliced(&sliced[self.depth..]);
                part.write(&oft.with_new_path(path), buffer)
            }
            PartitionNode::Node(map) => {
                if self.depth == sliced.len()
                    || (sliced.len() == self.depth + 1 && sliced[self.depth].is_empty())
                {
                    -1
                } else {
                    match map.get_mut(&sliced[self.depth]) {
                        None => -1,
                        Some(next) => next.write(oft, buffer),
                    }
                }
            }
        }
    }

    /// Flushes all changes to a file
    fn flush(&self) {
        todo!()
    }

    /// TODO
    fn lseek(&self) {
        todo!()
    }

    /// This is the function the kernel reads through.
    fn read_raw(&self) {
        todo!()
    }

    fn close(&mut self, oft: &OpenFileTable) -> bool {
        let sliced = oft.get_path().slice();
        match &mut self.subfiles {
            PartitionNode::Leaf(part) => {
                let path = Path::from_sliced(&sliced[self.depth..]);
                part.close(&oft.with_new_path(path));
                false
            }
            PartitionNode::Node(map) => {
                if self.depth == sliced.len()
                    || (sliced.len() == self.depth + 1 && sliced[self.depth].is_empty())
                {
                    false
                } else {
                    match map.get_mut(&sliced[self.depth]) {
                        None => false,
                        Some(next) => {
                            next.close(oft);
                            false
                        }
                    }
                }
            }
        }
    }

    /// Param
    fn give_param(&mut self, oft: &OpenFileTable, param: usize) -> usize {
        let sliced = oft.get_path().slice();
        match &mut self.subfiles {
            PartitionNode::Leaf(part) => {
                let path = Path::from_sliced(&sliced[self.depth..]);
                part.give_param(&oft.with_new_path(path), param)
            }
            PartitionNode::Node(map) => {
                if self.depth == sliced.len()
                    || (sliced.len() == self.depth + 1 && sliced[self.depth].is_empty())
                {
                    usize::MAX
                } else {
                    match map.get_mut(&sliced[self.depth]) {
                        None => usize::MAX,
                        Some(next) => next.give_param(oft, param),
                    }
                }
            }
        }
    }
}

impl VFS {
    pub fn add_file(&mut self, path: Path, data: Box<dyn Partition>) -> Result<(), ErrVFS> {
        let sliced = path.slice();
        warningln!("C {:?} {:?} depth: {}", sliced, path, self.depth);
        if self.depth == sliced.len() {
            match &self.subfiles {
                PartitionNode::Leaf(_) => Err(ErrVFS()),
                PartitionNode::Node(map) => {
                    if map.is_empty() {
                        self.subfiles = PartitionNode::Leaf(data);
                        Ok(())
                    } else {
                        Err(ErrVFS())
                    }
                }
            }
        } else {
            match &mut self.subfiles {
                PartitionNode::Leaf(_) => Err(ErrVFS()),
                PartitionNode::Node(map) => match map.get_mut(&sliced[self.depth]) {
                    None => {
                        let mut next = VFS {
                            depth: self.depth + 1,
                            subfiles: PartitionNode::Node(BTreeMap::new()),
                        };
                        let res = next.add_file(path, data);
                        map.insert(String::from(&sliced[self.depth]), next);
                        res
                    }
                    Some(next) => next.add_file(path, data),
                },
            }
        }
    }
    pub fn new() -> Self {
        Self {
            depth: 0,
            subfiles: PartitionNode::Node(BTreeMap::new()),
        }
    }
}
impl Default for VFS {
    fn default() -> Self {
        Self::new()
    }
}
