#![allow(clippy::upper_case_acronyms)]

use alloc::collections::BTreeSet;
use core::ops::BitAnd;
use core::slice::Iter;

/// Flags used to open a file. May be incomplete
#[repr(u64)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum OpenFlags {
    ORDO = 1,         // read-only
    OWRO = 1 << 1,    // write-only
    ORDWR = 1 << 2,   // read and write
    OCREAT = 1 << 3,  // Create the file if doesn't exist
    OAPPEND = 1 << 4, // Writes at the end of the file
}
impl Into<u64> for OpenFlags {
    fn into(self) -> u64 {
        let ptr: *const OpenFlags = &self;
        unsafe { *(ptr as *const u64) }
    }
}

impl BitAnd<u64> for OpenFlags {
    type Output = u64;

    fn bitand(self, rhs: u64) -> Self::Output {
        self as u64 & rhs
    }
}

impl OpenFlags {
    pub const ALL_FLAGS: [OpenFlags; 5] = [
        Self::ORDO,
        Self::OWRO,
        Self::ORDWR,
        Self::OCREAT,
        Self::OAPPEND,
    ];

    pub fn iter() -> Iter<'static, OpenFlags> {
        Self::ALL_FLAGS.iter()
    }

    pub fn parse(flag_code: u64) -> BTreeSet<OpenFlags> {
        let mut flag_set = BTreeSet::new();
        for f in OpenFlags::iter() {
            if (*f) & flag_code != 0 {
                flag_set.insert(*f);
            }
        }
        flag_set
    }
}
