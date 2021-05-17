//! Flags

#![allow(clippy::upper_case_acronyms)]

use alloc::collections::BTreeSet;
use bitflags::bitflags;
use core::ops::BitAnd;
use core::slice::Iter;

bitflags! {
    #[repr(transparent)]
    pub struct OpenFlags: usize {
        const ORD = 1;
        const OWR = 1 << 1;
        const OCREAT = 1 << 2;
        const OAPPEND = 1 << 3;
        const OXCUTE = 1 << 4;
    }
}

impl OpenFlags {
    pub fn from(f: usize) -> Self {
        OpenFlags::ORD
    }
}

/*
/// Flags used to open a file. May be incomplete
#[repr(usize)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum OpenFlags {
    ONONE = 0,
    ORDO = 1,         // read-only
    OWRO = 1 << 1,    // write-only
    ORDWR = 1 << 2,   // read and write
    OCREAT = 1 << 3,  // Create the file if doesn't exist
    OAPPEND = 1 << 4, // Writes at the end of the file
    OXCUTE = 1 << 5,  // Read-only to execute it
}
#[allow(clippy::from_over_into)] // voluntary, as the other way is undefined
impl Into<usize> for OpenFlags {
    fn into(self) -> usize {
        let ptr: *const OpenFlags = &self;
        unsafe { *(ptr as *const usize) }
    }
}

impl BitAnd<usize> for OpenFlags {
    type Output = usize;

    fn bitand(self, rhs: usize) -> Self::Output {
        self as usize & rhs
    }
}

impl OpenFlags {
    pub const ALL_FLAGS: [OpenFlags; 6] = [
        Self::ORDO,
        Self::OWRO,
        Self::ORDWR,
        Self::OCREAT,
        Self::OAPPEND,
        Self::OXCUTE,
    ];

    pub fn iter() -> Iter<'static, OpenFlags> {
        Self::ALL_FLAGS.iter()
    }

    pub fn parse(flag_code: usize) -> BTreeSet<OpenFlags> {
        let mut flag_set = BTreeSet::new();
        for f in OpenFlags::iter() {
            if (*f) & flag_code != 0 {
                flag_set.insert(*f);
            }
        }
        flag_set
    }
}
*/
