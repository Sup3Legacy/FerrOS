use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use alloc::string::String;

use super::partition::Partition;

struct VFS {
    partitions : BTreeMap<String, Box<dyn Partition>>,
}

impl VFS {
    
}