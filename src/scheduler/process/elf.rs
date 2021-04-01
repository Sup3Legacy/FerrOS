use x86_64::structures::paging::PageTableFlags;
use xmas_elf::{sections::ShType, ElfFile};


// TODO : change this to respect the conventions
// For now, it is very probably wrong
// for certain writable segment types
pub fn get_table_flags(section: ShType) -> PageTableFlags {
    match section {
        ShType::ProgBits => PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
        ShType::SymTab => PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
        ShType::StrTab => PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
        _ => {
            PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::PRESENT
                | PageTableFlags::NO_EXECUTE
                | PageTableFlags::WRITABLE
        }
    }
}

