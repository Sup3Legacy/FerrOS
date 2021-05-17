//! All the logic around `Process`

use super::PROCESS_MAX_NUMBER;

use bit_field::BitField;
use core::{
    cmp::max,
    sync::atomic::{AtomicU64, Ordering},
};
//use lazy_static::lazy_static;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::PageTableFlags;
use x86_64::structures::paging::PhysFrame;
use x86_64::{PhysAddr, VirtAddr};

use xmas_elf::{program::SegmentData, program::Type, ElfFile};

use crate::alloc::collections::{BTreeMap, BTreeSet};
use crate::alloc::vec::Vec;
use crate::data_storage::{path::Path, queue::Queue, random};
use crate::filesystem;
use crate::filesystem::descriptor::{FileDescriptor, ProcessDescriptorTable};
use crate::filesystem::fsflags::OpenFlags;
use crate::hardware;
use crate::memory;
use crate::{debug, errorln, println};
use alloc::string::String;

/// Default allocated heap size (in number of pages)
const DEFAULT_HEAP_SIZE: u64 = 2;

pub mod elf;

#[derive(Debug)]
pub enum ProcessError {
    InvalidELFHeader,
    AllocatorError,
    WriteError,
    StackError,
    HeapError,
    InvalidExec,
    ReadError,
}

#[allow(improper_ctypes)]
extern "C" {
    fn launch_asm(first_process: fn(), initial_rsp: u64);

    /// Old function definition
    pub fn _leave_context(rsp: u64);
}

#[naked]
/// # Safety
///
/// Highgly unsafe function!
///
/// Given `Cr3` and `rsp` values, it leaves the context.
pub unsafe extern "C" fn leave_context_cr3(_cr3: u64, _rsp: u64) {
    asm!(
        "mov cr3, rdi",
        "mov rsp, rsi",
        "pop r9",
        "pop r8",
        "pop r10",
        "pop rdx",
        "pop rsi",
        "pop rdi",
        "pop rax",
        "pop rbx",
        "pop rcx",
        "pop rbp",
        "pop r11",
        "pop r12",
        "pop r13",
        "pop r14",
        "pop r15",
        "vmovaps ymm0, [rsp]",
        "add rsp, 32",
        //"sti",
        "iretq",
        options(noreturn,),
    )
}

#[naked]
/// # Safety
/// TODO
pub unsafe extern "C" fn leave_context(_rsp: u64) {
    asm!(
        "mov rsp, rdi",
        "pop r9",
        "pop r8",
        "pop r10",
        "pop rdx",
        "pop rsi",
        "pop rdi",
        "pop rax",
        "pop rbx",
        "pop rcx",
        "pop rbp",
        "pop r11",
        "pop r12",
        "pop r13",
        "pop r14",
        "pop r15",
        "vmovaps ymm0, [rsp]",
        "add rsp, 32",
        //"sti",
        "iretq",
        options(noreturn,),
    )
}

#[naked]
/// # Safety
/// TODO
///
/// Goes towards the userland with a stack- and an instruction-pointer
pub unsafe extern "C" fn towards_user(_rsp: u64, _rip: u64) {
    asm!(
        // Ceci n'est pas exécuté
        "mov rax, 0x0", // data segment
        "mov ds, eax",
        "mov es, eax",
        "mov fs, eax",
        "mov gs, eax",
        "mov rsp, rdi",
        "add rsp, 8",
        "push 0x42",
        "push rax",  // stack segment
        "push rdi",  // stack pointer
        "push 518",  // cpu flags
        "push 0x08", // code segment
        "push rsi",  // instruction pointer
        "mov rax, 0",
        "mov rbx, 0",
        "mov rcx, 0",
        "mov rdx, 0",
        "mov rdi, 0",
        "mov rsi, 0",
        "mov rbp, 0",
        "mov r8, 0",
        "mov r9, 0",
        "mov r10, 0",
        "mov r11, 0",
        "mov r12, 0",
        "mov r13, 0",
        "mov r14, 0",
        "mov r15, 0",
        "iretq",
        options(noreturn,),
    )
}

