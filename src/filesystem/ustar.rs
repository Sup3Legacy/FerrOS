use super::disk_operations;
use alloc::vec::Vec;
use core::{mem::transmute, todo};
use lazy_static::lazy_static;

/// Max number of blocks usable in short mode
const SHORT_MODE_LIMIT: u32 = 100;

lazy_static! {
    /// Main table of available tables
    static ref LBA_TABLE: LBATable = {
        disk_operations::init();
        let mut res = LBATable::from_u16_array(disk_operations::read_sector(1));
        res.data[0] = false;
        res.data[1] = false;
        res
    };
}

lazy_static! {
    /// Index of lowest free sector. Useful to keep track of to avoid useless computations.
    static ref LBA_TABLE_INDEX: u32 = 2;
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum FileType {
    Available = 10,
    File = 0,
    Directory = 1,
}
/*
- 2 octets de flags/autorisation (u:rwxs, g:rwxs, o:rwxs, opened, ...)
- 100 octets de nom (on peut diminuer au besoin)
- 1 octet de type
- 8 octets (ie 64 bits) de user ID
- 8 octets (ie 64 bits) de group ID
- adresse du dossier parent?/ nom du dossier parent?
*/

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HeaderFlags {
    user_owner: u8,
    group_misc: u8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Type {
    Dir = 0 as u8,
    File = 1 as u8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum FileMode {
    Short = 0 as u8,
    Long = 1 as u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UGOID(u64);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    file_type: Type,    // 1 byte
    flags: HeaderFlags, // 2 bytes
    name: [u8; 32],     // 100 bytes
    user: UGOID,        // 8 bytes
    owner: UGOID,       // 8 bytes
    group: UGOID,       // 8 bytes
    parent_adress: u32, // 8 bytes
    length: u32,        // 8 bytes. In case of a header, it is the number of sub-items.
    mode: FileMode, // If Short then we list all blocks. Else each block contains the adresses of the data blocks.
    padding: [u32; 9], // Padding to have a nice SHORT_MODE_LIMIT number
    blocks: [u32; SHORT_MODE_LIMIT as usize],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MemFile {
    header: Header,
    data: Vec<u16>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DirBlock {
    subitems: [[u8; 32]; 16],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FileBlock {
    data: [u8; 512],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LBATable {
    data: [bool; 512],
}

impl MemFile {
    pub fn write_to_disk(&self) -> () {
        // Might want to Result<(), SomeError>
        let mut file_header = self.header;
        let length = file_header.length;
        if length < SHORT_MODE_LIMIT * 256 {
            // i.e. file short enough to use short mode
            file_header.mode = FileMode::Short;
        } else {
            file_header.mode = FileMode::Long;
            todo!()
        }
    }
}

impl U16Array for Header {
    fn to_u16_array(&self) -> [u16; 256] {
        return unsafe { transmute::<Header, [u16; 256]>(*self) };
    }

    fn from_u16_array(array: [u16; 256]) -> Self {
        return unsafe { transmute::<[u16; 256], Header>(array) };
    }
}

impl U16Array for FileBlock {
    fn to_u16_array(&self) -> [u16; 256] {
        return unsafe { transmute::<FileBlock, [u16; 256]>(*self) };
    }

    fn from_u16_array(array: [u16; 256]) -> Self {
        return unsafe { transmute::<[u16; 256], FileBlock>(array) };
    }
}

impl U16Array for DirBlock {
    fn to_u16_array(&self) -> [u16; 256] {
        return unsafe { transmute::<DirBlock, [u16; 256]>(*self) };
    }

    fn from_u16_array(array: [u16; 256]) -> Self {
        return unsafe { transmute::<[u16; 256], DirBlock>(array) };
    }
}

impl U16Array for LBATable {
    fn to_u16_array(&self) -> [u16; 256] {
        return unsafe { transmute::<LBATable, [u16; 256]>(*self) };
    }

    fn from_u16_array(array: [u16; 256]) -> Self {
        return unsafe { transmute::<[u16; 256], LBATable>(array) };
    }
}

pub trait U16Array {
    fn to_u16_array(&self) -> [u16; 256];

    fn from_u16_array(array: [u16; 256]) -> Self;
}

pub fn write_to_disk(data: impl U16Array, lba: u32) {
    disk_operations::write_sector(&data.to_u16_array(), lba);
}

pub fn read_from_disk<T: U16Array>(lba: u32) -> T {
    T::from_u16_array(disk_operations::read_sector(lba))
}
