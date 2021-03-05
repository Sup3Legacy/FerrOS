use super::disk_operations;

use crate::{print, println};
use alloc::vec::Vec;
use core::{mem::transmute, todo};

// Number of 512-sector segments
const LBA_TABLES_COUNT: u32 = 1;

/// Max number of blocks usable in short mode
const SHORT_MODE_LIMIT: u32 = 100;

pub static mut LBA_TABLE_GLOBAL: LBATableGlobal = LBATableGlobal {
    index: 0,
    data: [LBATable {
        index: 0,
        data: [true; 510],
    }; LBA_TABLES_COUNT as usize],
};

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
- addresse du dossier parent?/ nom du dossier parent?
*/

/// Contains a file's flags.
///
/// * `user_owner` - 4 bits for the user's `rwxs` and 4 bits for the owner's one.
/// * `group_misc` - 4 bits for the group's `rwxs` and the rest for `opened`, etc.
///
/// TO DO : extend this header, because there is some space left in [`Header::padding`]
#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct HeaderFlags {
    pub user_owner: u8,
    pub group_misc: u8,
}

/// Type of a chunk of data.
///
/// Currently only `directory` and `file` but some other things like `Pipe` might be added.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Dir = 1_u8,
    File = 2_u8,
}

/// Specifies the mode of storage of the chunk of data.
///
/// * `Short` - When the chunk can be stored within `SHORT_MODE_LIMIT` sectors
///(that is the number of sectors whose address can fit inside the header),
/// We directly allocate these sectors and store their addresses inside the header.
/// * `Long` - When the chunk is too big, we allocate some sectors
/// which will hold all the addresses of the chunk's data's sectors.
/// These intermediate sectors get their addresses stored inside the header.
///
/// We thus effectively have two size limits : around `50kB` and around `26MB`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileMode {
    Short = 0_u8,
    Long = 1_u8,
}

/// A user's of group's or owner's ID. Might get replaced by a more general definition.
#[repr(packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UGOID(pub u64);

/// The address of a sector on the disk.
///
/// *Fields*
///
/// * `lba` - the index of the `LBA` the sector belongs to.
/// * `block` - its index within that `LBA`.
#[repr(packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address {
    pub lba: u16,
    pub block: u16, // Really only u8 needed
}

/// A chunk's header
///
/// *Fields$
///
/// * `file_type` - a [`Type`] indicating what type this chunk is ([`Type::Dir`] or [`Type::File`]).
/// * `flags` - a [`HeaderFlags`] holding the different permission flags of that chunk.
/// * `name` - the chunk's name on a 32-wide char array.
/// * `user` - the user's ID
/// * `owner - the owner's ID
/// * `group` - the group's ID
#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub file_type: Type,         // 1 byte
    pub flags: HeaderFlags,      // 2 bytes
    pub name: [u8; 32],          // 100 bytes
    pub user: UGOID,             // 8 bytes
    pub owner: UGOID,            // 8 bytes
    pub group: UGOID,            // 8 bytes
    pub parent_address: Address, // 4 bytes
    pub length: u32,             // 4 bytes. In case of a directory, it is the number of sub-items.
    pub blocks_number: u32,
    pub mode: FileMode, // If Short then we list all blocks. Else each block contains the addresses of the data blocks.
    pub padding: [u32; 10], // Padding to have a nice SHORT_MODE_LIMIT number
    pub blocks: [Address; SHORT_MODE_LIMIT as usize],
}

impl Header {
    fn is_dir(&self) -> bool {
        match self.file_type {
            Type::Dir => true,
            _ => false,
        }
    }
}