#[naked]
/// # Safety
/// TODO
///
/// Goes towards the userland with a stack- and an instruction-pointer.
/// It is also given a heap.
pub unsafe extern "C" fn towards_user_give_heap(
    _heap_addr: u64,
    _heap_size: u64,
    _rsp: u64,
    _rip: u64,
) -> ! {
    asm!(
        // Ceci n'est pas exécuté
        "mov rax, 0x0", // data segment
        "mov ds, eax",
        "mov es, eax",
        "mov fs, eax",
        "mov gs, eax",
        "mov rsp, rdx",
        "add rsp, 8",
        "push 0x42",
        "push rax",  // stack segment
        "push rdx",  // stack pointer
        "push 518",  // cpu flags
        "push 0x08", // code segment
        "push rcx",  // instruction pointer
        "mov rax, 0",
        "mov rbx, 0",
        "mov rcx, 0",
        "mov rdx, 0",
        "mov rbp, 0",
        "mov r8, 0",
        "mov r9, 0",
        "mov r10, 0",
        "mov r11, 0",
        "mov r12, 0",
        "mov r13, 0",
        "mov r14, 0",
        "mov r15, 0",
        "iretq",
        options(noreturn,),
    )
}

/// # Safety
/// TODO
///
/// Goes towards the userland with a stack- and an instruction-pointer.
/// It is also given a heap and arguments.
pub unsafe extern "C" fn towards_user_give_heap_args(
    heap_addr: u64,
    heap_size: u64,
    args: u64,
    args_number: u64,
    rsp: u64,
    rip: u64,
) -> ! {
    asm!(
        // Ceci n'est pas exécuté
        "mov rax, 0x0", // data segment
        "mov ds, eax",
        "mov es, eax",
        "mov fs, eax",
        "mov gs, eax",
        "mov rsp, r8",
        "add rsp, 8",
        "push 0x42",
        "push rax",  // stack segment
        "push r8",  // stack pointer
        "push 518",  // cpu flags
        "push 0x08", // code segment
        "push r9",   // instruction pointer
        "mov rax, 0",
        "mov rbx, 0",
        //"mov rcx, 0", number of arguments
        //"mov rdx, 0", In this register we pass the pointer to the arguments
        "mov rbp, 0",
        "mov r8, 0",
        "mov r9, 0",
        "mov r10, 0",
        "mov r11, 0",
        "mov r12, 0",
        "mov r13, 0",
        "mov r14, 0",
        "mov r15, 0",
        "iretq",
        in("rdi") heap_addr,
        in("rsi") heap_size,
        in("rdx") args,
        in("rcx") args_number,
        in("r8") rsp,
        in("r9") rip,
        //options(noreturn,),
    );
    loop {}
}

/// # Safety
/// TODO
///
/// allocates a given number of additional pages to the process' heap.
pub unsafe fn allocate_additional_heap_pages(
    frame_allocator: &mut memory::BootInfoAllocator,
    start: u64,
    number: u64,
    process: &Process,
) -> u64 {
    let mut maxi = 0;
    for i in 0..number {
        match frame_allocator.add_entry_to_table(
            PhysFrame::containing_address(process.cr3),
            VirtAddr::new(start + i * 0x1000),
            PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | elf::HEAP,
            false,
        ) {
            Ok(()) => {
                maxi = i;
                match memory::write_into_virtual_memory(
                    PhysFrame::containing_address(process.cr3),
                    VirtAddr::new(start + i * 0x1000),
                    &[0_u8; 0x1000],
                ) {
                    Ok(()) => (),
                    Err(a) => errorln!("{:?} at additional heap-section : {:?}", a, i),
                };
            }
            Err(memory::MemoryError(err)) => {
                errorln!(
                    "Could not allocate the {}-th additional part of the heap. Error : {:?}",
                    i,
                    err
                );
            }
        }
    }
    maxi
}

