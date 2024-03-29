#![allow(clippy::upper_case_acronyms)]

use super::super::fsflags::OpenFlags;
use super::super::partition::{IoError, Partition};
use super::disk_operations;
use crate::filesystem::descriptor::OpenFileTable;
use crate::println;
use crate::{data_storage::path::Path, debug, errorln};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::IntoIter;
use alloc::vec::Vec;
use core::fmt;
use core::iter::Peekable;
use core::{mem::transmute, todo};

#[derive(Debug)]
pub enum UsTarError {
    GenericError,
    FileNotFound,
    DirNotFound,
    InvalidSize,
    InvalidMode,
}

/// Main cache for Path -> Adress conversion.
/// Used to speed-up filesystem quarries while only allocating a small amount of data.
///
/// For instance, we don't (at least for now) store files, the filesystem has to
/// fetch a file from disk every time it is requested.
static mut FILE_ADRESS_CACHE: AddressCache = AddressCache(BTreeMap::new());

static mut DIR_CACHE: DirCache = DirCache(BTreeMap::new());

/// This holds the buffer for opened files.
/// When a file is opened (be it in read or write mode), it gets placed into this buffer.
/// All read/write actions are then performed on ths buffered version.
/// When the file is closed, the buffered version is then placed back into the disk
static mut FILE_BUFFER: FileBuffer = FileBuffer(BTreeMap::new());

/// Number of 512-sector segments.
///
/// It should be replaced by an automatic detection of the number of segments,
/// Using the informations given by the drive at initialization.
const LBA_TABLES_COUNT: u32 = 4;

/// Max number of blocks usable in short mode
const SHORT_MODE_LIMIT: u32 = 100;

/// Base port for the disk index 2 for QEMU
pub const DISK_PORT: u16 = 0x170;

pub static mut LBA_TABLE_GLOBAL: LBATableGlobal = LBATableGlobal {
    index: 0,
    data: [LBATable {
        index: 0,
        data: [true; 510],
    }; LBA_TABLES_COUNT as usize],
};

/// The type of data contained in the sector.
///
/// *Types*
/// * `Available` - the sector is availabel (default state)
/// * `File` - the sector contains data from a file
/// * `Directory` - the sector contains data from a directory.
///
/// This should be moved into [`crate::filesystem::mod`], as the distinction `File`/`Dir` is irrelevant here.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum FileType {
    Available = 0,
    File = 1,
    Directory = 2,
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
#[repr(C)]
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
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UGOID(pub u64);

/// The address of a sector on the disk.
///
/// *Fields*
///
/// * `lba` - the index of the `LBA` the sector belongs to.
/// * `block` - its index within that `LBA`.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Address {
    pub lba: u16,
    pub block: u16, // Really only u8 needed
}
impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self != &(Address { lba: 0, block: 0 }) {
            f.debug_struct("Address")
                .field("lba", &self.lba)
                .field("block", &self.block)
                .finish()
        } else {
            f.debug_struct("NULL").finish()
        }
    }
}

/// A chunk's header
///
/// # Fields
///
/// * `file_type` - a [`Type`] indicating what type this chunk is ([`Type::Dir`] or [`Type::File`]).
/// * `flags` - a [`HeaderFlags`] holding the different permission flags of that chunk.
/// * `name` - the chunk's name on a 32-wide char array.
/// * `user` - the user's ID
/// * `owner` - the owner's ID
/// * `group` - the group's ID
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Header {
    pub user: UGOID,             // 8 bytes
    pub owner: UGOID,            // 8 bytes
    pub group: UGOID,            // 8 bytes
    pub parent_address: Address, // 4 bytes
    pub length: u32,             // 4 bytes. In case of a directory, it is the number of sub-items.
    pub blocks_number: u32,
    pub blocks: [Address; SHORT_MODE_LIMIT as usize],
    pub flags: HeaderFlags, // 2 bytes
    pub mode: FileMode, // If Short then we list all blocks. Else each block contains the addresses of the data blocks.
    pub name: [u8; 32], // 100 bytes
    pub file_type: Type, // 1 byte
    pub padding: [u8; 40], // Padding to have a nice SHORT_MODE_LIMIT number
}
impl Header {
    /// Returns whether the header is of a directory. Pretty useless.
    fn is_dir(&self) -> bool {
        matches!(self.file_type, Type::Dir)
    }
}
fn strip_end<T: Eq>(a: &[T], c: T) -> &[T] {
    let mut idx = a.len() - 1;
    while idx > 0 && a[idx - 1] == c {
        idx -= 1;
    }
    &a[..idx]
}
impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name_to_print = strip_end(&self.name, 0);
        let blocks_to_print = strip_end(&self.blocks, Address { lba: 0, block: 0 });
        f.debug_struct("Header")
            .field("type", &self.file_type)
            //.field("owner",&self.owner)
            //.field("group",&self.group)
            .field("length", &self.length)
            .field("blocks_number", &self.blocks_number)
            //.field("flags", &self.flags)
            .field("mode", &self.mode)
            .field("blocks", &blocks_to_print)
            .field("name", &name_to_print)
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone)]
pub struct MemDir {
    name: String,
    address: Address,
    files: BTreeMap<String, Address>,
}

