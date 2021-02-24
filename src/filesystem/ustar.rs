use super::disk_operations;
use alloc::vec::Vec;
use core::{mem::transmute, todo};
use disk_operations::write_sector;
use lazy_static::lazy_static;

const LBA_TABLE_POSITION: u32 = 1;
/// Max number of blocks usable in short mode
const SHORT_MODE_LIMIT: u32 = 100;

lazy_static! {
    /// Main table of available tables
    static ref LBA_TABLE: LBATable = {
        disk_operations::init();
        let mut res = LBATable::load_from_disk(LBA_TABLE_POSITION);
        res.data[0] = false;
        res.data[1] = false;
        res
    };
}

static mut LBA_TABLE_INDEX: u32 = 2;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Dir = 0 as u8,
    File = 1 as u8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileMode {
    Short = 0 as u8,
    Long = 1 as u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    blocks_number : u32,
    mode: FileMode, // If Short then we list all blocks. Else each block contains the adresses of the data blocks.
    padding: [u32; 8], // Padding to have a nice SHORT_MODE_LIMIT number
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
    data: [u16; 256],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LBATable {
    data: [bool; 512],
}

impl LBATable {
    fn init(&mut self) {
        for i in 0..512 {
            self.data[i] = true;
        }
        self.data[0] = false;
        self.data[1] = false;
    }
    fn load_from_disk(lba: u32) -> Self {
        LBATable::from_u16_array(disk_operations::read_sector(lba))
    }
    fn write_to_disk(&self) {
        disk_operations::write_sector(&self.to_u16_array(), 1);
    }
    pub fn is_available(&self, i: u32) -> bool {
        self.data[i as usize]
    }
    pub fn mark_available(&mut self, i: u32) {
        self.data[i as usize] = true;
    }
    pub fn mark_unavailable(&mut self, i: u32) {
        self.data[i as usize] = false;
    }
}

fn slice_vec(data: &Vec<u16>) -> Vec<[u16; 256]> {
    let n = data.len();
    let block_number = n / 256 + (if n % 256 > 0 { 1 } else { 0 });
    let mut res: Vec<[u16; 256]> = Vec::new();
    let mut index = 0;
    for i in 0..block_number {
        let mut arr = [0 as u16; 256];
        for j in 0..256 {
            if index >= n {
                break;
            }
            arr[j] = data[index];
            index += 1;
        }
        res[i] = arr;
    }
    res
}

impl MemFile {
    pub fn write_to_disk(&self) -> () {
        // Might want to Result<(), SomeError>
        let mut file_header = self.header;
        let length = file_header.length; // TODO : make sure it is also the length of self.data
        if length < SHORT_MODE_LIMIT * 256 {
            file_header.mode = FileMode::Short;
            let mut block_adresses: Vec<u32> = Vec::new();
            for _ in 0..file_header.blocks_number {
                block_adresses.push(0);
            }
            let mut indice = 0;
            unsafe {
                while indice < file_header.blocks_number {
                    if LBA_TABLE.is_available(LBA_TABLE_INDEX) {
                        block_adresses[indice as usize] = LBA_TABLE_INDEX;
                        indice += 1;
                    }
                    LBA_TABLE_INDEX += 1;
                }
            }
            let mut adresses = [0 as u32; SHORT_MODE_LIMIT as usize];
            for i in 1..(file_header.blocks_number as usize) {
                adresses[i - 1] = block_adresses[i];
            }
            file_header.blocks = adresses;
            write_to_disk(file_header, block_adresses[0]);
            let blocks_to_write = slice_vec(&self.data);
            for i in 0..(file_header.blocks_number - 1) {
                let file_block = FileBlock {
                    data: blocks_to_write[i as usize],
                };
                write_to_disk(file_block, block_adresses[(i + 1) as usize]);
            }
        } else {
            file_header.mode = FileMode::Long;
            todo!()
        }
        LBA_TABLE.write_to_disk();
    }
    pub fn read_from_disk(lba: u32) -> Self {
        let header: Header = read_from_disk(lba);
        let mut file = Self {
            header,
            data: Vec::new(),
        };
        if header.mode == FileMode::Short {
            for i in 0..header.blocks_number {
                let sector : FileBlock = read_from_disk(header.blocks[i as usize]);
                for j in 0..256 {
                    file.data.push(sector.data[j]);
                }
            }
        } else {
            todo!()
        }
        file
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