/// Function to launch the first process !
/// # Safety
/// TODO
pub unsafe fn launch_first_process(
    frame_allocator: &mut memory::BootInfoAllocator,
    code_address: *const u8,
    number_of_block: u64,
    stack_size: u64,
) -> ! {
    ID_TABLE[0].state = State::Runnable;
    if let Ok(level_4_table_addr) = frame_allocator.allocate_level_4_frame() {
        // addresses telling where the code and the stack starts
        let addr_code: u64 = 0x320000000000;
        let addr_stack: u64 = 0x63fffffffff8;

        // put the code blocks at the right place
        for i in 0..number_of_block {
            let data: *const [u64; 512] =
                VirtAddr::from_ptr(code_address.add((i * 4096) as usize)).as_mut_ptr();
            match frame_allocator.add_entry_to_table_with_data(
                level_4_table_addr,
                VirtAddr::new(addr_code + i * 4096),
                PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
                &*data,
            ) {
                Ok(()) => (),
                Err(memory::MemoryError(err)) => {
                    errorln!(
                        "Could not allocate the {}-th part of the code. Error : {:?}",
                        i,
                        err
                    );
                    hardware::power::shutdown();
                }
            }
        }

        // allocate every necessary blocks on the stack
        for i in 0..stack_size {
            match frame_allocator.add_entry_to_table(
                level_4_table_addr,
                VirtAddr::new(addr_stack - i * 0x1000),
                PageTableFlags::USER_ACCESSIBLE
                    | PageTableFlags::PRESENT
                    | PageTableFlags::NO_EXECUTE
                    | PageTableFlags::WRITABLE,
                false,
            ) {
                Ok(()) => (),
                Err(memory::MemoryError(err)) => {
                    errorln!(
                        "Could not allocate the {}-th part of the stack. Error : {:?}",
                        i,
                        err
                    );
                    hardware::power::shutdown();
                }
            }
        }

        let (_cr3, cr3f) = Cr3::read();
        Cr3::write(level_4_table_addr, cr3f);
        println!("good luck user ;) {} {}", addr_stack, addr_code);
        println!("target : {:x}", towards_user as usize);
        towards_user(addr_stack, addr_code); // good luck user ;)

        // should not be reached
        hardware::power::shutdown();
    } else {
        errorln!("couldn't allocate a level 4 table");
        hardware::power::shutdown();
    }
}

/// Converts flags
pub fn page_table_flags_from_u64(flags: u64) -> PageTableFlags {
    let mut res = elf::MODIFY_WITH_EXEC | PageTableFlags::PRESENT;
    if flags.get_bit(0) {
        res |= PageTableFlags::PRESENT;
    }
    if flags.get_bit(1) {
        res |= PageTableFlags::WRITABLE;
    }
    res |= PageTableFlags::USER_ACCESSIBLE;
    if flags.get_bit(3) {
        res |= PageTableFlags::WRITE_THROUGH;
    }
    if flags.get_bit(4) {
        res |= PageTableFlags::NO_CACHE;
    }
    if flags.get_bit(5) {
        res |= PageTableFlags::ACCESSED;
    }
    if flags.get_bit(6) {
        res |= PageTableFlags::DIRTY;
    }
    if flags.get_bit(7) {
        res |= PageTableFlags::HUGE_PAGE;
    }
    if flags.get_bit(8) {
        res |= PageTableFlags::GLOBAL;
    }
    if flags.get_bit(63) {
        res |= PageTableFlags::NO_EXECUTE;
    }
    res
}

/// Flattens arguments into a pages. Strings are NULL-ended
pub fn flatten_arguments(args: &Vec<String>) -> (u64, [u8; 0x1000]) {
    let mut res = [0_u8; 0x1000];
    let mut index = 0;
    let mut args_number = 0;
    for arg in args {
        if index >= 0x1000 {
            res[0x1000 - 1] = 0;
            break;
        }
        let bytes = arg.as_bytes();
        let length = bytes.len();
        for i in 0..length {
            if i + index >= 0x1000 {
                // If the length of the arguments is too large
                // To make sure that, at least, the last argument is correctly terminated
                res[i + index - 1] = 0;
                break;
            }
            res[i + index] = bytes[i];
        }
        res[length + index] = 0;
        index += length + 1;
        args_number += 1;
    }
    (args_number, res)
}