/// Memory representation of a raw file
#[repr(C)]
#[derive(Debug, Clone)]
pub struct MemFile {
    pub header: Header,
    pub data: Vec<u8>,
}

/// Memory representation of a sector containing data of a directory.
/// This data is a correspondance table `name`<->`Address`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DirBlock {
    subitems: [([u8; 28], Address); 16],
}

impl DirBlock {}

/// Memory representation of a sector containing generic data (that of a file).
#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct FileBlock {
    data: [u16; 256],
}

/// Memory representation of a LBA-Table.
/// It contains its index of the first available sector
/// as well as the occupation table of all its sectors
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LBATable {
    index: u16,
    data: [bool; 510],
}

/// Memory representation of the global LBA table.
/// It is never written to/read from the disk directly, but simply constructed at boot-time.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LBATableGlobal {
    index: u32,
    data: [LBATable; LBA_TABLES_COUNT as usize],
}

/// Memory representation of the address blocks of a blob stored in long-mode.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LongFile {
    pub addresses: [Address; 128],
}

impl LBATable {
    fn init(&mut self) {
        self.data = [true; 510];
        self.data[0] = false;
        self.data[1] = false;
    }
    fn load_from_disk(lba: u32, port: u16) -> Self {
        LBATable::from_u16_array(disk_operations::read_sector(lba, port))
    }
    fn write_to_disk(&self, lba_index: u32, port: u16) {
        disk_operations::write_sector(&self.to_u16_array(), lba_index, port);
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
    /// Initialization of the global LBA-table.
    /// IT basically resets its index and all its LBA's index and occupation table.
    pub fn init(&mut self, port: u16) {
        self.index = 0;
        for i in 0..LBA_TABLES_COUNT {
            self.data[i as usize].index = 1;
            for j in 0..510 {
                self.data[i as usize].data[j as usize] = true;
            }
        }
        self.write_to_disk(port);
    }
    /// Constructs the global LBA-table from disk.
    ///
    /// It simply reads all LBA-tables and store them.
    fn load_from_disk(port: u16) -> Self {
        //disk_operations::init();
        let mut index = LBA_TABLES_COUNT;
        let mut glob = [LBATable {
            index: 1,
            data: [true; 510],
        }; LBA_TABLES_COUNT as usize];
        // Load the LBA tables from disk
        for i in 0..LBA_TABLES_COUNT {
            println!("Importing table {}/{}", i, LBA_TABLES_COUNT);
            let new = LBATable::from_u16_array(disk_operations::read_sector(512 * i, port));
            println!("Imported");
            // update the global LBA-table's index if found the first non-full LBA.
            if new.index != 510 && index == LBA_TABLES_COUNT {
                index = i;
            }
            glob[i as usize] = new;
        }
        Self {
            index: 0,
            data: glob,
        }
    }
    /// Rewrites the global LBA-table to the disk.
    pub fn write_to_disk(&self, port: u16) {
        for i in 0..LBA_TABLES_COUNT {
            self.data[i as usize].write_to_disk(512 * i, port);
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
        self.data[lba as usize].index != 510
    }
}

#[derive(Debug)]
struct AddressCache(BTreeMap<Path, Address>);

#[derive(Debug)]
struct DirCache(BTreeMap<Path, MemDir>);

struct FileBuffer(BTreeMap<Path, MemFile>);

/// Slices a `Vec<u8>` of binary data into a `Vec<[u16; 256]>`.
/// This simplifies the conversion from data-blob to set of `256-u16` sectors.
fn slice_vec(data: &[u8], offset: u32) -> Vec<[u16; 256]> {
    if offset > 512 {
        return Vec::new();
    };
    let n = data.len();
    let block_number = div_ceil(n as u32, 512) as usize;
    let mut res: Vec<[u16; 256]> = Vec::new();
    let mut index = 0;
    let off_2 = (offset & 1) as usize;
    for i in 0..block_number {
        let mut arr = [0_u16; 256];
        let mut n_skip = 0;
        if i == 0 {
            n_skip = (offset / 2) as usize;
            if off_2 == 1 {
                arr[(offset / 2) as usize] = data[0] as u16;
                n_skip += 1;
            }
        };
        for elt in arr.iter_mut().skip(n_skip) {
            if 2 * index + 1 + off_2 >= n {
                if 2 * index + 1 + off_2 == n {
                    *elt = ((data[2 * index + off_2] as u16) & 0xff) << 8;
                }
                break;
            }
            *elt = (((data[2 * index + off_2] as u16) & 0xff) << 8)
                + (data[2 * index + 1 + off_2] as u16);
            index += 1;
        }
        res.push(arr);
    }
    res
}

fn div_ceil(a: u32, b: u32) -> u32 {
    a / b + if a % b == 0 { 0 } else { 1 }
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

#[derive(Clone)]
struct PathDecomp {
    decomp: Peekable<IntoIter<String>>,
    current_path: Path,
    current_dir: MemDir,
}
pub struct UsTar {
    port: u16,
    lba_table_global: LBATableGlobal,
}

impl UsTar {
    pub fn new() -> Self {
        disk_operations::init(DISK_PORT);
        let res = Self {
            port: DISK_PORT,
            lba_table_global: LBATableGlobal::load_from_disk(DISK_PORT),
        };
        unsafe {
            DIR_CACHE.0.insert(
                Path::from("root"),
                res.memdir_from_address(Address { lba: 0, block: 0 })
                    .expect("Could not read root"),
            )
        };
        unsafe { println!("DIR_CACHE : {:?}", DIR_CACHE.0) };
        res
    }

    fn find_first_uncached(path: &Path) -> Result<PathDecomp, UsTarError> {
        let mut decomp = path.slice().into_iter().peekable();
        let mut current_path = match decomp.next() {
            Some(start) => Path::from(&start),
            None => return Err(UsTarError::GenericError),
        };
        if let Some(mut current_dir) = unsafe { DIR_CACHE.0.get_mut(&current_path) } {
            while let Some(a) = unsafe { DIR_CACHE.0.get_mut(&current_path) } {
                current_dir = a;
                if let Some(next_dir) = decomp.next() {
                    current_path.push_str(&next_dir);
                    if decomp.peek().is_none() {
                        break;
                    }
                }
            }
            Ok(PathDecomp {
                decomp,
                current_path,
                current_dir: current_dir.clone(),
            })
        } else {
            Err(UsTarError::FileNotFound)
        }
    }

    fn add_cache(&self, d: &mut PathDecomp) -> Result<(), UsTarError> {
        let name = d.current_path.get_name();
        let current_dir = &d.current_dir;
        let next_address = current_dir
            .files
            .get(&name)
            .ok_or(UsTarError::FileNotFound)?;
        let memdir = self.memdir_from_address(*next_address);
        let file = self.memfile_from_disk(next_address);
        if let Ok(dir) = memdir {
            unsafe { DIR_CACHE.0.insert(Path::from(&d.current_path.to()), dir) };
        }
        if file.is_ok() {
            unsafe {
                FILE_ADRESS_CACHE
                    .0
                    .insert(Path::from(&d.current_path.to()), *next_address)
            };
        }
        Ok(())
    }

    pub fn find_address(&self, path: &Path) -> Result<Address, UsTarError> {
        if let Some(addr) = unsafe { FILE_ADRESS_CACHE.0.get(&path) } {
            Ok(*addr)
        } else {
            let mut path_decomp;
            match UsTar::find_first_uncached(path) {
                Ok(dc) => path_decomp = dc,
                Err(x) => {
                    errorln!("Find in cache failed: {:?}", x);
                    return Err(x);
                }
            }
            match self.add_cache(&mut path_decomp) {
                Ok(_) => (),
                Err(x) => {
                    errorln!("Add to cache failed:{:?}", x);
                    return Err(x);
                }
            };
            self.find_address(path)
        }
    }

    pub fn find_memdir(&self, path: &Path) -> Result<MemDir, UsTarError> {
        let memdir_res = unsafe { DIR_CACHE.0.get(path) };
        println!(":D");
        match memdir_res {
            Some(x) => Ok(x.clone()),
            None => {
                println!("Find first uncached go ->");
                let mut path_decomp = UsTar::find_first_uncached(path)?;
                println!("Find first uncached go <-");
                self.add_cache(&mut path_decomp)?;
                println!("Find add cache <-");
                self.find_memdir(path)
            }
        }
    }

    /// Fetches a `MemFile` from a `Path`.
    /// It uses both caches to speed-up search
    /// and mutate them on-the-fly to speed-up
    /// future searches even more.
    pub fn find_memfile(&self, path: &Path) -> Result<MemFile, UsTarError> {
        if path.len() == 1 {
            if let Some(file_address) = unsafe { DIR_CACHE.0.get(path) } {
                return self.memfile_from_disk(&file_address.address);
            } else {
                panic!("{:?} doesn't exist !!!!!! Report this please", path)
            }
        }
        let parent_dir = self.find_memdir(&path.get_parent())?;
        for (file_name, file_address) in parent_dir.files.iter() {
            if file_name == &path.get_name() {
                return self.memfile_from_disk(file_address);
            }
        }
        Err(UsTarError::FileNotFound)
    }

    /// Returns a vector of fresh addresses. /!\ once they are returned, they are also marked as reserved by the filesystem!
    /// So one must avoid getting more addresses than needed (this could allocate all the disk with blank unused data).
    unsafe fn get_addresses(&mut self, n: u32) -> Vec<Address> {
        let mut index = 0;
        let mut res = Vec::new();
        let mut current_lba = self.lba_table_global.get_index() as usize;
        let mut current_block = self.lba_table_global.get_lba_index(current_lba as u32) as usize;
        // /!\ Could loop forever if drive full
        while index < n {
            if self.lba_table_global.is_lba_available(current_lba as u32) {
                if self
                    .lba_table_global
                    .is_available(current_lba as u32, current_block as u32)
                {
                    res.push(Address {
                        lba: current_lba as u16,
                        block: current_block as u16,
                    });
                    // Write back allocation informations
                    self.lba_table_global
                        .mark_unavailable(current_lba as u32, (current_block) as u32);
                    index += 1;
                    self.lba_table_global.data[current_lba as usize].index =
                        if self.lba_table_global.data[current_lba as usize].index < 510 {
                            self.lba_table_global.data[current_lba as usize].index + 1
                        } else {
                            510
                        };
                } else {
                    self.lba_table_global
                        .set_lba_index(current_lba as u32, (current_block + 1) as u16);
                    current_block += 1;
                }
            } else {
                self.lba_table_global.set_index((current_lba + 1) as u32);
                current_lba += 1;
                current_block =
                    self.lba_table_global
                        .get_lba_index(self.lba_table_global.index) as usize;
            }
        }
        self.lba_table_global.write_to_disk(self.port);
        res
    }

    /// Writes a `MemFile` to the disk and returns the address of its header.
    /// It works by pre-allocating exactly the number of sectors needed and then populating them.
    ///
    /// The logic depends heavily on whether the file if short enough to fit in the `SHORT-MODE` or not.
    pub fn write_memfile_to_disk(&mut self, memfile: &MemFile) -> Address {
        // Might want to Result<(), SomeError>
        let mut file_header = memfile.header;
        let _length = file_header.length; // TODO : make sure it is also the length of self.data
        let blocks_number = file_header.blocks_number;
        match file_header.mode {
            FileMode::Short => {
                //println!("Writing in short mode.");
                let header_address: Address = unsafe { self.get_addresses(1)[0] };
                let block_addresses: Vec<Address> = unsafe { self.get_addresses(blocks_number) };
                let mut addresses = [Address { lba: 0, block: 0 }; SHORT_MODE_LIMIT as usize];
                addresses[..(blocks_number as usize)]
                    .clone_from_slice(&block_addresses[..(blocks_number as usize)]);
                file_header.blocks = addresses;
                self.write_to_disk(
                    file_header,
                    (header_address.lba * 512 + header_address.block + 1) as u32,
                );
                let blocks_to_write = slice_vec(&memfile.data, 0);
                for i in 0..blocks_number {
                    let file_block = FileBlock {
                        data: blocks_to_write[i as usize],
                    };
                    self.write_to_disk(
                        file_block,
                        (block_addresses[i as usize].lba * 512
                            + block_addresses[i as usize].block
                            + 1) as u32,
                    );
                }
                self.lba_table_global.write_to_disk(self.port);
                header_address
            }
            FileMode::Long => {
                //println!("Writing in long mode.");
                let number_address_block = div_ceil(blocks_number, 128);
                let header_address = unsafe { self.get_addresses(1)[0] };
                let address_block_addresses: Vec<Address> =
                    unsafe { self.get_addresses(number_address_block) };
                let data_addresses: Vec<Address> = unsafe { self.get_addresses(blocks_number) };

                // This is the segment of addresses in the header
                let mut addresses = [Address { lba: 0, block: 0 }; SHORT_MODE_LIMIT as usize];
                addresses[..(number_address_block as usize)]
                    .clone_from_slice(&address_block_addresses[..(number_address_block as usize)]);
                file_header.blocks = addresses;
                self.write_to_disk(
                    file_header,
                    (header_address.lba * 512 + header_address.block + 1) as u32,
                );
                // Here, header has been written.

                // We now write all address blocks
                let mut compteur = 0;
                for i in 0..number_address_block {
                    // Fresh address-block
                    let mut block = LongFile {
                        addresses: [Address { lba: 0, block: 0 }; 128_usize],
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
                    self.write_to_disk(
                        block,
                        (address_block_addresses[i as usize].lba * 512
                            + address_block_addresses[i as usize].block + 1)
                            as u32,
                    );
                }

                // Now we write all data blocks
                let blocks_to_write = slice_vec(&memfile.data, 0);
                for i in 0..blocks_number {
                    let file_block = FileBlock {
                        data: blocks_to_write[i as usize],
                    };
                    self.write_to_disk(
                        file_block,
                        (data_addresses[i as usize].lba * 512 + data_addresses[i as usize].block + 1)
                            as u32,
                    );
                }
                self.lba_table_global.write_to_disk(self.port);

                header_address
            }
        }
    }

    pub fn memfile_from_disk(&self, address: &Address) -> Result<MemFile, UsTarError> {
        let header: Header = self.read_from_disk((address.lba * 512 + address.block + 1) as u32); // /!\
        let length = match header.file_type {
            Type::File => header.length,
            Type::Dir => header.length * 32,
        };
        //println!("{:?}", header);
        let mut file = MemFile {
            header,
            data: Vec::new(),
        };
        //println!("{:?}, {}, {:?}", header.name, header.length, header.mode);
        if header.mode == FileMode::Short {
            //println!("Reading in short mode");
            let mut counter = 0;
            for i in 0..header.blocks_number {
                let address = header.blocks[i as usize];
                let sector: FileBlock =
                    self.read_from_disk((address.lba * 512 + address.block + 1) as u32);
                for j in 0..256 {
                    if counter >= length {
                        break;
                    }
                    file.data.push((sector.data[j] >> 8) as u8);
                    if counter + 1 >= length {
                        break;
                    }
                    file.data.push((sector.data[j] & 0xff) as u8);
                    counter += 2;
                }
            }
        } else if header.mode == FileMode::Long {
            println!("Reading in long mode");
            let mut counter = 0;
            let _number_address_block = div_ceil(header.blocks_number, 128);
            let mut data_addresses = Vec::new();
            // Read all addresses of data blocks
            let nb_bloc = div_ceil(length, 512);
            for i in 0..header.blocks_number {
                let address = header.blocks[i as usize];
                let sector: LongFile =
                    self.read_from_disk((address.lba * 512 + address.block + 1) as u32);
                for j in 0..128 {
                    if counter >= nb_bloc {
                        break;
                    }
                    data_addresses.push(sector.addresses[j]);
                    counter += 1;
                }
            }
            //println!("LEL {:?}", data_addresses);
            // Read these data blocks
            counter = 0;
            println!("{} for {}", nb_bloc, header.blocks_number);
            for i in 0..nb_bloc {
                let address = data_addresses[i as usize];
                let sector: FileBlock =
                    self.read_from_disk((address.lba * 512 + address.block + 1) as u32);
                for j in 0..256 {
                    if counter >= length {
                        break;
                    }
                    file.data.push((sector.data[j] >> 8) as u8);
                    if counter + 1 == length {
                        break;
                    }
                    file.data.push((sector.data[j] & 0xff) as u8);
                    counter += 2;
                }
            }
        } else {
            return Err(UsTarError::InvalidMode);
        }
        println!("Finished that file");
        Ok(file)
    }

    fn memdir_from_address(&self, address: Address) -> Result<MemDir, UsTarError> {
        let file = self.memfile_from_disk(&address)?;
        let data = file.data;
        let len = (file.header.length << 1) as usize; // x2 because header.length is in u16... Might change that

        // These assert_eq are only here for debugging purposes
        //assert_eq!(len as usize, data.len()); // length in u8 of the data segment of the directory
        if file.header.file_type != Type::Dir {
            return Err(UsTarError::InvalidMode);
        }
        // Checks whether the blob is really a directory
        //assert_eq!(len % 32, 0); // Checks whether the data segment has a compatible size

        let mut files: BTreeMap<String, Address> = BTreeMap::new();
        //println!("#0");
        let _number = len / 32; // number of sub_items of the dir
                                //println!("Number : {} {}", number, len);
        for i in 0..(len / 2) {
            let mut name_vec = Vec::new();
            let mut itter = 0;
            while itter < 28 && data[32 * i + itter] != 0 {
                name_vec.push(data[32 * i + itter] as u8);
                itter += 1;
            }
            let name = String::from_utf8_lossy(&name_vec[..]).into_owned();
            let temp_address = Address {
                lba: ((data[32 * i + 28] as u8 as u16) << 8) + (data[32 * i + 29] as u8 as u16), // TODO /!\ May be incorrect
                block: ((data[32 * i + 30] as u8 as u16) << 8) + (data[32 * i + 31] as u8 as u16), // TODO /!\ May be incorrect
            };
            files.entry(name).or_insert(temp_address);
        }
        // Acquire the name of the directory
        let mut name = String::new();
        let mut itter = 0;
        while itter < 32 && file.header.name[itter] != 0 {
            name.push(file.header.name[itter] as char);
            itter += 1;
        }
        Ok(MemDir {
            name,
            address,
            files,
        })
    }

    pub fn add_file_in_directory(
        &mut self,
        parent_dir: MemDir,
        parent_path: Path,
        name: [u8; 32],
        name_str: String,
        pos: Address,
    ) -> Result<(), UsTarError> {
        let _s = parent_dir.files.len();
        let addr = parent_dir.address;
        let mut dir = self.read_from_disk::<Header>(addr.lba as u32 * 512 + addr.block as u32 + 1);
        let mut name_arr2 = [0; 28];
        for i in 0..14 {
            name_arr2[2 * i] = name[2 * i + 1];
            name_arr2[2 * i + 1] = name[2 * i];
        }
        if dir.length < 160 {
            if dir.length % 16 == 0 {
                let new_add = unsafe { self.get_addresses(1)[0] };
                let mut dir_bloc = DirBlock {
                    subitems: [([b' '; 28], Address { lba: 0, block: 0 }); 16],
                };
                dir_bloc.subitems[0] = (name_arr2, pos);
                self.write_to_disk(dir_bloc, new_add.lba as u32 * 512 + new_add.block as u32 + 1);
                dir.blocks[(dir.blocks_number) as usize] = new_add;
                dir.blocks_number += 1;
            } else {
                let add = dir.blocks[(dir.length / 16) as usize];
                let mut dir_bloc =
                    self.read_from_disk::<DirBlock>(add.lba as u32 * 512 + add.block as u32 + 1);
                dir_bloc.subitems[(dir.length % 16) as usize] = (name_arr2, pos);
                self.write_to_disk(dir_bloc, add.lba as u32 * 512 + add.block as u32 + 1);
            }

            dir.length += 1;
            self.write_to_disk(dir, addr.lba as u32 * 512 + addr.block as u32 + 1);
            unsafe {
                match DIR_CACHE.0.remove(&parent_path) {
                    Some(mut d) => {
                        d.files.insert(name_str, pos);
                        DIR_CACHE.0.insert(parent_path, d);
                    }
                    None => panic!("Should not happen, please report this"),
                }
            };
            Ok(())
        } else {
            Err(UsTarError::GenericError)
        }
    }

    pub fn write_to_disk(&self, data: impl U16Array, lba: u32) {
        disk_operations::write_sector(&data.to_u16_array(), lba, self.port);
    }

    pub fn read_from_disk<T: U16Array>(&self, lba: u32) -> T {
        T::from_u16_array(disk_operations::read_sector(lba, self.port))
    }

    pub fn del_file(&mut self, oft: &OpenFileTable) -> Result<Address, UsTarError> {
        let memfile = self.find_memfile(oft.get_path());
        match memfile {
            Err(_) => Err(UsTarError::FileNotFound),
            Ok(file) => {
                let header = file.header;
                match header.mode {
                    FileMode::Short => {
                        for addr in header.blocks.iter() {
                            self.lba_table_global
                                .mark_available(addr.lba as u32, addr.block as u32);
                        }
                    }
                    FileMode::Long => {
                        for addr in header.blocks.iter() {
                            let sector: LongFile =
                                self.read_from_disk((addr.lba * 512 + addr.block + 1) as u32);
                            for a in sector.addresses.iter() {
                                self.lba_table_global
                                    .mark_available(a.lba as u32, a.block as u32);
                            }
                            self.lba_table_global
                                .mark_available(addr.lba as u32, addr.block as u32);
                        }
                    }
                }
                let header_address = self.find_address(oft.get_path())?;
                self.lba_table_global
                    .mark_available(header_address.lba as u32, header_address.block as u32);
                unsafe { FILE_ADRESS_CACHE.0.remove(oft.get_path()) };
                Ok(header_address)
            }
        }
    }
}
impl Default for UsTar {
    fn default() -> Self {
        Self::new()
    }
}
impl Partition for UsTar {
    fn open(&mut self, path: &Path, _flags: OpenFlags) -> Option<usize> {
        let mut path_name = String::from("root");
        if !path.is_empty() {
            path_name.push('/');
        }
        path_name.push_str(&path.to());
        let path_name = Path::from(&path_name);
        let _memfile = self.find_memfile(&path_name);
        /*match memfile {
            Some(f) => {
                if flags.contains(OpenFlags::ORD | OpenFlags::OWR) | flags.contains(OpenFlags::OXCUTE | OpenFlags::OWR) {
                    None
                } else {
                    if flags.contains(OpenFlags::OAPPEND) {
                        todo!()
                    } else if flags.contains(OpenFlags::OXCUTE) {
                        Some(1)
                    } else {
                        Some(1)
                    }
                }
            },
            None => {
                if flags.contains(OpenFlags::ORD | OpenFlags::OWR) | flags.contains(OpenFlags::OXCUTE | OpenFlags::OWR) {
                    None
                } else {
                    if !flags.contains(OpenFlags::OWR) {
                        None
                    else {
                        todo!()
                    }
                }
            },
        }*/
        Some(1)
    }

    fn read(&mut self, oft: &OpenFileTable, size: usize) -> Result<Vec<u8>, IoError> {
        let mut path_name = String::from("root");
        if !oft.get_path().is_empty() {
            path_name.push('/');
        }
        path_name.push_str(&oft.get_path().to());
        let path = Path::from(&path_name);
        let file = match self.find_memfile(&path) {
            Ok(f) => {
                match f.header.file_type {
                    Type::File => f,
                    Type::Dir => {
                        let mut new_data = Vec::new();
                        for i in 0..(f.data.len()/32) {
                            for j in 0..28 {
                                new_data.push(f.data[i * 32 + j])
                            }
                            new_data.push(b'\n')
                        }
                        MemFile {
                            header: f.header,
                            data: new_data
                        }
                    }
                }
            },
            Err(_) => return Err(IoError::Continue),
        };
        let res = if size == usize::MAX {
            file.data
        } else if oft.get_offset() >= file.data.len() {
            Vec::new()
        } else if oft.get_offset() + size >= file.data.len() {
            file.data[oft.get_offset()..].to_vec()
        } else {
            file.data[oft.get_offset()..oft.get_offset() + size].to_vec()
        };
        Ok(res)
    }

    fn write(&mut self, oft: &OpenFileTable, buffer: &[u8]) -> isize {
        let mut path_name = String::from("root");
        if !oft.get_path().is_empty() {
            path_name.push('/');
        }
        path_name.push_str(&oft.get_path().to());
        let path_name = Path::from(&path_name);
        debug!("Writing {:#?}, with {:?}", oft, buffer);
        if !(oft.get_flags().contains(OpenFlags::OWR)) {
            errorln!("Tried to write in {:?}, but no right!", path_name);
            return -1; // no right to write
        }
        // find the file
        let memfile = self.find_memfile(&path_name);
        match memfile {
            Err(_) => {
                debug!("File {:?} did not exist, creating it", path_name);
                // create the file?
                if !oft.get_flags().contains(OpenFlags::OCREAT) {
                    errorln!(
                        "Tried to create {:?}, but no right! {:?}",
                        path_name,
                        oft.get_flags()
                    );
                    -1
                } else {
                    // look for the parent folder in which we will create the file
                    let parent_path = path_name.get_parent();
                    let parent_dir = if let Ok(a) = self.find_memdir(&parent_path) {
                        debug!("Parent folder is : {:?}", a);
                        a
                    } else {
                        errorln!(
                            "Tried to access {:?}, but parent folder does not exist",
                            path_name
                        );
                        return -1;
                    };
                    let name = path_name.get_name();
                    let bytes = name.as_bytes();
                    if name.len() > 32 {
                        errorln!("File name too long!");
                        return -1;
                    }
                    // convert it in a byte array
                    let mut name_arr = [0; 32];
                    name_arr[..name.len()].clone_from_slice(&bytes);
                    let length = buffer.len() as u32;
                    let blocks_number = div_ceil(length, 512) as u32;
                    let mode = if blocks_number > SHORT_MODE_LIMIT {
                        FileMode::Long
                    } else {
                        FileMode::Short
                    };
                    let header = Header {
                        user: UGOID(412),
                        owner: UGOID(666),
                        group: UGOID(777),
                        parent_address: parent_dir.address,
                        length: length as u32,
                        blocks_number,
                        blocks: [Address { lba: 0, block: 0 }; SHORT_MODE_LIMIT as usize],
                        flags: HeaderFlags {
                            user_owner: 0b1111_1111_u8,
                            group_misc: 0b1111_1111_u8,
                        },
                        mode,
                        name: name_arr,
                        file_type: Type::File,
                        padding: [0_u8; 40],
                    };
                    let file = MemFile {
                        header,
                        data: buffer.to_vec(),
                    };
                    debug!("File created: {:?}", file);
                    let file_address = self.write_memfile_to_disk(&file);
                    match self.add_file_in_directory(
                        parent_dir,
                        parent_path,
                        name_arr,
                        name,
                        file_address,
                    ) {
                        Ok(()) => buffer.len() as isize,
                        Err(_) => panic!("Unhandled"),
                    }
                }
            }
            Ok(mut file) => {
                if file.header.file_type == Type::Dir {
                    return 0;
                }
                // compute the new size of the file, to see if we need to allocate/deallocate disk memory
                debug!("File exists and is : {:?}", file);
                let header_address = self.find_address(&path_name).unwrap();
                let old_size = file.header.length;
                let true_offset = if oft.get_flags().contains(OpenFlags::OAPPEND) {
                    old_size as usize
                } else {
                    oft.get_offset()
                } as u32;
                debug!("Old size = {}", old_size);
                debug!("True offset = {}", true_offset);
                let new_size = true_offset + (buffer.len() as u32);
                match file.header.mode {
                    FileMode::Short => {
                        debug!("File was short!");
                        let block_addresses = file.header.blocks;
                        let mut blocks_to_write = slice_vec(buffer, true_offset % 512);
                        let number_blocks_to_write = blocks_to_write.len();
                        let block_offset = true_offset / 512;
                        let new_blocks_number = div_ceil(true_offset + buffer.len() as u32, 512);

                        if new_size <= old_size {
                            debug!("New file is shorter!");
                            for i in new_blocks_number..file.header.blocks_number {
                                let addr = file.header.blocks[i as usize];
                                self.lba_table_global
                                    .mark_available(addr.lba as u32, addr.block as u32);
                            }
                            if true_offset % 512 != 0 {
                                let b: [u16; 256] = self
                                    .read_from_disk::<FileBlock>(
                                        (file.header.blocks[block_offset as usize].lba * 512
                                            + file.header.blocks[block_offset as usize].block + 1)
                                            as u32,
                                    )
                                    .to_u16_array();
                                for i in 0..(true_offset % 512) / 2 {
                                    blocks_to_write[0][i as usize] = b[i as usize];
                                }
                                if true_offset % 2 == 1 {
                                    blocks_to_write[0][((true_offset % 512) / 2) as usize] |=
                                        b[((true_offset % 512) / 2) as usize] & 0xFF00;
                                }
                            }
                            for i in 0..number_blocks_to_write {
                                let file_block = FileBlock {
                                    data: blocks_to_write[i as usize],
                                };
                                self.write_to_disk(
                                    file_block,
                                    (block_addresses[i + block_offset as usize].lba * 512
                                        + block_addresses[i + block_offset as usize].block + 1)
                                        as u32,
                                );
                            }
                            file.header.blocks_number = new_blocks_number;
                            file.header.length = new_size;
                            self.write_to_disk(
                                file.header,
                                (header_address.lba * 512 + header_address.block + 1) as u32,
                            );
                            self.lba_table_global.write_to_disk(self.port);
                            (new_size - true_offset) as isize
                        } else if new_size <= 512 * SHORT_MODE_LIMIT {
                            debug!("File longer but still short");

                            let new_addresses = unsafe {
                                self.get_addresses(new_blocks_number - file.header.blocks_number)
                            };
                            for i in file.header.blocks_number..new_blocks_number {
                                file.header.blocks[i as usize] =
                                    new_addresses[(i - file.header.blocks_number + 1) as usize];
                            }
                            if true_offset % 512 != 0 {
                                let b: [u16; 256] = self
                                    .read_from_disk::<FileBlock>(
                                        (file.header.blocks[block_offset as usize].lba * 512
                                            + file.header.blocks[block_offset as usize].block + 1)
                                            as u32,
                                    )
                                    .to_u16_array();
                                for i in 0..(true_offset % 512) / 2 {
                                    blocks_to_write[0][i as usize] = b[i as usize];
                                }
                                if true_offset % 2 == 1 {
                                    blocks_to_write[0][((true_offset % 512) / 2) as usize] |=
                                        b[((true_offset % 512) / 2) as usize] & 0xFF00;
                                }
                            }
                            for i in 0..number_blocks_to_write {
                                let file_block = FileBlock {
                                    data: blocks_to_write[i as usize],
                                };
                                self.write_to_disk(
                                    file_block,
                                    (file.header.blocks[i + block_offset as usize].lba * 512
                                        + file.header.blocks[i + block_offset as usize].block + 1)
                                        as u32,
                                );
                            }
                            file.header.blocks_number = new_blocks_number;
                            file.header.length = new_size;
                            self.write_to_disk(
                                file.header,
                                (header_address.lba * 512 + header_address.block + 1) as u32,
                            );
                            self.lba_table_global.write_to_disk(self.port);
                            (new_size - true_offset) as isize
                        } else {
                            debug!("File longer and becomes Long");
                            let old_header_addr_res = self.del_file(oft);
                            let old_header_addr = match old_header_addr_res {
                                Err(_) => return -1,
                                Ok(x) => x,
                            };
                            let effective_data =
                                [&file.data[..true_offset as usize], buffer].concat();
                            debug!("New data: {:?}", effective_data);
                            let exit_code = self.write(oft, &effective_data);
                            if exit_code < 0 {
                                return exit_code;
                            }
                            let new_header_addr_res = self.find_address(&path_name);
                            let new_header_addr = match new_header_addr_res {
                                Err(_) => return -1,
                                Ok(x) => x,
                            };
                            let new_header: Header = self.read_from_disk(
                                (new_header_addr.lba * 512 + new_header_addr.block + 1) as u32,
                            );
                            self.write_to_disk(
                                new_header,
                                (old_header_addr.lba * 512 + old_header_addr.block + 1) as u32,
                            );
                            self.lba_table_global.mark_available(
                                new_header_addr.lba as u32,
                                new_header_addr.block as u32,
                            );
                            exit_code
                        }
                    }
                    FileMode::Long => {
                        debug!("File was Long");
                        let old_header_addr_res = self.del_file(oft);
                        let old_header_addr = match old_header_addr_res {
                            Err(_) => return -1,
                            Ok(x) => x,
                        };
                        let effective_data = [&file.data[..true_offset as usize], buffer].concat();
                        debug!("New data: {:?}", effective_data);
                        let exit_code = self.write(oft, &effective_data);
                        if exit_code < 0 {
                            return exit_code;
                        }
                        let new_header_addr_res = self.find_address(&path_name);
                        let new_header_addr = match new_header_addr_res {
                            Err(_) => return -1,
                            Ok(x) => x,
                        };
                        let new_header: Header = self.read_from_disk(
                            (new_header_addr.lba * 512 + new_header_addr.block + 1) as u32,
                        );
                        self.write_to_disk(
                            new_header,
                            (old_header_addr.lba * 512 + old_header_addr.block + 1) as u32,
                        );
                        self.lba_table_global.mark_available(
                            new_header_addr.lba as u32,
                            new_header_addr.block as u32,
                        );
                        exit_code
                    }
                }
            }
        }
    }

    fn close(&mut self, _oft: &OpenFileTable) -> bool {
        false
    }

    /*fn duplicate(&mut self, _path: &Path, _id: usize) -> Option<usize> {
        todo!()
    }*/

    fn lseek(&self) {
        todo!()
    }

    fn flush(&self) {
        todo!()
    }

    fn read_raw(&self) {
        todo!()
    }

    fn give_param(&mut self, _oft: &OpenFileTable, _param: usize) -> usize {
        usize::MAX
    }
}
