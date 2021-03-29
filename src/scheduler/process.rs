use super::PROCESS_MAX_NUMBER;

use alloc::vec::Vec;
use core::{
    convert::TryInto,
    sync::atomic::{AtomicU64, Ordering},
};
//use lazy_static::lazy_static;
use core::{mem::transmute, todo};
use x86_64::structures::paging::PageTableFlags;
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::Page,
};
use x86_64::{PhysAddr, VirtAddr};

use xmas_elf::{sections::ShType, ElfFile};

use crate::data_storage::{queue::Queue, random};
use crate::errorln;
use crate::hardware;
use crate::memory;
use crate::println;

#[allow(improper_ctypes)]
extern "C" {
    fn launch_asm(first_process: fn(), initial_rsp: u64);

    /// Old function definition
    pub fn _leave_context(rsp: u64);
}

#[naked]
/// # Safety
/// TODO
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
        "add rsp, 32",
        "vmovaps ymm0, [rsp]",
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
        "add rsp, 32",
        "vmovaps ymm0, [rsp]",
        //"sti",
        "iretq",
        options(noreturn,),
    )
}

#[naked]
/// # Safety
/// TODO
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
        "push 0",
        "push rax",  // stack segment
        "push rdi",  // stack pointer
        "push 518",  // cpu flags
        "push 0x08", // code segment
        "push rsi",  // instruction pointer
        "iretq",
        options(noreturn,),
    )
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