/// Takes in a slice containing an ELF file,
/// disassembles it and executes the program.
///
/// TODO : maybe use `number_of_block` as the maximum
/// number of frames allocated to the program?
///
/// PROG_OFFSET is set arbitrary and may need some fine-tuning.
/// # Safety
/// TODO
#[allow(clippy::empty_loop)]
pub unsafe fn disassemble_and_launch(
    code: &[u8],
    frame_allocator: &mut memory::BootInfoAllocator,
    _number_of_block: u64,
    stack_size: u64,
    args: &Vec<String>,
    new_process: bool,
) -> Result<!, ProcessError> {
    // TODO maybe consider changing this
    let addr_stack: u64 = if new_process {
        0x00007ffffffffff8
    } else {
        get_current().stack_base
    };
    println!(
        "0x219000 was allocated ? {}",
        memory::check_if_has_flags(
            Cr3::read().0,
            VirtAddr::new(0x219000),
            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE,
        )
    );
    // We get the `ElfFile` from the raw slice
    println!("Code len : {}", code.len());
    let elf = ElfFile::new(code).map_err(|_| ProcessError::InvalidELFHeader)?;
    // We get the main entry point and mmake sure it is
    // a 64-bit ELF file
    let prog_entry = match elf.header.pt2 {
        xmas_elf::header::HeaderPt2::Header64(a) => a.entry_point,
        _ => panic!("Expected a 64-bit ELF!"),
    };
    // This allocates a new level-4 table
    let level_4_table_addr = if new_process {
        match frame_allocator.allocate_level_4_frame() {
            Ok(l4) => l4,
            Err(_) => panic!("no more memory available"),
        }
    } else {
        let (cr3, _) = Cr3::read();
        cr3
    };

    // TODO Change this
    ID_TABLE[0].state = State::Runnable;
    // This represents the very end of all loaded segments
    let mut maximum_address = 0;
    let args_len = args.len();
    // Loop over each section
    for program in elf.program_iter() {
        // Characteristics of the section
        let address = program.virtual_addr();
        let offset = program.offset();
        let size = program.mem_size();
        let file_size = program.file_size();

        println!(
            " code at {:x} {:x} {:x}",
            address,
            address + size,
            address + file_size
        );
        maximum_address = max(maximum_address, address + size);
        match program.get_type() {
            Err(_) => continue,
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
            Type::Dynamic => match program.get_data(&elf).unwrap() {
                SegmentData::Undefined(a) => a,
                SegmentData::Note64(_, a) => a,
                _ => panic!(":("),
            },
            Type::Interp => match program.get_data(&elf).unwrap() {
                SegmentData::Undefined(a) => a,
                SegmentData::Note64(_, a) => a,
                _ => panic!(":("),
            },
            Type::Tls => match program.get_data(&elf).unwrap() {
                SegmentData::Undefined(a) => a,
                SegmentData::Note64(_, a) => a,
                _ => panic!(":("),
            },
            Type::GnuRelro => match program.get_data(&elf).unwrap() {
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
        let num_blocks = (file_size + 0xFFF) / 0x1000 + 1;
        let mut flags =
            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | elf::MODIFY_WITH_EXEC;
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
                }
            }
        }
        match memory::write_into_virtual_memory(level_4_table_addr, VirtAddr::new(address), _data) {
            Ok(()) => (),
            Err(a) => errorln!("{:?} at section : {:?}", a, 0),
        };
        if size != file_size {
            println!(
                "file_size and mem_size differ : file {}, mem {}",
                file_size, size
            );
            let mut padding = Vec::new();
            padding.resize(file_size as usize, 0_u8);
            memory::write_into_virtual_memory(
                level_4_table_addr,
                VirtAddr::new(address + size),
                &padding[..],
            )
            .map_err(|_| ProcessError::WriteError)?;
        }
    }
    // Allocate frames for the stack
    for i in 0..stack_size {
        match frame_allocator.add_entry_to_table(
            level_4_table_addr,
            VirtAddr::new(addr_stack - i * 0x1000),
            PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::PRESENT
                | PageTableFlags::NO_EXECUTE
                | PageTableFlags::WRITABLE
                | elf::STACK,
            true,
        ) {
            Ok(()) => (),
            Err(memory::MemoryError(err)) => {
                errorln!(
                    "Could not allocate the {}-th part of the stack. Error : {:?}",
                    i,
                    err
                );
                return Err(ProcessError::StackError);
            }
        }
    }
    // Allocate pages for the heap
    // We define the heap start address as
    let heap_address = maximum_address + 0x8000_u64;
    let heap_address_normalized = heap_address - (heap_address % 0x1000);
    let heap_size = DEFAULT_HEAP_SIZE;

    for i in 0..heap_size {
        match frame_allocator.add_entry_to_table(
            level_4_table_addr,
            VirtAddr::new(heap_address_normalized + i * 0x1000),
            PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | elf::HEAP,
            false,
        ) {
            Ok(()) => println!("Allocated {:x}", heap_address_normalized + i * 0x1000),
            Err(memory::MemoryError(err)) => {
                errorln!(
                    "Could not allocate the {}-th part of the heap. Error : {:?}",
                    i,
                    err
                );
            }
        }
        match memory::write_into_virtual_memory(
            level_4_table_addr,
            VirtAddr::new(heap_address_normalized + i * 0x1000),
            &[0_u8; 0x1000],
        ) {
            Ok(()) => (),
            Err(a) => errorln!("{:?} at heap-section : {:?}", a, i),
        };
    }

    // Allocate a page for the process's arguments.
    let args_address = 0x1000;
    match frame_allocator.add_entry_to_table(
        level_4_table_addr,
        VirtAddr::new(args_address),
        PageTableFlags::USER_ACCESSIBLE
            | PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | elf::HEAP,
        false,
    ) {
        Ok(()) => (),
        Err(memory::MemoryError(err)) => {
            errorln!("Could not allocate the args page. Error : {:?}", err);
        }
    };
    debug!("Gonna flatten arguments : {:?}", args.len());
    debug!("args : {:?}", args);
    let (args_number, args_data) = flatten_arguments(args);
    // Write the arguments onto the process's memory
    debug!("Gonna write arguments");
    match memory::write_into_virtual_memory(
        level_4_table_addr,
        VirtAddr::new(args_address),
        &args_data,
    ) {
        Ok(()) => (),
        Err(a) => errorln!("Error when writing arguments : {:?}", a),
    };

    get_current_as_mut().heap_address = heap_address_normalized;
    get_current_as_mut().heap_size = heap_size;
    if new_process {
        get_current_as_mut().stack_base = addr_stack;
    }

    let (_cr3, cr3f) = Cr3::read();
    Cr3::write(level_4_table_addr, cr3f);
    println!("good luck user ;) {:x} {:x}", addr_stack, prog_entry);
    println!("target : {:x}", prog_entry);
    Ok(towards_user_give_heap_args(
        heap_address_normalized,
        heap_size,
        args_address,
        args_number,
        addr_stack,
        prog_entry,
    ))
}

