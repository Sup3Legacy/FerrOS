use crate::data_storage::path::Path;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::todo;
use lazy_static::lazy_static;
pub mod drivers;
pub mod fsflags;
pub mod partition;
pub mod test;
pub mod vfs;

// disk_operations here is only temporary.
// TODO remove it and add interface in driver::ustar
pub use drivers::{disk_operations, ustar};
pub use vfs::VFS;

use crate::{print, println};

static mut VFS: VFS = VFS::new();

/// Initializes the VFS with the basic filetree and partitions.
/// TODO
fn init_vfs() {}

#[derive(Debug, PartialEq)]
pub struct FileSystemError(String);

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum OpenMode {
    Read = 0b00000000,
    Write = 0b00000001,
    Execute = 0b00000010,
}

pub fn open_file(_path: Path, _mode: OpenMode) -> &'static [u8] {
    todo!();
}

pub fn write_file(_path: Path, _data: &'static [u8]) {
    todo!();
}

fn test() {
    println!(
        "{:?}",
        open_file(Path::from(&String::from("test")), OpenMode::Read)
    );
}
