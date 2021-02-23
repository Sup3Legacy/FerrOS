use alloc::sync::Arc;
use core::{borrow::BorrowMut, ops::DerefMut};
use core::cell::RefCell;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use lazy_static::{__Deref, lazy_static};
use spin::{Mutex, MutexGuard};

pub struct FileSystemError(String);

pub struct File();

/// Base type for the filesystem.
/// A `FileNode` is either a directory, containing an arbitrary number of further nodes, or a file
pub enum FileNode {
    /// Directory type.
    Dir(String, BTreeMap<String, FileNode>),
    /// TODO add field in `File` for the code and data frame
    File(String, File),
}

lazy_static! {
    static ref ROOT: Mutex<FileNode> = {
        let root = Mutex::new(FileNode::Dir(String::from(""), BTreeMap::new()));
        root
    };
}

pub unsafe fn find_node() -> Result<&'static mut FileNode, FileSystemError> {
    Ok(ROOT.lock().deref_mut())
}

pub fn fetch_file(path: String) -> Result<&'static FileNode, FileSystemError> {
    let split_path  = path.split("/");
    let mut current = ROOT.lock().deref();
    while let Some(item) = split_path.next() {
        if let FileNode::Dir(name, hash_table) = current {
            current = {
            match hash_table.get(item) {
                Some(c) => c,
                None => return Err(FileSystemError("File doesn't exist.".into())),
            }
        }
    }
    }
    Ok(current)
}