/// Main structure of a process.
/// It contains all informations about a process and its operating frame.
/// It is based on the x86 structure of the TSS.
///
/// # Fields
/// * `pid` - the id of the process (unique)
/// * `ppid` - its parent's (i.e. the process that spawned it) id
/// * `priority` - the priority, used by the scheduler (not used for now)
/// * `quantum` - the number of consecutive quanta the process has already been running for
/// * `cr3` - pointer to its 1st order VM table. TO DO : replace it with a PhysFrame or PhysAddr
/// * `cr3f` - cr3 flags ???
/// * `rip` - current value of the instruction pointer
/// * `state` - state of the process (e.g. Zombie, Runnable...)
/// * `owner` - owner ID of the process (can be root or user) usefull for syscalls

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Process {
    pid: ID,
    ppid: ID,
    priority: Priority,
    quantum: u64,
    pub cr3: PhysAddr,
    pub cr3f: Cr3Flags,
    pub rsp: u64, // every registers are saved on the stack
    pub stack_base: u64,
    pub state: State,
    owner: u64,
    pub heap_address: u64,
    pub heap_size: u64,
    pub open_files: ProcessDescriptorTable,
    //pub screen: VirtualScreenID,
}

impl Process {
    pub fn create_new(parent: ID, priority: Priority, owner: u64) -> Self {
        let new_pid = ID::new();
        unsafe {
            CHILDREN.insert(new_pid, BTreeSet::new());
        }
        Self {
            pid: new_pid,
            ppid: parent,
            priority,
            quantum: 0_u64,
            cr3: PhysAddr::zero(),
            cr3f: Cr3Flags::empty(),
            rsp: 0,
            stack_base: 0,
            state: State::Runnable,
            owner,
            heap_address: 0,
            heap_size: 0,
            open_files: ProcessDescriptorTable::init(),
            //screen: VirtualScreenID::new(),
        }
    }

    pub const fn missing() -> Self {
        Self {
            pid: ID(0),
            ppid: ID(0),
            priority: Priority(0),
            quantum: 0_u64,
            cr3: PhysAddr::zero(),
            cr3f: Cr3Flags::empty(),
            rsp: 0,
            stack_base: 0,
            state: State::SlotAvailable,
            owner: 0,
            heap_address: 0,
            heap_size: 0,
            open_files: ProcessDescriptorTable::init(),
            //screen: VirtualScreenID::null(),
        }
    }

