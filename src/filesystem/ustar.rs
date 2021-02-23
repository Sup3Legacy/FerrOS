use super::disk_operations;
use lazy_static::lazy_static;
use spin::Mutex;

const NUMBER_FILE: u32 = 128;

lazy_static! {
    static ref Memory: Mutex<[FileNode; NUMBER_FILE as usize]> =
        Mutex::new([FileNode::missing(); NUMBER_FILE as usize]);
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
    name: [u8; 100],    // 100 bytes
    user: UGOID,        // 8 bytes
    owner: UGOID,       // 8 bytes
    group: UGOID,       // 8 bytes
    parent_adress: u32, // 8 bytes
    length: u32,        // 8 bytes
    mode: FileMode, // If Short then we list all blocks. Else each block contains the adresses of the data blocks.
    blocks: [u32; 92],
}

/// Copied the norm, should decide our own impelmentation
/// * type_flag : 0 normal file, 1 hard link, 2 symbolic link, 3 character device, 4 block device, 5 directory, 6 named pipe (FIFO)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FileNode {
    file_name: [u8; 100],
    owner_id: u64,
    group_id: u64,

    file_mode: u64, // to store the autorisation of the file

    file_size: [u8; 12], // file size store in octal string of 12 bytes

    last_modif_high: u32,
    last_modif_low: u64, // last modif in 12 bytes

    check_sum: u64, // checksum for header record

    type_flag: FileType,
    name_of_linked_file: [u32; 25],

    ustar_indicator: [char; 6], // should be "ustar" then nul
    ustar_version: [u8; 2],     // should be 00

    user_name: [char; 32],
    group_name: [char; 32],

    device_major_number: u64,
    device_minor_number: u64,

    file_name_prefix: [char; 155],
}

pub fn get_available() -> u32 {
    let memory = Memory.lock();
    for i in 0..NUMBER_FILE {
        if (memory[i as usize].type_flag as u8) == (FileType::Available as u8) {
            return i;
        }
    }
    panic!("memory is full")
}

impl FileNode {
    pub fn missing() -> Self {
        FileNode {
            file_name: [0; 100],
            file_mode: 0,
            owner_id: 0,
            group_id: 0,
            file_size: [0; 12],
            last_modif_high: 0,
            last_modif_low: 0,
            check_sum: 0,
            type_flag: FileType::Available,
            name_of_linked_file: [NUMBER_FILE; 25],
            ustar_indicator: ['u', 's', 't', 'a', 'r', '\x00'],
            ustar_version: [0, 0],
            user_name: ['\x00'; 32],
            group_name: ['\x00'; 32],
            device_major_number: 0,
            device_minor_number: 0,
            file_name_prefix: ['\x00'; 155],
        }
    }

    pub fn change_name(&mut self, new_name: &str) -> Result<(), ()> {
        let n = new_name.len();
        if n >= 100 {
            Err(())
        } else {
            for i in 0..n {
                self.file_name[i as usize] = new_name.as_bytes()[i as usize];
            }
            self.file_name[n] = 0;
            Ok(())
        }
    }

    pub fn new_directory(file_position: u32, directory_name: &str) -> Result<(), ()> {
        let mut memory = Memory.lock();
        let new_id = get_available();
        let mut name = [0; 100];
        let name_size = name.len();
        let mut calculated_sum: u64 =
            memory[file_position as usize].owner_id + memory[file_position as usize].group_id;
        if name_size >= 100 {
            return Err(());
        }
        for i in 0..name_size {
            name[i as usize] = directory_name.as_bytes()[i as usize];
            calculated_sum = calculated_sum + (directory_name.as_bytes()[i as usize] as u64);
        }
        name[name_size] = 0;
        let mut linked_files = [NUMBER_FILE; 25];
        linked_files[0] = new_id;
        linked_files[1] = file_position;
        let directory = FileNode {
            file_name: name,
            file_mode: 0,
            owner_id: memory[file_position as usize].owner_id,
            group_id: memory[file_position as usize].group_id,
            file_size: [0; 12],
            last_modif_high: 0, // need to implement timer
            last_modif_low: 0,
            check_sum: calculated_sum,
            type_flag: FileType::Directory,
            name_of_linked_file: linked_files,
            user_name: memory[file_position as usize].user_name,
            group_name: memory[file_position as usize].group_name,
            device_major_number: 0,
            device_minor_number: 0,
            file_name_prefix: ['\x00'; 155],
            ustar_indicator: ['u', 's', 't', 'a', 'r', '\x00'],
            ustar_version: [0, 0],
        };
        memory[new_id as usize] = directory;
        Ok(())
    }
}