#[repr(packed)]
#[derive(Debug, Clone)]
pub struct MemFile {
    pub header: Header,
    pub data: Vec<u8>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DirBlock {
    subitems: [([u8; 28], Address); 16],
}

impl DirBlock {}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct FileBlock {
    data: [u16; 256],
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct LBATable {
    index: u16,
    data: [bool; 510],
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct LBATableGlobal {
    index: u32,
    data: [LBATable; LBA_TABLES_COUNT as usize],
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct LongFile {
    pub addresses: [Address; 128],
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
    fn write_to_disk(&self, lba_index: u32) {
        disk_operations::write_sector(&self.to_u16_array(), lba_index);
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

impl LBATableGlobal {
    pub fn init(&mut self) {
        self.index = 0;
        for i in 0..LBA_TABLES_COUNT {
            self.data[i as usize].index = 1;
            for j in 0..510 {
                self.data[i as usize].data[j as usize] = true;
            }
        }
        self.write_to_disk();
    }
    fn load_from_disk() -> Self {
        disk_operations::init();
        let mut glob = [LBATable {
            index: 1,
            data: [true; 510],
        }; LBA_TABLES_COUNT as usize];
        // Load the LBA tables from disk
        for i in 0..LBA_TABLES_COUNT {
            glob[i as usize] = LBATable::from_u16_array(disk_operations::read_sector(512 * i + 1));
        }
        Self {
            index: 0,
            data: glob,
        }
    }
    pub fn write_to_disk(&self) {
        for i in 0..LBA_TABLES_COUNT {
            self.data[i as usize].write_to_disk(512 * i + 1);
        }
    }
    fn get_index(&self) -> u32 {
        self.index
    }
    fn set_index(&mut self, index: u32) {
        self.index += index;
    }
    fn set_lba_index(&mut self, lba: u32, index: u16) {
        self.data[lba as usize].index = index;
    }
    fn get_lba_index(&self, lba: u32) -> u16 {
        self.data[lba as usize].index
    }
    fn is_available(&self, lba: u32, index: u32) -> bool {
        self.data[lba as usize].data[index as usize]
    }
    fn mark_available(&mut self, lba: u32, index: u32) {
        self.data[lba as usize].data[index as usize] = true;
    }
    fn mark_unavailable(&mut self, lba: u32, index: u32) {
        self.data[lba as usize].data[index as usize] = false;
    }
    fn is_lba_available(&self, lba: u32) -> bool {
        self.data[lba as usize].index != 0
    }
}

fn slice_vec(data: &Vec<u8>) -> Vec<[u16; 256]> {
    let n = data.len();
    let block_number = n / 512 + (if n % 512 > 0 { 1 } else { 0 });
    let mut res: Vec<[u16; 256]> = Vec::new();
    let mut index = 0;
    for _i in 0..block_number {
        let mut arr = [0_u16; 256];
        for j in 0..256 {
            if 2 * index + 1 >= n {
                break;
            }
            arr[j] = (((data[2 * index] as u16) & 0xff) << 8) + (data[2 * index + 1] as u16);
            index += 1;
        }
        res.push(arr);
    }
    res
}

/// returns a vector of fresh addresses. /!\ once they are returned, they are also marked as reserved by the filesystem!
unsafe fn get_addresses(n: u32) -> Vec<Address> {
    let mut indice = 0;
    let mut res = Vec::new();
    let mut current_lba = LBA_TABLE_GLOBAL.get_index() as usize;
    let mut current_block = LBA_TABLE_GLOBAL.get_lba_index(current_lba as u32) as usize;
    // /!\ Could loop forever if drive full
    while indice < n {
        if LBA_TABLE_GLOBAL.is_lba_available(current_lba as u32) {
            if LBA_TABLE_GLOBAL.is_available(current_lba as u32, current_block as u32) {
                res.push(Address {
                    lba: current_lba as u16,
                    block: current_block as u16,
                });
                // Write back allocation informations
                LBA_TABLE_GLOBAL.mark_unavailable(current_lba as u32, (current_block) as u32);
                indice += 1;
                LBA_TABLE_GLOBAL.data[current_lba as usize].index =
                    if LBA_TABLE_GLOBAL.data[current_lba as usize].index < 510 {
                        LBA_TABLE_GLOBAL.data[current_lba as usize].index + 1
                    } else {
                        0
                    };
            } else {
                LBA_TABLE_GLOBAL.set_lba_index(current_lba as u32, (current_block + 1) as u16);
                current_block += 1;
            }
        } else {
            LBA_TABLE_GLOBAL.set_index((current_lba + 1) as u32);
            current_lba += 1;
        }
    }
    LBA_TABLE_GLOBAL.write_to_disk();
    res
}

impl MemFile {
    pub fn write_to_disk(&self) -> Address {
        // Might want to Result<(), SomeError>
        let mut file_header = self.header;
        let length = file_header.length; // TODO : make sure it is also the length of self.data
        if file_header.mode == FileMode::Short {
            //println!("Writing in short mode.");
            let blocks_number = file_header.blocks_number;
            let header_address: Address = unsafe { get_addresses(1)[0] };
            let block_addresses: Vec<Address> = unsafe { get_addresses(blocks_number) };
            let mut addresses = [Address { lba: 0, block: 0 }; SHORT_MODE_LIMIT as usize];
            for i in 0..(blocks_number as usize) {
                addresses[i] = block_addresses[i];
            }
            file_header.blocks = addresses;
            write_to_disk(
                file_header,
                (header_address.lba * 512 + header_address.block + 1) as u32,
            );
            unsafe {
                let blocks_to_write = slice_vec(&self.data);
                for i in 0..blocks_number {
                    let file_block = FileBlock {
                        data: blocks_to_write[i as usize],
                    };
                    write_to_disk(
                        file_block,
                        (block_addresses[i as usize].lba * 512
                            + block_addresses[i as usize].block
                            + 1) as u32,
                    );
                }
                LBA_TABLE_GLOBAL.write_to_disk();
                header_address
            }
        } else {
            //println!("Writing in long mode.");
            let blocks_number = file_header.blocks_number;
            let number_address_block = blocks_number / 128 + {
                if blocks_number % 128 == 0 {
                    0
                } else {
                    1
                }
            };
            let header_address = unsafe { get_addresses(1)[0] };
            let address_block_addresses: Vec<Address> =
                unsafe { get_addresses(number_address_block) };
            let data_addresses: Vec<Address> = unsafe { get_addresses(blocks_number) };

            // This is the segment of addresses in the header
            let mut addresses = [Address { lba: 0, block: 0 }; SHORT_MODE_LIMIT as usize];
            for i in 0..(number_address_block as usize) {
                addresses[i] = address_block_addresses[i];
            }
            file_header.blocks = addresses;
            write_to_disk(
                file_header,
                (header_address.lba * 512 + header_address.block + 1) as u32,
            );
            // Here, header has been written.

            // We now write all address blocks
            let mut compteur = 0;
            for i in 0..number_address_block {
                // Fresh address-block
                let mut block = LongFile {
                    addresses: [Address { lba: 0, block: 0 }; 128 as usize],
                };
                // We fill this block with block addresses
                for j in (i * 128)..((i + 1) * 128) {
                    if compteur >= file_header.blocks_number {
                        break;
                    }
                    block.addresses[(j % 128) as usize] = data_addresses[j as usize];
                    compteur += 1;
                }
                // Then we write the address-block to the disk
                write_to_disk(
                    block,
                    (address_block_addresses[i as usize].lba * 512
                        + address_block_addresses[i as usize].block
                        + 1) as u32,
                );
            }

            // Now we write all data blocks
            let blocks_to_write = slice_vec(&self.data);
            for i in 0..blocks_number {
                let file_block = FileBlock {
                    data: blocks_to_write[i as usize],
                };
                write_to_disk(
                    file_block,
                    (data_addresses[i as usize].lba * 512 + data_addresses[i as usize].block + 1)
                        as u32,
                );
            }
            unsafe {
                LBA_TABLE_GLOBAL.write_to_disk();
            }

            header_address
        }
    }
    pub fn read_from_disk(address: Address) -> Self {
        let header: Header = read_from_disk((address.lba * 512 + address.block + 1) as u32); // /!\
        let length = header.length;
        //println!("{:?}", header);
        let mut file = Self {
            header,
            data: Vec::new(),
        };
        println!("{:?}, {}", header.name, header.length);
        if header.mode == FileMode::Short {
            //println!("Reading in short mode");
            let mut compteur = 0;
            for i in 0..header.blocks_number {
                let address = header.blocks[i as usize];
                let sector: FileBlock =
                    read_from_disk((address.lba * 512 + address.block + 1) as u32);
                for j in 0..256 {
                    if compteur >= length {
                        break;
                    }
                    unsafe {
                        file.data.push((sector.data[j] >> 8) as u8);
                        file.data.push((sector.data[j] & 0xff) as u8);
                    }
                    compteur += 1;
                }
            }
        } else if header.mode == FileMode::Long {
            //println!("Reading in long mode");
            let mut compteur = 0;
            let number_address_block = header.blocks_number / 128 + {
                if header.blocks_number % 128 == 0 {
                    0
                } else {
                    1
                }
            };
            let mut data_addresses = Vec::new();
            // Read all addresses of data blocks
            for i in 0..number_address_block {
                let address = header.blocks[i as usize];
                let sector: LongFile =
                    read_from_disk((address.lba * 512 + address.block + 1) as u32);
                for j in 0..128 {
                    if compteur >= length {
                        break;
                    }
                    data_addresses.push(sector.addresses[j]);
                    compteur += 1;
                }
            }

            // Read these data blocks
            compteur = 0;
            for i in 0..header.blocks_number {
                let address = data_addresses[i as usize];
                let sector: FileBlock =
                    read_from_disk((address.lba * 512 + address.block + 1) as u32);
                for j in 0..256 {
                    if compteur >= length {
                        break;
                    }
                    unsafe {
                        file.data.push((sector.data[j] >> 8) as u8);
                        file.data.push((sector.data[j] & 0xff) as u8);
                    }
                    compteur += 1;
                }
            }
        } else {
            panic!("No mode selected in file");
        }
        file
    }
}

impl U16Array for Header {
    fn to_u16_array(&self) -> [u16; 256] {
        unsafe { transmute::<Header, [u16; 256]>(*self) }
    }

    fn from_u16_array(array: [u16; 256]) -> Self {
        unsafe { transmute::<[u16; 256], Header>(array) }
    }
}

impl U16Array for FileBlock {
    fn to_u16_array(&self) -> [u16; 256] {
        unsafe { transmute::<FileBlock, [u16; 256]>(*self) }
    }

    fn from_u16_array(array: [u16; 256]) -> Self {
        unsafe { transmute::<[u16; 256], FileBlock>(array) }
    }
}

impl U16Array for DirBlock {
    fn to_u16_array(&self) -> [u16; 256] {
        unsafe { transmute::<DirBlock, [u16; 256]>(*self) }
    }

    fn from_u16_array(array: [u16; 256]) -> Self {
        unsafe { transmute::<[u16; 256], DirBlock>(array) }
    }
}

impl U16Array for LBATable {
    fn to_u16_array(&self) -> [u16; 256] {
        unsafe { transmute::<LBATable, [u16; 256]>(*self) }
    }

    fn from_u16_array(array: [u16; 256]) -> Self {
        unsafe { transmute::<[u16; 256], LBATable>(array) }
    }
}

impl U16Array for LongFile {
    fn to_u16_array(&self) -> [u16; 256] {
        unsafe { transmute::<LongFile, [u16; 256]>(*self) }
    }

    fn from_u16_array(array: [u16; 256]) -> Self {
        unsafe { transmute::<[u16; 256], LongFile>(array) }
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