    /// Creates a new process and set it as a child of `self`.
    /// `self` inherits a new child.
    /// `spawn` returns the PID of the child that is newly created.
    pub fn spawn(self, priority: Priority) -> ID {
        // -> &Mutex<Self> {
        let child = Process::create_new(self.pid, priority, self.owner);
        unsafe {
            CHILDREN.entry(self.pid).and_modify(|set| {
                set.insert(child.pid);
            });
            child.pid
        }
    }

    #[allow(clippy::empty_loop)]
    /// # Safety
    /// TODO
    pub unsafe fn launch() {
        fn f() {
            loop {}
        } // /!\
        launch_asm(f, 0);
    }
}

// Keeps track of the children of the processes, in order to keep the Process struct on the stack
static mut CHILDREN: BTreeMap<ID, BTreeSet<ID>> = BTreeMap::new();

/// A process's priority, used by the scheduler
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(usize);

/// All different states a process can reach
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Runnable,
    Running,
    Zombie(usize),
    SleepInterruptible,
    SleepUninterruptible,
    Stopped,
    SlotAvailable,
}

/// A process's ID.
///
/// Its uniqueness throughout the system is ensured by the atomic `fetch_add` operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ID(pub u64);

impl ID {
    /// Returns a fresh ID. It uses an atomic operation to make sure no two processes can have the same id.
    ///
    /// For now, a process's id isn't freed when it exits.
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        for _i in 0..PROCESS_MAX_NUMBER {
            let new = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            unsafe {
                if ID_TABLE[(new % PROCESS_MAX_NUMBER) as usize].state == State::SlotAvailable {
                    return ID(new % PROCESS_MAX_NUMBER);
                }
            }
        }
        panic!("no slot available")
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    /// Forges an `ID`, must *not* be used other than to build the first one.
    pub fn forge(index: u64) -> Self {
        Self(index)
    }
}
impl Default for ID {
    fn default() -> Self {
        Self::new()
    }
}

/// Main array of all processes
pub static mut ID_TABLE: [Process; PROCESS_MAX_NUMBER as usize] =
    [Process::missing(); PROCESS_MAX_NUMBER as usize];

pub fn spawn_first_process() {
    let mut proc = Process::create_new(ID::forge(0), Priority(0), 0);
    let cr3 = x86_64::registers::control::Cr3::read();
    proc.cr3 = cr3.0.start_address();
    proc.cr3f = cr3.1;
    /*if let Some(mainscreen) = unsafe { &mut mainscreen::MAIN_SCREEN } {
        proc.screen = mainscreen.new_screen(0, 0, 0, 0, VirtualScreenLayer::new(0));
    } else {
        errorln!("could not find mainscreen in first process");
    }*/
    let screen_file_name = "/hard/kbd";
    proc.open_files
        .create_file_table(Path::from(&screen_file_name), OpenFlags::ORDO as usize);
    let screen_file_name = "/hard/screen";
    proc.open_files
        .create_file_table(Path::from(&screen_file_name), OpenFlags::OWRO as usize);
    let shell_file_name = "/hard/host";
    proc.open_files
        .create_file_table(Path::from(&shell_file_name), OpenFlags::OWRO as usize);
    unsafe {
        ID_TABLE[0] = proc;
    }
}

pub static mut CURRENT_PROCESS: usize = 0;

/// # Safety
/// Depends of the usage of the data !
/// From the number of cycles executed, returns the current process
/// data structure (as mutable) and the next process to run one's (non mut)
/// Beware of not doing anything on this data !
pub unsafe fn gives_switch(_counter: u64) -> (&'static Process, &'static mut Process) {
    let old_pid = CURRENT_PROCESS;
    let new_pid = next_pid_to_run().0 as usize;
    CURRENT_PROCESS = new_pid;
    (&ID_TABLE[new_pid], &mut ID_TABLE[old_pid])
}

/// # Safety
/// Depends of the usage of the data !
/// From the number of cycles executed and return code, returns a new process
pub unsafe fn process_died(_counter: u64, return_code: u64) -> &'static Process {
    let old_pid = CURRENT_PROCESS;
    // Change parentality
    ID_TABLE[old_pid].state = State::Zombie(return_code as usize);
    for process in ID_TABLE.iter_mut() {
        if process.ppid == ID_TABLE[old_pid].pid {
            process.ppid = ID_TABLE[old_pid].ppid;
        }
    }

    let new_pid = next_pid_to_run().0 as usize;
    CURRENT_PROCESS = new_pid;

    &ID_TABLE[new_pid]
}

