//! Only a temporary file

use super::super::vfs::VFS;
use super::ustar;
use crate::data_storage::path::Path;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

impl MemDir {}

pub fn init() {
    // Initializes the disk
    //disk_operations::init();
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
