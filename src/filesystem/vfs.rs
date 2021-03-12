use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::partition::Partition;

use crate::data_storage::path::Path;

struct PartitionTree {
    root: PartitionNode,
}

enum PartitionNode {
    Node(BTreeMap<String, Box<PartitionNode>>),
    Leaf(Box<dyn Partition>),
}

struct VFS {
    //partitions: BTreeMap<String, Box<dyn Partition>>,
    partitions: PartitionTree,
}

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
                    return next_part.get_partition(sliced_path, index + 1);
                } else {
                    return Err(());
                }
            }
            PartitionNode::Leaf(part) => return Ok(part),
        }
    }
}

impl VFS {
    /// Returns the index of file descriptor. -1 if error
    fn open(&'static mut self, path: Path) -> isize {
        let sliced = path.slice();
        let res_partition = self.partitions.root.get_partition(sliced, 0);
        // If the VFS couldn't find the corresponding partition, return -1
        if res_partition.is_err() {
            return -1;
        }
        let partition = res_partition.unwrap();
        todo!()
    }

    fn close(&self) -> () {
        todo!()
    }

    fn read(&self) {
        todo!()
    }

    fn write(&self) {
        todo!()
    }

    fn lseek(&self) -> () {
        todo!()
    }
}
