/*
use alloc::sync::Arc;
use core::{borrow::BorrowMut, ops::DerefMut};
use core::cell::RefCell;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use lazy_static::{__Deref, lazy_static};
use spin::{Mutex, MutexGuard};
*/
use alloc::collections::BTreeMap;
use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;

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

pub struct FileSystem {
    root: FileNode,
}

lazy_static! {
    static ref ROOT: Mutex<FileSystem> = {
        let root = Mutex::new(FileSystem {
            root: FileNode::Dir(String::from(""), BTreeMap::new()),
        });
        root
    };
}

pub fn fetch_file(path: String) -> Result<(), FileSystemError> {
    let mut split_path = path.split("/");
    let mut current = &mut ROOT.lock().root;
    while let Some(item) = split_path.next() {
        if let FileNode::Dir(_name, hash_table) = current {
            current = {
                match hash_table.get_mut(item) {
                    Some(c) => c,
                    None => return Err(FileSystemError("File doesn't exist.".into())),
                }
            };
        }
    }
    Ok(())
}
