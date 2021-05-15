use super::ProcessError;
use crate::data_storage::path::Path;
use crate::filesystem::read_file_from_path;
use crate::memory;
use crate::{debug, errorln, warningln};
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::max;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;
use xmas_elf::{program::SegmentData, program::Type, sections::ShType, ElfFile};

pub const MODIFY_WITH_EXEC: PageTableFlags = PageTableFlags::BIT_9;
pub const STACK: PageTableFlags = PageTableFlags::BIT_10;
pub const HEAP: PageTableFlags = PageTableFlags::BIT_11;
pub const HEAP_ADDED: PageTableFlags = PageTableFlags::BIT_52;

// TODO : change this to respect the conventions
// For now, it is very probably wrong
// for certain writable segment types
pub fn get_table_flags(section: ShType) -> PageTableFlags {
    match section {
        ShType::ProgBits => PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
        ShType::SymTab => PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
        ShType::StrTab => PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
        ShType::NoBits => {
            PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT | PageTableFlags::ACCESSED
        }
        _ => {
            PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::PRESENT
                | PageTableFlags::NO_EXECUTE
                | PageTableFlags::WRITABLE
        }
    }
}

#[allow(dead_code)]
const PROG_OFFSET: u64 = 0x8048000000;

pub const ADDR_STACK: u64 = 0x1ffff8;

pub const MINIMAL_HEAP_SIZE: u64 = 100;

/// # Safety
/// TODO
pub unsafe fn load_elf_for_exec(file_name: &str, args: &Vec<String>) -> Result<!, ProcessError> {
    let frame_allocator = match &mut memory::FRAME_ALLOCATOR {
        Some(fa) => fa,
        None => panic!("the frame allocator wasn't initialized"),
    };
    let code: &[u8] = &read_file_from_path(Path::from(file_name));
    if code.len() == 0 {
        debug!("tries to return");
        return Err(ProcessError::InvalidExec);
    }

    let mut args2 = Vec::new();
    debug!("Copy of the data {}", args.len());
    for i in args {
        debug!("{}", i.len());
        debug!("{}", i);
        args2.push(String::from(i))
    }

    warningln!("Code len 1 => {}", code.len());

    if let Ok(_level_4_table_addr) = frame_allocator.allocate_level_4_frame() {
        let current = super::get_current();

        // deallocate precedent file
        if !frame_allocator.deallocate_level_4_page(current.cr3, MODIFY_WITH_EXEC, true) {
            debug!("mod_with_exec page table is not empty")
        } else {
            debug!("mod_with_exec page table is empty")
        }

        // deallocate precedent heap
        if !frame_allocator.deallocate_level_4_page(current.cr3, HEAP, true) {
            debug!("heap page table is not empty")
        } else {
            debug!("heap page table is empty")
        }

        super::disassemble_and_launch(code, frame_allocator, 0, 0, &args2, false)
    } else {
        Err(ProcessError::AllocatorError)
    }
}

/*
/// # Safety
/// Never safe ! You just need to know what you are doing before calling it
pub unsafe fn _load_elf_for_exec(_file_name: &str) -> ! {
    let frame_allocator = match &mut memory::FRAME_ALLOCATOR {
        Some(fa) => fa,
        None => panic!("the frame allocator wasn't initialized"),
    };

    let elf = ElfFile::new(code).unwrap();

    // We get the main entry point and make sure it is
    // a 64-bit ELF file
    let prog_entry = match elf.header.pt2 {
        xmas_elf::header::HeaderPt2::Header64(a) => a.entry_point,
        _ => panic!("Expected a 64-bit ELF!"),
    };

    if let Ok(level_4_table_addr) = frame_allocator.allocate_level_4_frame() {
        let mut current = super::get_current_as_mut();

        // deallocate precedent file
        if !frame_allocator.deallocate_level_4_page(current.cr3, MODIFY_WITH_EXEC, true) {
            debug!("page table is not empty")
        } else {
            debug!("page table is empty")
        }

        // deallocate precedent heap
        let b = frame_allocator.deallocate_level_4_page(current.cr3, HEAP_ADDED, true);
        if !b {
            debug!("page table is not empty")
        } else {
            debug!("page table is empty")
        }

        let mut maximum_address = 0;

        for program in elf.program_iter() {
            // Characteristics of the section
            let address = program.virtual_addr();
            let offset = program.offset();
            let size = program.mem_size();
            let file_size = program.file_size();
            maximum_address = max(maximum_address, address + size);

            match program.get_type() {
                Ok(Type::Phdr) | Err(_) => continue,
                Ok(_) => (),
            };
            if address == 0 {
                continue;
            }

            let mut zeroed_data = Vec::new();
            let _data = match program.get_type().unwrap() {
                Type::Load => match program.get_data(&elf).unwrap() {
                    SegmentData::Undefined(a) => a,
                    SegmentData::Note64(_, a) => a,
                    _ => panic!(":("),
                },
                _ => {
                    for _ in 0..size {
                        zeroed_data.push(0)
                    }
                    &zeroed_data[..]
                }
            };

            let num_blocks = (size + offset) / 4096 + 1;

            let mut flags =
                PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | MODIFY_WITH_EXEC;
            if program.flags().is_write() {
                flags |= PageTableFlags::WRITABLE;
            }
            if !program.flags().is_execute() {
                flags |= PageTableFlags::NO_EXECUTE;
            }

            for i in 0..num_blocks {
                // Allocate a frame for each page needed.
                match frame_allocator.add_entry_to_table(
                    level_4_table_addr,
                    VirtAddr::new(address + (i as u64) * 0x1000),
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
                        //hardware::power::shutdown();
                    }
                }
            }

            match memory::write_into_virtual_memory(
                level_4_table_addr,
                VirtAddr::new(address),
                _data,
            ) {
                Ok(()) => (),
                Err(a) => errorln!("{:?} at section : {:?}", a, 0),
            };
            if size != file_size {
                warningln!(
                    "file_size and mem_size differ : file {}, mem {}",
                    file_size,
                    size
                );
                let mut padding = Vec::new();
                padding.resize(file_size as usize, 0_u8);
                memory::write_into_virtual_memory(
                    level_4_table_addr,
                    VirtAddr::new(address + size),
                    &padding[..],
                )
                .unwrap();
            }
        }
        current.heap_size = MINIMAL_HEAP_SIZE;
        debug!("Going towards user");
        debug!(
            "{:x} {:x} {:x}",
            current.stack_base,
            prog_entry,
            super::towards_user_give_heap as usize
        );
        super::towards_user_give_heap(
            current.heap_address,
            current.heap_size,
            current.stack_base,
            prog_entry,
        );
    } else {
        panic!("could not launch process")
    }
}
*/