pub fn listen(id: usize) -> (usize, usize) {
    unsafe {
        let ppid = ID::forge(CURRENT_PROCESS as u64);
        if id == 0 {
            for (pid, process) in ID_TABLE.iter_mut().enumerate() {
                if process.ppid == ppid {
                    if let State::Zombie(return_value) = process.state {
                        process.state = State::SlotAvailable;
                        process.open_files.close();
                        if let Some(frame_allocator) = &mut memory::FRAME_ALLOCATOR {
                            frame_allocator.deallocate_level_4_page(
                                process.cr3,
                                PageTableFlags::USER_ACCESSIBLE,
                                true,
                            );
                            frame_allocator.deallocate_4k_frame(process.cr3);
                        }
                        return (pid, return_value);
                    }
                }
            }
        } else {
            let mut process = ID_TABLE[id];
            if process.ppid == ppid {
                if let State::Zombie(return_value) = process.state {
                    process.state = State::SlotAvailable;
                    process.open_files.close();
                    if let Some(frame_allocator) = &mut memory::FRAME_ALLOCATOR {
                        frame_allocator.deallocate_level_4_page(
                            process.cr3,
                            PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
                            true,
                        );
                        frame_allocator.deallocate_4k_frame(process.cr3);
                    }
                    return (id, return_value);
                }
            }
        }
        (0, 0)
    }
}

/// Returns the current process data structure as read only
/// # Safety
/// TODO
pub fn get_current() -> &'static Process {
    unsafe { &ID_TABLE[CURRENT_PROCESS] }
}

/// # Safety
/// Depends on the usage. May cause aliasing
/// Returns the current process data structure as mutable
pub unsafe fn get_current_as_mut() -> &'static mut Process {
    &mut ID_TABLE[CURRENT_PROCESS]
}

/// TODO safeguard the index
/// # Safety
/// TODO
pub unsafe fn get_process(pid: usize) -> &'static Process {
    &ID_TABLE[pid]
}

/// # Safety
/// Depending on the current process situation. Use knowingly
/// Function to duplicate the current process into two childs
/// For more info on the usage, see the code of the fork syscall
/// Returns : child process pid
pub unsafe fn fork() -> ID {
    let mut son = Process::create_new(
        ID_TABLE[CURRENT_PROCESS].pid,
        ID_TABLE[CURRENT_PROCESS].priority,
        ID_TABLE[CURRENT_PROCESS].owner,
    );
    if let Some(frame_allocator) = &mut memory::FRAME_ALLOCATOR {
        match frame_allocator.copy_table_entries(ID_TABLE[CURRENT_PROCESS].cr3) {
            Ok(phys) => son.cr3 = phys,
            Err(_) => panic!("TODO"),
        }
        son.cr3f = ID_TABLE[CURRENT_PROCESS].cr3f;
    } else {
        panic!("un initialized frame allocator");
    }
    let pid = son.pid;
    son.state = State::Runnable;
    son.rsp = ID_TABLE[CURRENT_PROCESS].rsp;
    son.stack_base = ID_TABLE[CURRENT_PROCESS].stack_base;
    son.open_files.copy(ID_TABLE[CURRENT_PROCESS].open_files);
    ID_TABLE[pid.0 as usize] = son;
    WAITING_QUEUES[son.priority.0]
        .push(pid)
        .expect("Could not push son process into the queue");
    // TODO
    pid
}

pub fn dup2(fd_target: usize, fd_from: usize) -> usize {
    unsafe {
        ID_TABLE[CURRENT_PROCESS]
            .open_files
            .dup(FileDescriptor::new(fd_target), FileDescriptor::new(fd_from))
    }
}

/// # Safety
/// It is irreversible, you just can't improve the priority of a process
/// This will set the priority of the current process to
/// the given value. It can be only decreasing
/// Returns : usize::MAX or the new priority if succeeds
pub unsafe fn set_priority(prio: usize) -> usize {
    // TODO : change attribution in WAITING_QUEUES? Or do we wait till the next execution? Is the overhead worth it?
    if prio > MAX_PRIO {
        return usize::MAX;
    }
    if ID_TABLE[CURRENT_PROCESS].priority.0 <= prio {
        ID_TABLE[CURRENT_PROCESS].priority.0 = prio;
        prio
    } else {
        usize::MAX
    }
}

