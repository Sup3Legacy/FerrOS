#![allow(clippy::upper_case_acronyms)]

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::partition::Partition;

use super::drivers::nopart::NoPart;

use crate::data_storage::path::Path;

#[derive(Debug)]
pub struct ErrVFS();

/// Root of the partition-tree
struct PartitionTree {
    root: PartitionNode,
}

enum PartitionNode {
    Node(BTreeMap<String, Box<PartitionNode>>),
    Leaf(Box<dyn Partition>),
}

/// Main data structure of the FileSystem.
/// The [`VFS`] is a key component in the virtualization of
/// storage ressources. It can treansparently handle each and
/// every type of partition (RAM-Disk, ustar on ATA drive, raw device)
/// while having a global unified file-tree.
///
/// It holds a partition-tree :
/// This tree stores the hierarchy of partition in the global filesystem.
///
/// For example, the `/proc/` folder can be mounted to a RAM-Disk for
/// increased efficiency, while `/usr/`, `/bin/`, etc. get mounted
/// to a physical drive for bulk storage capacity.
pub struct VFS {
    /// partitions: BTreeMap<String, Box<dyn Partition>>,
    partitions: PartitionTree,
}

/// This immplementation takes care of all operations needed on the partition-tree,
/// such as the recursive search for the partition given a certain path.
impl PartitionNode {
    /// Returns reference to partition containing the path's target
    pub fn get_partition(
        &'static mut self,
        sliced_path: Vec<String>,
        index: usize,
    ) -> Result<(&'static mut Box<dyn Partition>, Path), ()> {
        match self {
            PartitionNode::Node(next) => {
                if index >= sliced_path.len() {
                    return Err(());
                }
                if let Some(next_part) = next.get_mut(&sliced_path[index]) {
                    next_part.get_partition(sliced_path, index + 1)
                } else {
                    Err(())
                }
            }
            PartitionNode::Leaf(part) => Ok((part, Path::from_sliced(&sliced_path[index..]))),
        }
    }

    pub fn add_entry(
        &mut self,
        sliced_path: Vec<String>,
        index: usize,
        data: Box<dyn Partition>,
    ) -> Result<(), ErrVFS> {
        match self {
            PartitionNode::Node(next) => {
                if index == sliced_path.len() {
                    return Err(ErrVFS());
                }
                if next.get(&sliced_path[index]).is_some() {
                    if let Some(next_part) = next.get_mut(&sliced_path[index]) {
                        next_part.add_entry(sliced_path, index + 1, data)
                    } else {
                        panic!("should not happen")
                    }
                } else {
                    let mut tree = PartitionNode::Leaf(data);
                    let mut i2 = sliced_path.len() - 1;
                    while i2 > index {
                        let mut map = BTreeMap::new();
                        map.insert(String::from(sliced_path[i2].as_str()), Box::new(tree));
                        tree = PartitionNode::Node(map);
                        i2 -= 1;
                    }
                    next.insert(String::from(sliced_path[index].as_str()), Box::new(tree));
                    Ok(())
                }
            }
            PartitionNode::Leaf(_) => Err(ErrVFS()),
        }
    }
}

/// This should be the main interface of the filesystem.
/// Still it will need some work besides this implementation,
/// as we need to implement all structures of file descriptors, etc.
impl VFS {
    /// Returns the index of file descriptor. -1 if error
    pub fn open(&'static mut self, path: Path) -> Result<(), ErrVFS> {
        let sliced = path.slice();
        let res_partition = self.partitions.root.get_partition(sliced, 0);
        // If the VFS couldn't find the corresponding partition, return -1
        if res_partition.is_err() {
            return Err(ErrVFS());
        }
        let _partition = res_partition.unwrap();
        todo!()
    }

    pub fn add_file(&mut self, path: Path, data: Box<dyn Partition>) -> Result<(), ErrVFS> {
        let sliced = path.slice();
        self.partitions.root.add_entry(sliced, 0, data)
    }

    pub fn close(&self) {
        todo!()
    }

    pub fn read(&'static mut self, path: Path, offset: usize, length: usize) -> Vec<u8> {
        let sliced = path.slice();
        let res_partition = self.partitions.root.get_partition(sliced, 0);
        // TODO check it actuallye returned something
        let (partition, _remaining_path) = res_partition.unwrap();
        partition.read(path, offset, length)
    }

    pub fn write(&'static mut self, path: Path, data: Vec<u8>) -> usize {
        let sliced = path.slice();
        let res_partition = self.partitions.root.get_partition(sliced, 0);
        // TODO check it actuallye returned something
        let (partition, remaining_path) = res_partition.unwrap();
        partition.write(remaining_path, data)
    }

    pub fn lseek(&self) {
        todo!()
    }

    pub fn new() -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            String::from("null"),
            Box::new(PartitionNode::Leaf(Box::new(NoPart::new()))),
        );
        let parts = PartitionTree {
            root: PartitionNode::Node(map),
        };
        Self { partitions: parts }
    }
}
impl Default for VFS{
    fn default() -> Self {
        Self::new()
    }
}
