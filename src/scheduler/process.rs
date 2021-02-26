use super::PROCESS_MAX_NUMBER;
//use crate::data_storage::registers::Registers;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
//use spin::Mutex;

extern "C" {
    fn launch_asm(first_process: fn(), initial_rsp: u64);
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
/// * `rip` - current value of the instruction pointer
/// * `state` - state of the process (e.g. Zombie, Runnable...)
/// * `children` - vec containing the processes it spawned.
/// * `value` - return value -> move this to the extended Zombie state
/// * `owner` - owner ID of the process (can be root or user) useful for syscalls
///
/// Note: registers and flags are stored on the stack when doing context switching
#[derive(Debug)]
#[repr(C)]
pub struct Process {
    rip: usize,
    cr3: usize,
    pid: ID,
    ppid: ID,
    priority: Priority,
    quantum: u64,
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
            quantum: 0 as u64,
            cr3: 42, // /!\
            rip: 0,  // /!\
            state: State::Runnable,
            children: Vec::new(),
            value: 0,
            owner,
        }
    }

    pub fn missing() -> Self {
        Self {
            pid: ID(0),
            ppid: ID(0),
            priority: Priority(0),
            quantum: 0 as u64,
            cr3: 0,
            rip: 0,
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
        for i in 0..PROCESS_MAX_NUMBER {
            let new = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            match ID_TABLE[(new % PROCESS_MAX_NUMBER) as usize].state {
                State::SlotAvailable => return ID(new),
                _ => (),
            }
        }
        panic!("no slot available")
    }
}

lazy_static! {
    static ref ID_TABLE: [Process; PROCESS_MAX_NUMBER as usize] = [
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
        Process::missing()
    ];
    static ref CURRENT_PROCESS: Process = Process::missing();
}
