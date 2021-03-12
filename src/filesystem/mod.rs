use crate::data_storage::path::Path;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::todo;
pub mod drivers;
pub mod fsflags;
pub mod partition;
pub mod test;
pub mod vfs;

// disk_operations here is only temporary.
// TODO remove it and add interface in driver::ustar
pub use drivers::{disk_operations, ustar};

use crate::{print, println};

/// Main cache for Path -> Adress conversion.
/// Used to speed-up filesystem quarries while only allocating a small amount of data.
///
/// For instance, we don't (at least for now) store files, the filesystem has to
/// fetch a file from disk every time it is requested.
static mut FILE_ADRESS_CACHE: AddressCache = AddressCache(BTreeMap::new());

static mut DIR_CACHE: DirCache = DirCache(BTreeMap::new());

#[derive(Debug, PartialEq)]
pub struct FileSystemError(String);

#[derive(Debug)]
struct MemDir {
    name: String,
    address: ustar::Address,
    files: BTreeMap<String, ustar::Address>,
}

impl MemDir {
    pub fn from_address(address: ustar::Address) -> Self {
        let file = ustar::MemFile::read_from_disk(address);
        let data = file.data;
        let len = (file.header.length << 1) as usize; // x2 because header.length is in u16... Might change that

        // These assert_eq are only here for debugging purposes
        assert_eq!(len as usize, data.len()); // length in u8 of the data segment of the directory
        assert_eq!(file.header.file_type, ustar::Type::Dir); // Checks whether the blob is really a directory
        assert_eq!(len % 32, 0); // Checks whether the data segment has a compatible size
        let mut files: BTreeMap<String, ustar::Address> = BTreeMap::new();
        let number = len / 32; // number of sub_items of the dir
        for i in 0..number {
            let mut name = String::new();
            let mut itter = 0;
            while itter < 28 && data[32 * i + itter] != 0 {
                name.push(data[32 * i + itter] as char);
                itter += 1;
            }
            let temp_address = ustar::Address {
                lba: ((data[32 * i] as u16) << 8) + (data[32 * i + 1] as u16), // TODO /!\ May be incorrect
                block: ((data[32 * i + 2] as u16) << 8) + (data[32 * i + 3] as u16), // TODO /!\ May be incorrect
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
        Self {
            name,
            address,
            files,
        }
    }
}

/// Fetches a `MemFile` from a `Path`.
/// It uses both caches to speed-up search
/// and mutate them on-the-fly to speed-up
/// future searches even more.
pub unsafe fn fetch_data(path: Path) -> ustar::MemFile {
    if let Some(add) = FILE_ADRESS_CACHE.0.get(&path) {
        ustar::MemFile::read_from_disk(*add)
    } else {
        let mut decomp = path.slice().into_iter();
        let mut current_path = Path::new();
        let mut current_dir = DIR_CACHE.0.get_mut(&current_path).unwrap();
        while let Some(a) = DIR_CACHE.0.get_mut(&current_path) {
            current_dir = a;
            if let Some(next_dir) = decomp.next() {
                current_path.push_str(&next_dir);
            }
        }
        // At this point, we just came onto a directory that isn't already in cache.
        while let Some(next_dir) = decomp.next() {
            current_path.push_str(&next_dir);
            let next_address = current_dir.files.get_mut(&next_dir).unwrap();
            let current_dir = MemDir::from_address(*next_address);
            DIR_CACHE
                .0
                .insert(Path::from(&current_path.to()), current_dir);
            FILE_ADRESS_CACHE
                .0
                .insert(Path::from(&current_path.to()), *next_address);
        }
        // This time, it should fall in the if because we cached all directories
        fetch_data(path)
    }
}

#[derive(Debug)]
struct AddressCache(BTreeMap<Path, ustar::Address>);

#[derive(Debug)]
struct DirCache(BTreeMap<Path, MemDir>);

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

pub fn init() {
    // Initializes the disk
    disk_operations::init();
    unsafe {
        // Initializes the LBA tables
        ustar::LBA_TABLE_GLOBAL.init();
    }
    // Root is stored on disk at (0,0)
    let address = ustar::Address { lba: 0, block: 0 };
    let file = ustar::MemFile::read_from_disk(address);
    let data = file.data;
    let len = (file.header.length << 1) as usize; // x2 because header.length is in u16... Might change that

    // These assert_eq are only here for debugging purposes
    assert_eq!(len as usize, data.len()); // length in u8 of the data segment of the directory
    assert_eq!(file.header.file_type, ustar::Type::Dir); // Checks whether the blob is really a directory
    assert_eq!(len % 32, 0); // Checks whether the data segment has a compatible size
    let mut files: BTreeMap<String, ustar::Address> = BTreeMap::new();
    let number = len / 32; // number of sub_items of the dir
    for i in 0..number {
        let mut name = String::new();
        let mut itter = 0;
        while itter < 28 && data[32 * i + itter] != 0 {
            name.push(data[32 * i + itter] as char);
            itter += 1;
        }
        let temp_address = ustar::Address {
            lba: ((data[32 * i] as u16) << 8) + data[32 * i + 1] as u16, // TODO /!\ May be incorrect
            block: ((data[32 * i + 2] as u16) << 8) + data[32 * i + 3] as u16, // TODO /!\ May be incorrect
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
    let root = MemDir {
        name,
        address,
        files,
    };
    println!("{:?}", &root);
    unsafe { DIR_CACHE.0.insert(Path::from(&String::from("")), root) };
}

fn test() {
    println!(
        "{:?}",
        open_file(Path::from(&String::from("test")), OpenMode::Read)
    );
}
