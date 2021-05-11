#![allow(dead_code)]

use crate::data_storage::path::Path;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

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
use descriptor::OpenFileTable;

pub static mut VFS: Option<VFS> = None;

/// Initializes the VFS with the basic filetree and partitions.
/// TODO
/// # Safety
/// TODO
pub unsafe fn init_vfs() {
    VFS = Some(VFS::new());
    if let Some(vfs) = &mut VFS {
        let s1 = screen_partition::ScreenPartition::new();
        vfs.add_file(Path::from("screen/screenfull"), Box::new(s1))
            .expect("could not create screen");
        let s2 = drivers::clock_driver::ClockDriver::new();
        vfs.add_file(Path::from("hardware/clock"), Box::new(s2))
            .expect("could not create clock driver.");
        let s3 = drivers::mouse_driver::MouseDriver::new();
        vfs.add_file(Path::from("hardware/mouse"), Box::new(s3))
            .expect("could not create mouse driver.");
        let s4 = drivers::sound::SoundDriver::new();
        vfs.add_file(Path::from("hardware/sound"), Box::new(s4))
            .expect("could not create sound driver.");
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

pub fn write_file(oft: &OpenFileTable, data: Vec<u8>) -> usize {
    unsafe {
        let path = oft.get_path();
        if let Some(ref mut vfs) = VFS {
            vfs.write(path, data, oft.get_offset(), 0) as usize
        } else {
            panic!("VFS not initialized in write_file.");
        }
    }
}

pub fn read_file(oft: &OpenFileTable, length: usize) -> Vec<u8> {
    unsafe {
        let path = oft.get_path();
        let offset = oft.get_offset();
        if let Some(ref mut vfs) = VFS {
            vfs.read(path, offset, length)
        } else {
            panic!("VFS not initialized in read_file.");
        }
    }
}

pub fn get_data(_path: Path) -> &'static [u8] {
    todo!()
}

fn test() {
    println!(
        "{:?}",
        open_file(Path::from(&String::from("test")), OpenMode::Read)
    );
}

pub fn close_file(oft: &OpenFileTable) {
    unsafe {
        let path = oft.get_path();
        let offset = oft.get_offset();
        if let Some(ref mut vfs) = VFS {
            vfs.close(path);
        } else {
            panic!("VFS not initialized in close_file.");
        }
    }
}