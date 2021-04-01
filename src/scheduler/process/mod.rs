use super::PROCESS_MAX_NUMBER;

use core::sync::atomic::{AtomicU64, Ordering};
//use lazy_static::lazy_static;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::PageTableFlags;
use x86_64::{PhysAddr, VirtAddr};

use xmas_elf::{sections::ShType, ElfFile};

use crate::alloc::collections::{BTreeMap, BTreeSet};
use crate::alloc::vec::Vec;
use crate::data_storage::{queue::Queue, random};
use crate::{errorln, println, warningln, debug};
use crate::hardware;
use crate::memory;

pub mod elf;

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
) -> ! {
    const PROG_OFFSET: u64 = 0x8048000000;
    // TODO maybe consider changing this
    let addr_stack: u64 = 0x63fffffffff8;
    // We get the `ElfFile` from the raw slice
    let elf = ElfFile::new(code).unwrap();
    // We get the main entry point and mmake sure it is
    // a 64-bit ELF file
    let prog_entry = match elf.header.pt2 {
        xmas_elf::header::HeaderPt2::Header64(a) => a.entry_point,
        _ => panic!("Expected a 64-bit ELF!"),
    };
    // This allocates a new level-4 table
    if let Ok(level_4_table_addr) = frame_allocator.allocate_level_4_frame() {
        ID_TABLE[0].state = State::Runnable;
        // Loop over each section
        for section in elf.section_iter() {
            // Characteristics of the section
            let address = section.address();
            let offset = section.offset();
            let size = section.size();
            // Section debug
            /*
            println!(
                "Block, address : 0x{:x?}, offset : 0x{:x?}, size : 0x{:x?}, type : {:?}",
                address,
                offset,
                size,
                section.get_type()
            );
            */

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

            // TODO : change this to respect the conventions
            // For now, it is very probably wrong
            // for certain writable segment types
            let flags = match section.get_type().unwrap() {
                ShType::ProgBits => PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
                ShType::SymTab => PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
                ShType::StrTab => PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT,
                _ => {
                    PageTableFlags::USER_ACCESSIBLE
                        | PageTableFlags::PRESENT
                        | PageTableFlags::NO_EXECUTE
                        | PageTableFlags::WRITABLE
                }
            };
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
        println!("good luck user ;) {} {}", addr_stack, prog_entry);
        println!("target : {:x}", towards_user as usize);
        towards_user(addr_stack, prog_entry + PROG_OFFSET); // good luck user ;)
        hardware::power::shutdown();
    }
    loop {}
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
    state: State,
    owner: u64,
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
            state: State::Runnable,
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
            state: State::SlotAvailable,
            owner: 0,
        }
    }

    /// Creates a new process and set it as a child of `self`.
    /// `self` inherits a new child.
    /// `spawn` returns the PID of the child that is newly created.
    pub fn spawn(self, priority: Priority) -> ID {
        // -> &Mutex<Self> {
        let child = Process::create_new(self.pid, priority, self.owner);
        unsafe{
            CHILDREN.entry(self.pid)
                    .and_modify(|set| {set.insert(child.pid);});
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
static mut CHILDREN : BTreeMap<ID,BTreeSet<ID>> = BTreeMap::new();

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
                    return ID(new);
                }
            }
        }
        panic!("no slot available")
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

pub fn spawn_first_process() {
    let mut proc = Process::create_new(ID::forge(0), Priority(0), 0);
    let cr3 = x86_64::registers::control::Cr3::read();
    proc.cr3 = cr3.0.start_address();
    proc.cr3f = cr3.1;
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

/// Returns the current process data structure as read only
pub fn get_current() -> &'static Process {
    unsafe { &ID_TABLE[CURRENT_PROCESS] }
}

/// # Safety
/// Depends on the usage. May cause aliasing
/// Returns the current process data structure as mutable
pub unsafe fn get_current_as_mut() -> &'static mut Process {
    &mut ID_TABLE[CURRENT_PROCESS]
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
    ID_TABLE[pid.0 as usize] = son;
    println!("new process of id {:#?}", pid);
    WAITING_QUEUES[son.priority.0]
        .push(pid)
        .expect("Could not push son process into the queue");
    // TODO
    pid
}

/// # Safety
/// It is irreversible, you just can't improve the priority of a process
/// This will set the priority of the current process to
/// the given value. It can be only decreasing
/// Returns : usize::MAX or the new priority if succeeds
pub unsafe fn set_priority(prio: usize) -> usize {
    // TODO : change attribution in WAITING_QUEUES? Or do we wait till the next execution? Is the overhead worth it?
    if prio > MAX_PRIO {
        return usize::MAX
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

const MAX_PRIO:usize = 8;
static mut WAITING_QUEUES : [Queue<ID>; MAX_PRIO] = [
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    Queue::new(),
    ];

static mut IDLE : BTreeSet<ID> = BTreeSet::new();

/// Adds the given pid to the correct priority queue
/// It tries to push it in the designated priority, but if it is full,
/// it will promote the process until it finds room
/// or there is no place left in priority 0, in which case it crashes.
/// # Safety
/// Requires WAITING_QUEUES to be sane
fn enqueue_prio(pid: ID, prio: usize) {
    unsafe{
    let mut effective_prio = prio;
    while WAITING_QUEUES[effective_prio].is_full() && effective_prio > 0{
        effective_prio -= 1
    }
    if effective_prio == 0 && WAITING_QUEUES[effective_prio].is_full() {
        panic!("Too many processes want to run at the same priority!")
    }
    WAITING_QUEUES[effective_prio].push(pid).expect("Scheduler massive fail");
    }
}

/// Adds a process that might be runnable later into the IDLE collection.
/// We guarantee that each element is present at most once.
fn add_idle(pid: ID) {
    unsafe{ IDLE.insert(pid); }
}

 /// # Safety
 /// Needs sane `WAITING_QUEUES`. Should be safe to use.
 unsafe fn next_pid_to_run() -> ID {
    let mut prio = next_priority_to_run();
    // Find the lowest priority at least as urgent as the one indated by the ticket that is not empty
    while WAITING_QUEUES[prio].is_empty(){
        prio -= 1; // need to check priority
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
        },
        State::SleepInterruptible
            | State::SleepUninterruptible
            | State::Stopped => add_idle(old_pid),
        _ => panic!("{:#?} unsupported in scheduler!",old_state)
    }
    new_pid
 }
