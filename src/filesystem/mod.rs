use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
pub mod disk_operations;
pub mod test;
pub mod ustar;

use crate::{print, println};

/// Main cache for Path -> Adress conversion.
/// Used to speed-up filesystem quarries while only allocating a small amount of data.
///
/// For instance, we don't (at leat for now) store files, the filesystem has to
/// fetch a file from disk every time it is requested.
static FILE_ADRESS_CACHE: FileCache = FileCache(BTreeMap::new());

static DIR_CACHE : DirCache = DirCache(BTreeMap::new());

#[derive(Debug, PartialEq)]
pub struct FileSystemError(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(String);

impl Path {
    fn new() -> Self {
        Self(String::new())
    }
    fn from(s: String) -> Self {
        Self(s)
    }
    // We might wanna to avoid cloning string everywhere...
    fn to(&self) -> String {
        self.0.clone()
    }
    fn owned_to(self) -> String {
        self.0
    }
    fn slice(&self) -> Vec<String> {
        let sliced = self
            .to()
            .split('\\')
            .map(String::from)
            .collect::<Vec<String>>();
        sliced
    }
}

#[derive(Debug)]
struct MemDir{
    name : String,
    address : ustar::Address,
    files : BTreeMap<String, ustar::Address>,
}

#[derive(Debug)]
struct FileCache(BTreeMap<Path, ustar::Address>);

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

pub fn write_file(_path: Path, _data : &'static [u8]) {
    todo!();
}

fn test() {
    println!(
        "{:?}",
        open_file(Path::from(String::from("test")), OpenMode::Read)
    );
}