pub unsafe fn disassemble_and_launch(
    code: &[u8],
    frame_allocator: &mut memory::BootInfoAllocator,
    number_of_block: u64,
    stack_size: u64,
) -> ! {
    let PROG_OFFSET = 0x8048000000;
    let elf = ElfFile::new(code).unwrap();
    let prog_entry = match elf.header.pt2 {
        xmas_elf::header::HeaderPt2::Header64(a) => a.entry_point,
        _ => panic!("Expected a 64-bit ELF!"),
    };
    if let Ok(level_4_table_addr) = frame_allocator.allocate_level_4_frame() {
        ID_TABLE[0].state = State::Runnable;
        // TODO maybe consider changing this
        let addr_stack: u64 = 0x63fffffffff8;
        // Allocate frames for each section
        for section in elf.section_iter() {
            let address = section.address();
            let offset = section.offset();
            let size = section.size();
            println!(
                "Block, address : 0x{:x?}, offset : 0x{:x?}, size : 0x{:x?}, type : {:?}",
                address,
                offset,
                size,
                section.type_()
            );

            if (address - offset) == 0 {
                continue;
            }

            let _data = section.raw_data(&elf);
            let data = transmute::<&[u8], &[u64]>(_data);
            let mut prev_offset = Vec::new();
            for _ in 0..(offset / 8) {
                prev_offset.push(0_u64);
            }
            let mut last_offset = Vec::new();
            for _ in 0..(512 - ((size + offset / 8) % 512)) {
                last_offset.push(0_u64);
            }
            let corrected_data = [&prev_offset[..], data, &last_offset[..]].concat();
            assert_eq!(corrected_data.len() % 512, 0);
            let sliced: &[[u64; 512]] = corrected_data.as_chunks_unchecked();
            let num_blocks = sliced.len();
            println!(
                "Total len of 0x{:x?}, {:?} blocks",
                num_blocks * 512,
                num_blocks
            );
            let flags = match section.get_type().unwrap() {
                ShType::ProgBits => PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
                ShType::SymTab => PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
                ShType::StrTab => PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
                _ => {
                    PageTableFlags::USER_ACCESSIBLE
                        | PageTableFlags::PRESENT
                        | PageTableFlags::NO_EXECUTE
                }
            };
            for i in 0..num_blocks {
                // Allocate a frame for each page needed.
                match frame_allocator.add_entry_to_table(
                    level_4_table_addr,
                    VirtAddr::new(address + (i as u64) * 4096 + PROG_OFFSET),
                    flags,
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
                VirtAddr::new(address + PROG_OFFSET),
                _data,
            ) {
                Ok(()) => (),
                Err(a) => errorln!("{:?} at section : {:?}", a, section),
            };
        }
        // Allocate frames for the stack
        for i in 0..stack_size {
            match frame_allocator.add_entry_to_table(
                level_4_table_addr,
                VirtAddr::new(addr_stack - i * 0x1000),
                PageTableFlags::USER_ACCESSIBLE
                    | PageTableFlags::PRESENT
                    | PageTableFlags::NO_EXECUTE
                    | PageTableFlags::WRITABLE,
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
        println!("good luck user ;) {} {}", addr_stack, prog_entry);
        println!("target : {:x}", towards_user as usize);
        towards_user(addr_stack, prog_entry + PROG_OFFSET); // good luck user ;)
        hardware::power::shutdown();
    }
    loop {}
}

/*
    mov rsp, rdi
    pop rbx
    pop rcx
    pop rbp
    pop r11
    pop r12
    pop r13
    pop r14
    pop r15
    pop r9
    pop r8
    pop r10
    pop rdx
    pop rsi
    pop rdi
    pop rax
    sti
    iretq
*/

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
/// * `rip` - current value of the instruction pointer
/// * `state` - state of the process (e.g. Zombie, Runnable...)
/// * `children` - vec containing the processes it spawned.
/// * `value` - return value
/// * `owner` - owner ID of the process (can be root or user) usefull for syscalls
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Process {
    pid: ID,
    ppid: ID,
    priority: Priority,
    quantum: u64,
    pub cr3: PhysAddr,
    pub cr3f: Cr3Flags,
    pub rsp: u64, // every registers are saved on the stack
    //pub stack_frame: InterruptStackFrameValue,
    //pub registers: Registers,
    state: State,
    children: Vec<ID>,
    value: usize,
    owner: u64,
}

impl Process {
    pub fn create_new(parent: ID, priority: Priority, owner: u64) -> Self {
        Self {
            pid: ID::new(),
            ppid: parent,
            priority,
            quantum: 0_u64,
            cr3: PhysAddr::zero(),
            cr3f: Cr3Flags::empty(),
            rsp: 0,
            //stack_frame: InterruptStackFrameValue::empty(),
            //registers: Registers::new(),
            state: State::Runnable,
            children: Vec::new(),
            value: 0,
            owner,
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
            //stack_frame: InterruptStackFrameValue::empty(),
            //registers: Registers::new(),
            state: State::SlotAvailable,
            children: Vec::new(),
            value: 0,
            owner: 0,
        }
    }

    pub fn spawn(&mut self, priority: Priority) -> &ID {
        // -> &Mutex<Self> {
        let child = Process::create_new(self.pid, priority, self.owner);
        self.children.push(child.pid); //Mutex::new(child));
        &(self.children[self.children.len() - 1])
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

    #[naked]
    unsafe fn save_state() {
        asm!(
            "push rax", // save rax
            "mov rax, [rsp+24]", // rax <- rip
            "mov [{0}], rax", // store rip
            "mov rax, cr3", // rax <- cr3
            "mov [{0}+8], rax", // store cr3
            "pop rax",

            "sub rsp, 8", // remove rip from the stack : we want to add it manually after

            // continue to save the other registers
            "push rbx",
            "push rcx",
            "push rdx",
            "push rbp",
            "push rsp",
            "push rsi",
            "push rdi",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            "push r12",
            "push r13",
            "push r14",
            "push r15",

            // save the flags
            "pushfq",
            sym CURRENT_PROCESS,
            options(noreturn)
        );
    }

    #[naked]
    unsafe fn load_state() {
        asm!(
            // restore flags
            "popfq",

            //restore registers
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rdi",
            "pop rsi",
            "pop rsp",
            "pop rbp",
            "pop rdx",
            "pop rcx",
            "pop rbx",

            //restore cr3 (paging)
            "mov rax, [{0} + 8]", // cr3
            "mov cr3, rax",

            // restore rip
            "mov rax, [{0}]", // rip
            "mov [rsp+8], rax",

            // restore rax
            "pop rax",

            "ret",
            sym CURRENT_PROCESS,
            options(noreturn)
        );
    }
}

/// A process's priority, used by the scheduler
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(usize);

/// All different states a process can reach
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Runnable,
    Running,
    Zombie,
    SleepInterruptible,
    SleepUninterruptible,
    Stopped,
    SlotAvailable,
}

/// A process's ID.
///
/// It's uniqueness throughout the system is ensured by the atomic `fetch_add` operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ID(u64);

impl ID {
    /// Returns a fresh ID. It uses an atomic operation to make sure no two processes can have the same id.
    ///
    /// For now, a process's id isn't freed when it exits.
    pub fn new_old() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let new = NEXT_ID.fetch_add(1, Ordering::Relaxed); // Maybe better to reallow previous numbers
        if new >= PROCESS_MAX_NUMBER {
            panic!("Reached maximum number of processes!");
        }
        Self(new)
    }

    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        for _i in 0..PROCESS_MAX_NUMBER {
            let new = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            unsafe {
                if ID_TABLE[(new % PROCESS_MAX_NUMBER) as usize].state == State::SlotAvailable {
                    return ID(new);
                }
            }
        }
        panic!("no slot available")
    }
}
impl Default for ID {
    fn default() -> Self {
        Self::new()
    }
}

static mut ID_TABLE: [Process; PROCESS_MAX_NUMBER as usize] = [
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
    Process::missing(),
];

pub static mut CURRENT_PROCESS: usize = 0;

/// # Safety
/// Depends of the usage of the data !
/// From the number of cycles executed, returns the current process
/// data structure (as mutable) and the next process to run one's (non mut)
/// Beware of not doing any think on this data !
pub unsafe fn gives_switch(_counter: u64) -> (&'static Process, &'static mut Process) {
    for (new, new_id) in ID_TABLE
        .iter()
        .enumerate()
        .take(PROCESS_MAX_NUMBER as usize)
    {
        if new != CURRENT_PROCESS && new_id.state == State::Runnable {
            let old = CURRENT_PROCESS;
            CURRENT_PROCESS = new;
            println!("{} <-> {}", old, new);
            return (&new_id, &mut ID_TABLE[old]);
        }
    }
    (&ID_TABLE[CURRENT_PROCESS], &mut ID_TABLE[CURRENT_PROCESS])
}

/// Returns the current process data structure as read only
pub fn get_current() -> &'static Process {
    unsafe { &ID_TABLE[CURRENT_PROCESS] }
}

