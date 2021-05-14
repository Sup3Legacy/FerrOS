use crate::data_storage::path::Path;

use crate::scheduler::process;
use alloc::string::String;

pub struct FileDesciptorError();

/// Max number of total opened files
const MAX_TOTAL_OPEN_FILES: usize = 256;

/// Max number of openable files by a process
const MAX_TOTAL_OPEN_FILES_BY_PROCESS: usize = 16;

static mut GLOBAL_FILE_TABLE: GeneralFileTable = GeneralFileTable::new();

/// Contains all the open_file_tables
#[derive(Clone, Debug)]
pub struct GeneralFileTable {
    /// Array that maps a fdindex to a OpenFileTable (ie all the relevant metadata on the given file)
    tables: [Option<OpenFileTable>; MAX_TOTAL_OPEN_FILES as usize],
    /// Index of the first unoccupied space in the table
    index: usize,
}

impl GeneralFileTable {
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
        match &mut self.tables[index] {
            Some(file) => {
                if file.close() {
                    crate::debug!("was closed");
                    super::close_file(&file);
                    self.tables[index] = None;
                } else {
                    crate::debug!("wasn't closed");
                }
            }
            None => panic!("Unexisting file was closed"),
        }
    }

    pub fn duplicate(&mut self, fd: usize) -> usize {
        match &mut self.tables[fd] {
            None => panic!("should not happend"),
            Some(oft) => oft.duplicate(),
        }
        fd
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
    flags: u64,
    offset: usize,
    id: usize,
    nb: usize,
}
impl OpenFileTable {
    pub fn new(path: Path, flags: u64, id: usize) -> Self {
        Self {
            path,
            flags,
            offset: 0,
            id,
            nb: 1,
        }
    }
    pub fn get_path(&self) -> Path {
        self.path.clone()
    }
    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn duplicate(&mut self) {
        self.nb += 1;
    }

    pub fn close(&mut self) -> bool {
        self.nb -= 1;
        self.nb == 0
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
#[derive(Debug, Copy, Clone)]
pub struct ProcessDescriptorTable {
    /// Associates a file descriptor to the index of the open file table
    /// in the [`GLOBAL_FILE_TABLE`]
    files: [Option<usize>; MAX_TOTAL_OPEN_FILES_BY_PROCESS],
    index: usize,
}

impl ProcessDescriptorTable {
    pub const fn init() -> Self {
        Self {
            files: [None; MAX_TOTAL_OPEN_FILES_BY_PROCESS],
            index: 0,
        }
    }

    /// Returns reference to filetable from a filedescriptor.
    pub fn get_file_table(
        &self,
        fd: FileDescriptor,
    ) -> Result<&'static OpenFileTable, FileDesciptorError> {
        if let Some(id) = self.files[fd.into_usize()] {
            Ok(unsafe { GLOBAL_FILE_TABLE.get_file_table_ref(id) })
        } else {
            Err(FileDesciptorError())
        }
    }

    pub fn is_none(&self, i: usize) -> bool {
        self.files[i].is_none()
    }

    pub fn add_file_table(&mut self, open_file_table: OpenFileTable) -> FileDescriptor {
        // ! This `3` if temporary, only for test purposes
        let mut i = 1;
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
    pub fn create_file_table(&mut self, path: Path, flags: u64) -> FileDescriptor {
        // Here we create a new OpenFileTable.
        // We fill it with all the passed values,
        // inserts it into the GLOBAL_FILE_TABLE
        // and finally add an entry to the index in
        // the GLOBAL_FILE_TABLE into the first
        // unoccupied FileDescriptor field.
        // We then return the associated FileDescriptor
        let id = match super::open_file(&path, super::open_mode_from_flags(flags)) {
            Ok(i) => i,
            Err(_) => {
                return FileDescriptor::new(usize::MAX)
            },
        };
        let open_file_table = OpenFileTable::new(path, flags, id);
        self.add_file_table(open_file_table)
    }

    /// self.dup(1, 4) redirects fd 1 to the OpenFileTable
    /// fd 4 points to.
    pub fn dup(&mut self, target: FileDescriptor, operand: FileDescriptor) -> usize {
        // TO DO check the bounds and validity of the given data!
        crate::debug!("Dup from descriptor {} -> {}", target.into_usize(), self.files[target.into_usize()].is_none());
        match self.files[target.into_usize()] {
            None => (),
            Some(fd) => unsafe {
                GLOBAL_FILE_TABLE.delete(fd);
            },
        }
        self.files[target.into_usize()] = self.files[operand.into_usize()];
        match self.files[operand.into_usize()] {
            None => (),
            Some(fd) => unsafe {
                GLOBAL_FILE_TABLE.duplicate(fd);
            },
        }
        0
    }

    pub fn copy(&mut self, father: ProcessDescriptorTable) {
        for i in 0..MAX_TOTAL_OPEN_FILES_BY_PROCESS {
            match father.files[i] {
                None => self.files[i] = None,
                Some(fd) => {
                    unsafe {
                        GLOBAL_FILE_TABLE.duplicate(fd);
                    }
                    self.files[i] = Some(fd);
                }
            }
        }
    }

    pub unsafe fn close(&mut self) {
        for i in 0..MAX_TOTAL_OPEN_FILES_BY_PROCESS {
            match self.files[i] {
                Some(fd) => {
                    GLOBAL_FILE_TABLE.delete(fd);
                }
                _ => (),
            }
        }
    }
}

pub fn open(filename: String, mode: super::OpenMode) -> FileDescriptor {
    let current_process = unsafe { process::get_current_as_mut() };
    // look for a place to put the next file in the process file descriptor table
    let mut new_pfdt = current_process.open_files;
    for i in 0..(MAX_TOTAL_OPEN_FILES_BY_PROCESS - 1) {
        if new_pfdt.index == 0 {
            new_pfdt.index = 1;
        }
        match new_pfdt.files[new_pfdt.index] {
            Some(_) => {
                new_pfdt.index = (new_pfdt.index + 1) % MAX_TOTAL_OPEN_FILES_BY_PROCESS;
                if i == MAX_TOTAL_OPEN_FILES_BY_PROCESS - 1 {
                    panic!("Cannot open any new file for current process");
                }
            }
            None => {
                new_pfdt.index = i;
                let id = match super::open_file(&Path::from(&filename), mode) {
                    Ok(i) => i,
                    Err(_) => return FileDescriptor::new(usize::MAX),
                };
                unsafe {
                    new_pfdt.files[i] = Some(GLOBAL_FILE_TABLE.insert(OpenFileTable::new(
                        Path::from(filename.as_str()),
                        0,
                        id,
                    )));
                }
                current_process.open_files = new_pfdt;
                return FileDescriptor::new(current_process.open_files.index);
            }
        };
    }
    current_process.open_files = new_pfdt;
    FileDescriptor::new(current_process.open_files.index)
}

pub fn close(descriptor: u64) -> Result<(), FileDesciptorError> {
    let current_proccess = unsafe { process::get_current_as_mut() };
    // we try to close the file, and if at any point we fail, raise an error
    match current_proccess.open_files.files[descriptor as usize] {
        None => Err(FileDesciptorError()),
        Some(idx) => {
            current_proccess.open_files.files[descriptor as usize] = None;
            unsafe {
                GLOBAL_FILE_TABLE.delete(idx);
                Ok(())
            }
        }
    }
}