/// # Safety
/// Need to add more security to prevent killing random processes
pub unsafe fn kill(target: usize) -> usize {
    let mut target_process = ID_TABLE[target];
    if target_process.priority < ID_TABLE[CURRENT_PROCESS].priority {
        1
    } else {
        target_process.state = State::Zombie(1);
        0
    }
}

pub unsafe fn write_to_stdout(message: String) {
    if let Ok(res) = &ID_TABLE[CURRENT_PROCESS]
        .open_files
        .get_file_table(FileDescriptor::new(1))
    {
        filesystem::write_file(res, message.as_bytes().to_vec());
    }
}

fn next_priority_to_run() -> usize {
    let mut ticket = random::random_u8();
    // debug!("Ticket = {:#b}", ticket);
    // Look for the most significant non null bit in the ticket
    let mut idx = 7;
    while idx > 0 && ticket != 0 {
        ticket <<= 1;
        idx -= 1;
    }
    // debug!("final idx: {}",idx);
    idx
}

const MAX_PRIO: usize = 8;
static mut WAITING_QUEUES: [Queue<ID>; MAX_PRIO] = [
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
];

static mut IDLE: BTreeSet<ID> = BTreeSet::new();

/// Adds the given pid to the correct priority queue
/// It tries to push it in the designated priority, but if it is full,
/// it will promote the process until it finds room
/// or there is no place left in priority 0, in which case it crashes.
/// # Safety
/// Requires WAITING_QUEUES to be sane
fn enqueue_prio(pid: ID, prio: usize) {
    unsafe {
        let mut effective_prio = prio;
        while WAITING_QUEUES[effective_prio].is_full() && effective_prio > 0 {
            effective_prio -= 1
        }
        if effective_prio == 0 && WAITING_QUEUES[effective_prio].is_full() {
            panic!("Too many processes want to run at the same priority!")
        }
        WAITING_QUEUES[effective_prio]
            .push(pid)
            .expect("Scheduler massive fail");
    }
}

/// Adds a process that might be runnable later into the IDLE collection.
/// We guarantee that each element is present at most once.
fn add_idle(pid: ID) {
    unsafe {
        IDLE.insert(pid);
    }
}

/// # Safety
/// Needs sane `WAITING_QUEUES`. Should be safe to use.
unsafe fn next_pid_to_run() -> ID {
    let mut prio = next_priority_to_run();
    // Find the lowest priority at least as urgent as the one indated by the ticket that is not empty
    while prio < MAX_PRIO && WAITING_QUEUES[prio].is_empty() {
        prio -= 1; // need to check priority
    }
    if prio >= MAX_PRIO {
        prio = 0;
        while prio < MAX_PRIO && WAITING_QUEUES[prio].is_empty() {
            prio += 1;
        }
        if prio == MAX_PRIO {
            return ID(CURRENT_PROCESS as u64);
        }
    }
    let old_pid = ID(CURRENT_PROCESS as u64);
    let old_priority = ID_TABLE[old_pid.0 as usize].priority.0;
    let old_state = ID_TABLE[old_pid.0 as usize].state;
    // TODO : book-keeping to potentially empty IDLE

    let new_pid = WAITING_QUEUES[prio].pop().expect("Scheduler massive fail");
    match old_state {
        State::Runnable => enqueue_prio(old_pid, old_priority),
        State::SlotAvailable | State::Zombie(_) => (),
        State::Running => {
            ID_TABLE[old_pid.0 as usize].state = State::Runnable;
            enqueue_prio(old_pid, old_priority);
        }
        State::SleepInterruptible | State::SleepUninterruptible | State::Stopped => {
            add_idle(old_pid)
        }
    }
    match ID_TABLE[new_pid.as_usize()].state {
        State::Runnable | State::Running => new_pid,
        State::SlotAvailable | State::Zombie(_) => next_pid_to_run(),
        State::SleepInterruptible | State::SleepUninterruptible | State::Stopped => {
            add_idle(new_pid);
            next_pid_to_run()
        }
    }
}
