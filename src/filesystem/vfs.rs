#![allow(clippy::upper_case_acronyms)]

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::partition::Partition;

use super::drivers::nopart::NoPart;

use crate::data_storage::path::Path;

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
    partitions: Option<PartitionTree>,
}

/// This immplementation takes care of all operations needed on the partition-tree,
/// such as the recursive search for the partition given a certain path.
impl PartitionNode {
    /// Returns reference to partition containing the path's target
    pub fn get_partition(
        &'static mut self,
        sliced_path: Vec<String>,
        index: usize,
    ) -> Result<&'static mut Box<dyn Partition>, ()> {
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
            PartitionNode::Leaf(part) => Ok(part),
        }
    }
}

/// This should be the main interface of the filesystem.
/// Still it will need some work besides this implementation,
/// as we need to implement all structures of file descriptors, etc.
impl VFS {
    /// Returns the index of file descriptor. -1 if error
    pub fn open(&'static mut self, path: Path) -> isize {
        let sliced = path.slice();
        let res_partition = self
            .partitions
            .get_or_insert(PartitionTree {
                root: PartitionNode::Leaf(Box::new(NoPart::new())),
            })
            .root
            .get_partition(sliced, 0);
        // If the VFS couldn't find the corresponding partition, return -1
        if res_partition.is_err() {
            return -1;
        }
        let _partition = res_partition.unwrap();
        todo!()
    }

    pub fn close(&self) {
        todo!()
    }

    /// Returns the amount of bytes that were read into the buffer
    pub fn read(&self, _buffer: *mut usize) -> usize {
        todo!()
    }

    pub fn write(&self) {
        todo!()
    }

    pub fn lseek(&self) {
        todo!()
    }

    pub const fn new() -> Self {
        Self { partitions: None }
    }
}
