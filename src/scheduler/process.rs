use super::PROCESS_MAX_NUMBER;
use crate::data_storage::registers::Registers;
use crate::interrupts::idt::InterruptStackFrameValue;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use x86_64::registers::control::Cr3Flags;
use x86_64::{PhysAddr, VirtAddr};

extern "C" {
    fn launch_asm(first_process: fn(), initial_rsp: u64);

    /// Old function definition
    pub fn _leave_context(rsp: u64);
}

#[naked]
pub unsafe extern "C" fn leave_context(_rsp: u64) {
    asm!(
        "mov rsp, rdi",
        "pop rbx",
        "pop rcx",
        "pop rbp",
        "pop r11",
        "pop r12",
        "pop r13",
        "pop r14",
        "pop r15",
        "pop r9",
        "pop r8",
        "pop r10",
        "pop rdx",
        "pop rsi",
        "pop rdi",
        "pop rax",
        //"sti",
        "iretq", options(noreturn,),
    )
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
///
/// # Fields
/// * `id` - the id of the process (unique)
/// * `pid` - its parent's (i.e. the process that spawned it) id
/// * `priority` - the priority, used by the scheduler
/// * `quantum` - tyhe number of consecutive quanta the process has already been running for
/// * `cr3` - pointer to its 1st order VM table. TO DO : replace it with a PhysFrame or PhysAddr
/// * `state` - state of the process
/// * `children` - vec containing the processes it spawned.
/// * `value` - return value
/// * `owner` - owner ID of the process (can be root or user) usefull for syscalls
#[derive(Debug)]
#[repr(C)]
pub struct Process {
    pid: ID,
    ppid: ID,
    priority: Priority,
    quantum: u64,
    pub cr3: PhysAddr,
    pub cr3f: Cr3Flags,
    pub rsp: VirtAddr, // every registers are saved on the stack
    //pub stack_frame: InterruptStackFrameValue,
    //pub registers: Registers,
    state: State,
    //  children: Vec<Mutex<Process>>, // Maybe better to just store ID's
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
            rsp: VirtAddr::zero(),
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
            rsp: VirtAddr::zero(),
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

    pub unsafe fn launch() {
        fn f() {
            loop {}
        } // /!\
        launch_asm(f, 0);
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
}
pub static mut CURRENT_PROCESS: Process = Process::missing();

pub unsafe fn gives_switch(counter: u64) -> (&'static Process, &'static mut Process) {
    return (&CURRENT_PROCESS, &mut CURRENT_PROCESS);
}
