/// Max number of total opened files
const MAX_TOTAL_OPEN_FILES: usize = 256;

static mut GLOBAL_FILE_TABLE: GeneralFileTable = GeneralFileTable::new();

/// Contains all the open_file_tables
pub struct GeneralFileTable {
    /// Array containing all the filetables
    tables: [Option<OpenFileTable>; MAX_TOTAL_OPEN_FILES],
    /// Index of the smallest unoccupied space in the table
    index: usize,
}

impl GeneralFileTable {
    pub const fn new() -> Self {
        Self {
            tables: [None; MAX_TOTAL_OPEN_FILES],
            index: 0,
        }
    }
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
}

#[derive(Debug, Copy, Clone)]
pub struct OpenFileTable {}
pub struct ProcessDescriptorTable {}
