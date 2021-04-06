use crate::data_storage::path::Path;
use crate::scheduler::process;
use alloc::string::String;

pub struct FileDesciptorError();

/// Max number of total opened files
const MAX_TOTAL_OPEN_FILES: usize = 256;

/// Max number of openable files by a process
const MAX_TOTAL_OPEN_FILES_BY_PROCESS: usize = 16;

pub static mut GLOBAL_FILE_TABLE: GeneralFileTable = GeneralFileTable::new();

/// Contains all the open_file_tables
#[derive(Clone,Debug)]
pub struct GeneralFileTable {
    /// Array that maps a fdindex to a OpenFileTable (ie all the relevant metadata on the given file)
    tables: [Option<OpenFileTable>; MAX_TOTAL_OPEN_FILES as usize],
    /// Index of the first unoccupied space in the table
    index: usize,
}

impl GeneralFileTable {
    pub const fn new() -> Self {
        Self {
            tables: [None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None],
            index: 0,
        }
    }

    /// Inserts an entry in the file table for the given file.
    pub fn insert(&mut self, openfile: OpenFileTable) -> usize {
        for _i in 0..MAX_TOTAL_OPEN_FILES {
            if self.tables[self.index].is_none() {
                self.tables[self.index] = Some(openfile);
                return self.index;
            }
            self.index += 1;
            if self.index == MAX_TOTAL_OPEN_FILES {
                self.index = 0;
            }
        }
        // need to be improved
        panic!("file system is full");
    }

    /// Deletes an entry in the table files.
    /// Should close be added ?
    pub fn delete(&mut self, index: usize) {
        self.tables[index] = None;
    }

    /// Returns mutable copy of a given entry
    pub fn get_file_table_ref(&'static self, index: usize) -> &'static OpenFileTable {
        &self.tables[index].as_ref().unwrap()
    }
}

impl Default for GeneralFileTable {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Clone)]
pub struct OpenFileTable {
    /// path of the file
    path: Path,
}
impl OpenFileTable {
    pub fn new(path: Path) -> Self {
        Self{path}
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FileDescriptor(usize);

impl FileDescriptor {
    pub fn new(a: usize) -> Self {
        Self(a)
    }
    pub fn into_usize(self) -> usize {
        self.0
    }
    pub fn into_u64(self) -> u64 {
        self.0 as u64
    }
}

/// Held by the [`crate::scheduler::process::Process`] struct.
#[derive(Debug,Copy,Clone)]
pub struct ProcessDescriptorTable {
    /// Associates a file descriptor to the index of the open file table
    /// in the [`GLOBAL_FILE_TABLE`]
    pub files: [Option<usize>; MAX_TOTAL_OPEN_FILES_BY_PROCESS],
    pub index: usize,
}

impl ProcessDescriptorTable {
    pub const fn init() -> Self {
        Self{
            files: [None; MAX_TOTAL_OPEN_FILES_BY_PROCESS],
            index: 0,
        }
    }
    
    /// Returns reference to filetable from a filedescriptor.
    pub fn get_file_table(&self, fd: FileDescriptor) -> &'static OpenFileTable {
        unsafe{
            GLOBAL_FILE_TABLE.get_file_table_ref(self.files[fd.into_usize()].unwrap())
        }
    }

    /// TODO : add fields like flags, etc.
    pub fn create_file_table(&mut self, _path: Path, _flags: u64) -> FileDescriptor {
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
    pub fn dup(
        &mut self,
        target: FileDescriptor,
        operand: FileDescriptor,
    ) -> Result<(), FileDesciptorError> {
        // TO DO check the bounds and validity of the given data!
        self.files[operand.into_usize()] = self.files[target.into_usize()];
        Ok(())
    }
}

pub fn open(filename: String) -> FileDescriptor {
    let current_process = unsafe{process::get_current_as_mut()};
    // look for a place to put the next file in the process file descriptor table
    let mut new_pfdt = current_process.open_files;
    for i in 0..(MAX_TOTAL_OPEN_FILES_BY_PROCESS-1) {
        match new_pfdt.files[new_pfdt.index] {
            Some(_) => {
                new_pfdt.index = (new_pfdt.index + 1) % MAX_TOTAL_OPEN_FILES_BY_PROCESS;
                if i == MAX_TOTAL_OPEN_FILES_BY_PROCESS-1 {
                    panic!("Cannot open any new file for current process");
                }
            }
            None => {
                new_pfdt.index = i;
                unsafe{
                    new_pfdt.files[i] = Some(
                        GLOBAL_FILE_TABLE
                        .insert(
                            OpenFileTable::new(
                                Path::from(
                                    filename.as_str()
                                )
                            )
                        )
                    );
                }
            }
        };
    };
    current_process.open_files = new_pfdt;
    FileDescriptor::new(current_process.open_files.index)
}
