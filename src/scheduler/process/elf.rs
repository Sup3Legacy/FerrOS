use x86_64::structures::paging::PageTableFlags;
use xmas_elf::{sections::ShType, ElfFile};
use x86_64::VirtAddr;
use alloc::string::String;
use x86_64::structures::paging::PhysFrame;
use crate::memory;
use crate::println;
use crate::errorln;

use super::get_current;

pub const MODIFY_WITH_EXEC: PageTableFlags = PageTableFlags::BIT_9;

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

const PROG_OFFSET:u64 = 0x8048000000;

pub const ADDR_STACK: u64 = 0x63fffffffff8;

/// # Safety
/// Never safe ! You just need to know what you are doing before calling it
pub unsafe fn load_elf_for_exec(_file_name: &String) -> VirtAddr {
    let code: &[u8] = &[]; // /!\ need to be implemented in the filesystem
    
    if let Some(frame_allocator) = &mut memory::FRAME_ALLOCATOR {
    
        let current = get_current();
        // deallocate precedent file
        match frame_allocator.deallocate_level_4_page(current.cr3, MODIFY_WITH_EXEC) {
            Ok(b) => if !b {panic!("page table is now empty")},
            Err(_) => panic!("failed at deallocation"),
        };

        // We get the `ElfFile` from the raw slice
        let elf = ElfFile::new(code).unwrap();
        // We get the main entry point and make sure it is
        // a 64-bit ELF file
        let prog_entry = match elf.header.pt2 {
            xmas_elf::header::HeaderPt2::Header64(a) => a.entry_point,
            _ => panic!("Expected a 64-bit ELF!"),
        };

        let level_4_table_addr = PhysFrame::containing_address(current.cr3);
        // Loop over each section
        for section in elf.section_iter() {
            // Characteristics of the section
            let address = section.address();
            let offset = section.offset();
            let size = section.size();
            // Section debug
            println!(
                "Block, address : 0x{:x?}, offset : 0x{:x?}, size : 0x{:x?}, type : {:?}",
                address,
                offset,
                size,
                section.get_type()
            );

            match section.get_type() {
                Ok(ShType::Null) | Err(_) => continue,
                Ok(_) => (),
            };

            let _data = section.raw_data(&elf);
            let total_length = _data.len() as u64 + offset;
            let num_blocks = total_length / 4096 + 1;
            println!(
                "Total len of 0x{:x?}, {:?} blocks",
                num_blocks * 512,
                num_blocks
            );


            let flags = get_table_flags(section.get_type().unwrap());
            for i in 0..num_blocks {
                // Allocate a frame for each page needed.
                match frame_allocator.add_entry_to_table(
                    level_4_table_addr,
                    VirtAddr::new(address + (i as u64) * 4096 + PROG_OFFSET),
                    flags,
                    true,
                ) {
                    Ok(()) => (),
                    Err(memory::MemoryError(err)) => {
                        errorln!(
                            "Could not allocate the {}-th part of the code. Error : {:?}",
                            i,
                            err
                        );
                    }
                }
            }
            match memory::write_into_virtual_memory(
                level_4_table_addr,
                VirtAddr::new(address + PROG_OFFSET),
                _data,
            ) {
                Ok(()) => (),
                Err(a) => errorln!("{:?} at section : {:?}", a, section),
            };
        }
        VirtAddr::new(prog_entry + PROG_OFFSET)
    } else {
        panic!("should not happen");
    }
}