/// # Safety depends on the usage. May cause aliasing
/// Returns the current process data structure as mutable
pub unsafe fn get_current_as_mut() -> &'static mut Process {
    &mut ID_TABLE[CURRENT_PROCESS]
}

/// # Safety
/// Depending on the current process situation. Use knowingly
/// Function to duplicate the current process into two childs
/// For more info on the usage, see the code of the fork syscall
/// Returns : child process pid
pub unsafe fn fork() -> u64 {
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
    let pid = son.pid.0;
    son.state = State::Runnable;
    son.rsp = ID_TABLE[CURRENT_PROCESS].rsp;
    ID_TABLE[pid as usize] = son;
    println!("new process of id {}", pid);

    // TODO
    pid
}

/// # Safety
/// It is irreversible, you just can't improve the priority of a process
/// This will set the priority of the current process to
/// the given value. It can be only decreasing
/// Returns : usize::MAX or the new priority if succeeds
pub unsafe fn set_priority(prio: usize) -> usize {
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

fn next_priority_to_run() -> usize {
    let mut ticket = random::random_u8();
    // Look for the most significant non null bit in the ticket
    let mut idx = 7;
    while idx > 0 || ticket != 0 {
        ticket <<= 1;
        idx += 1;
    }
    MAX_PRIO - idx
}

const MAX_PRIO: usize = 8;
static mut WAITING_QUEUES: [Queue<usize>; MAX_PRIO] = [
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
];

#[allow(dead_code)]
/// # Safety
/// Needs sane `WAITING_QUEUES`. Should be safe to use.
unsafe fn next_process_to_run() -> usize {
    let mut prio = next_priority_to_run();
    // Find the lowest priority at least as urgent as the one indated by the ticket that is not empty
    while WAITING_QUEUES[prio].is_empty() {
        prio -= 1; // need to check priority
    }
    let old_pid = CURRENT_PROCESS;
    let new_pid = WAITING_QUEUES[prio].pop().expect("Scheduler massive fail");
    let mut old_priority = ID_TABLE[old_pid].priority.0;
    while WAITING_QUEUES[old_pid].is_full() && old_priority > 0 {
        old_priority -= 1
    }
    if old_priority == 0 && WAITING_QUEUES[old_priority].is_full() {
        panic!("Too many processes want to run at the same priority!")
    }
    WAITING_QUEUES[old_priority]
        .push(old_pid)
        .expect("Scheduler massive fail");
    new_pid
}
