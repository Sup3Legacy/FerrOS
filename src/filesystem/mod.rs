#![allow(dead_code)]

use crate::data_storage::path::Path;

use alloc::string::String;
use alloc::boxed::Box;

use core::todo;

pub mod descriptor;
pub mod drivers;
pub mod fsflags;
pub mod partition;
pub mod screen_partition;
pub mod test;
pub mod vfs;

// disk_operations here is only temporary.
// TODO remove it and add interface in driver::ustar
pub use drivers::{disk_operations, ustar};
pub use vfs::VFS;

use crate::println;

pub static mut VFS: Option<VFS> = None;

/// Initializes the VFS with the basic filetree and partitions.
/// TODO
pub unsafe fn init_vfs() {
    VFS = Some(VFS::new());
    if let Some(vfs) = &mut VFS {
        let s1 = screen_partition::ScreenPartition::new(0, 0, 20, 80, 0);
        vfs.add_file(Path::from("screen/screenfull"), Box::new(s1));
    } else {
        panic!("should not happen")
    }
}

#[derive(Debug, PartialEq)]
pub struct FileSystemError(String);

/// TO DO remove and use maybe use more general enum
/// found in [`fsflags`]
#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum OpenMode {
    Read = 0b00000000,
    Write = 0b00000001,
    Execute = 0b00000010,
}

/// Main interface of the filesystem.
///
/// Every interaction of a user-program with hardware and/or
/// its stdin/stdout/stderr goes through this abstracted interface.
pub fn open_file(_path: Path, _mode: OpenMode) -> &'static [u8] {
    todo!();
}

pub fn write_file(_path: Path, _data: &'static [u8]) {
    todo!();
}

pub fn get_data(path: Path) -> &'static [u8] {
    todo!()
}

fn test() {
    println!(
        "{:?}",
        open_file(Path::from(&String::from("test")), OpenMode::Read)
    );
}
