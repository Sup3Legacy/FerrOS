//! All the logic of the filesystem, ranging from the drivers to the `VFS` pipeline

#![allow(dead_code)]

use crate::data_storage::path::Path;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use core::todo;

pub mod descriptor;
pub mod drivers;
pub mod fifo;
pub mod fsflags;
pub mod host_shell;
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
use fsflags::OpenFlags;
use partition::IoError;

pub static mut VFS: Option<VFS> = None;

/// Initializes the VFS with the basic filetree and partitions.
/// TODO
/// # Safety
/// TODO
pub unsafe fn init_vfs() {
    VFS = Some(VFS::new());
    if let Some(vfs) = &mut VFS {
        let s1 = screen_partition::ScreenPartition::new();
        vfs.add_file(Path::from("/hard/screen"), Box::new(s1))
            .expect("could not create screen");
        let s2 = drivers::clock_driver::ClockDriver::new();
        vfs.add_file(Path::from("/hard/clock"), Box::new(s2))
            .expect("could not create clock driver.");
        let s3 = drivers::mouse_driver::MouseDriver::new();
        vfs.add_file(Path::from("/hard/mouse"), Box::new(s3))
            .expect("could not create mouse driver.");
        let s4 = drivers::sound::SoundDriver::new();
        vfs.add_file(Path::from("/hard/sound"), Box::new(s4))
            .expect("could not create sound driver.");
        let s5 = host_shell::HostShellPartition::new();
        vfs.add_file(Path::from("/hard/host"), Box::new(s5))
            .expect("could not create shell printer.");

        let s6 = fifo::FiFoPartition::new();
        vfs.add_file(Path::from("/dev/fifo"), Box::new(s6))
            .expect("could not create fifo.");

        let s7 = drivers::proc::ProcDriver::new();
        vfs.add_file(Path::from("/proc"), Box::new(s7))
            .expect("could not create proc.");

        let s8 = drivers::keyboard::KeyBoard::new();
        vfs.add_file(Path::from("/hard/kbd"), Box::new(s8))
            .expect("could not create keyboard.");

        let s9 = ustar::UsTar::new();
        vfs.add_file(Path::from("/usr"), Box::new(s9))
            .expect("could not create disk driver.");
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

pub fn open_mode_from_flags(_flags: usize) -> OpenMode {
    OpenMode::Read
}

/// Main interface of the filesystem.
///
/// Every interaction of a user-program with hardware and/or
/// its stdin/stdout/stderr goes through this abstracted interface.
pub fn open_file(path: &Path, mode: OpenFlags) -> Result<usize, vfs::ErrVFS> {
    unsafe {
        if let Some(ref mut vfs) = VFS {
            vfs.open(path, mode)
        } else {
            panic!("VFS not initialized in open_file. {}", path.to());
        }
    }
}

pub fn write_file(oft: &OpenFileTable, data: Vec<u8>) -> usize {
    unsafe {
        if let Some(ref mut vfs) = VFS {
            vfs.write(oft, data) as usize
        } else {
            panic!("VFS not initialized in write_file.");
        }
    }
}

pub fn read_file(oft: &mut OpenFileTable, length: usize) -> Result<Vec<u8>, IoError> {
    unsafe {
        if let Some(ref mut vfs) = VFS {
            match vfs.read(oft, length) {
                Ok(res) => {
                    oft.add_offset(res.len());
                    Ok(res)
                }
                Err(err) => Err(err),
            }
        } else {
            panic!("VFS not initialized in read_file.");
        }
    }
}

pub fn read_file_from_path(path: Path) -> Result<Vec<u8>, IoError> {
    unsafe {
        if let Some(ref mut vfs) = VFS {
            let oft = OpenFileTable::new(path, fsflags::OpenFlags::OXCUTE as usize, usize::MAX);
            match vfs.read(&oft, usize::MAX) {
                Ok(res) => Ok(res),
                Err(err) => Err(err),
            }
        } else {
            panic!("VFS not initialized in read_file.");
        }
    }
}

pub fn modify_file(oft: &OpenFileTable, param: usize) -> usize {
    unsafe {
        if let Some(ref mut vfs) = VFS {
            vfs.give_param(oft, param)
        } else {
            panic!("VFS not initialized in read_file.");
        }
    }
}

/*pub fn duplicate_file(oft: &OpenFileTable) -> Option<usize> {
    unsafe {
        let path = oft.get_path();
        if let Some(ref mut vfs) = VFS {
            vfs.duplicate(path, oft.get_id())
        } else {
            panic!("VFS not initialized in duplicate_file")
        }
    }
}*/

pub fn get_data(_path: Path) -> &'static [u8] {
    todo!()
}

fn test() {
    println!(
        "{:?}",
        open_file(&Path::from(&String::from("test")), OpenFlags::ORDO)
    );
}

pub fn close_file(oft: &OpenFileTable) {
    unsafe {
        if let Some(ref mut vfs) = VFS {
            vfs.close(oft).expect("Unexisting file to close");
        } else {
            panic!("VFS not initialized in close_file.");
        }
    }
}
