use crate::data_storage::path::Path;

pub struct FileDesciptorError();

/// Max number of total opened files
const MAX_TOTAL_OPEN_FILES: usize = 256;

/// Max number of openable files by a process
const MAX_TOTAL_OPEN_FILES_BY_PROCESS: usize = 16;

static mut GLOBAL_FILE_TABLE: GeneralFileTable = GeneralFileTable::new();

/// Contains all the open_file_tables
pub struct GeneralFileTable {
    /// Array that maps a fdindex to a OpenFileTable (ie all the relevant metadata on the given file)
    tables: [Option<OpenFileTable>; MAX_TOTAL_OPEN_FILES as usize],
    /// Index of the first unoccupied space in the table
    index: usize,
}

impl GeneralFileTable {
    /// Yeah.
    pub const fn new() -> Self {
        Self {
            tables: [
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
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

#[derive(Debug)]
pub struct OpenFileTable {
    /// path of the file
    path: Path,
    flags: u64,
    offset: usize,
}
impl OpenFileTable {
    pub fn new(path: Path, flags: u64) -> Self {
        Self {
            path,
            flags,
            offset: 0,
        }
    }
    pub fn get_path(&self) -> Path {
        self.path.clone()
    }
    pub fn get_offset(&self) -> usize {
        self.offset
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

/// Should be held by the [`crate::scheduler::process::Process`] struct.
#[derive(Debug, Copy, Clone)]
pub struct ProcessDescriptorTable {
    /// Associates a file descriptor to the index of the open file table
    /// in the [`GLOBAL_FILE_TABLE`]
    files: [Option<usize>; MAX_TOTAL_OPEN_FILES_BY_PROCESS],
}

impl ProcessDescriptorTable {
    pub const fn init() -> Self {
        Self {
            files: [None; MAX_TOTAL_OPEN_FILES_BY_PROCESS],
        }
    }

    /// Returns reference to filetable from a filedescriptor.
    pub fn get_file_table(&self, fd: FileDescriptor) -> Result<&'static OpenFileTable, FileDesciptorError> {
        if let Some(id) = self.files[fd.into_usize()] {
            Ok(unsafe { GLOBAL_FILE_TABLE.get_file_table_ref(id) })
        } else {
            Err(FileDesciptorError())
        }
    }

    pub fn add_file_table(&mut self, open_file_table: OpenFileTable) -> FileDescriptor {
        // ! This `3` if temporary, only for test purposes
        let mut i = 3;
        while i < MAX_TOTAL_OPEN_FILES_BY_PROCESS {
            if self.files[i].is_none() {
                // File descriptor to be returned
                break;
            }
            i += 1;
        }
        if i == MAX_TOTAL_OPEN_FILES_BY_PROCESS {
            panic!("Too many opened files by process.");
        } else {
            let fd = i;
            let index = unsafe { GLOBAL_FILE_TABLE.insert(open_file_table) };
            self.files[i] = Some(index);
            FileDescriptor::new(fd)
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
        let open_file_table = OpenFileTable::new(_path, _flags);
        self.add_file_table(open_file_table)
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
