use core::cmp::min;

use crate::data_storage::path::Path;
use alloc::vec::Vec;
use lazy_static::lazy;
use lazy_static::lazy_static;

/// Max number of total opened files
const MAX_TOTAL_OPEN_FILES: usize = 256;

/// Max number of openable files by a process
const MAX_TOTAL_OPEN_FILES_BY_PROCESS: usize = 16;

lazy_static! {
    pub static ref GLOBAL_FILE_TABLE: GeneralFileTable = GeneralFileTable::new();
}

/// Contains all the open_file_tables
pub struct GeneralFileTable {
    /// Array containing all the filetables
    tables: Vec<Option<OpenFileTable>>,
    /// Index of the first unoccupied space in the table
    index: usize,
}

impl GeneralFileTable {
    pub fn new() -> Self {
        let mut tab = Vec::new();
        for _ in 0..MAX_TOTAL_OPEN_FILES {
            tab.push(None);
        }
        Self {
            tables: tab,
            index: 0,
        }
    }

    /// Inserts an entry
    pub fn insert(&mut self, openfile: OpenFileTable) {
        self.tables[self.index] = Some(openfile);
        loop {
            self.index += 1;
            if self.index == MAX_TOTAL_OPEN_FILES {
                panic!("VFS : reached maximum number of opened files.");
            }
            if let None = self.tables[self.index] {
                break;
            }
        }
    }

    /// Deletes an entry
    pub fn delete(&mut self, index: usize) {
        self.tables[index] = None;
        self.index = min(index, self.index);
    }
    pub fn get_file_table_ref(&'static self, index: usize) -> &'static OpenFileTable {
        &self.tables[index].as_ref().unwrap()
    }
}

#[derive(Debug)]
pub struct OpenFileTable {
    /// path of the file
    path: Path,
}

#[derive(Debug, Clone, Copy)]
pub struct FileDescriptor(usize);

impl FileDescriptor {
    pub fn new(a: usize) -> Self {
        Self(a)
    }
    pub fn into_usize(&self) -> usize {
        self.0
    }
}

/// Should be held by the [`crate::scheduler::process::Process`] struct.
pub struct ProcessDescriptorTable {
    /// Associates a file descriptor to the index of the open file table
    /// in the [`GLOBAL_FILE_TABLE`]
    files: [Option<usize>; MAX_TOTAL_OPEN_FILES_BY_PROCESS],
    index: usize,
}

impl ProcessDescriptorTable {
    /// Returns reference to filetable from a filedescriptor.
    pub fn get_file_table(&self, fd: FileDescriptor) -> &'static OpenFileTable {
        GLOBAL_FILE_TABLE.get_file_table_ref(self.files[fd.into_usize()].unwrap())
    }

    /// TODO : add fields like flags, etc.
    pub fn create_file_table(&mut self, path: Path, flags: u64) -> FileDescriptor {
        // Here we create a new OpenFileTable.
        // We fill it with all the passed values,
        // inserts it into the GLOBAL_FILE_TABLE
        // and finally add an entry to the index in
        // the GLOBAL_FILE_TABLE into the first
        // unoccupied FileDescriptor field.
        // We then return the associated FileDescriptor
        todo!()
    }

    /// self.dup(4, 1) redirects fd 1 to the OpenFileTable
    /// fd 4 points to.
    pub fn dup(&mut self, target: FileDescriptor, operand: FileDescriptor) -> Result<(), ()> {
        // TO DO check the bounds and validity of the given data!
        self.files[operand.into_usize()] = self.files[target.into_usize()];
        Ok(())
    }
}
