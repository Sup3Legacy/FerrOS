use super::PROCESS_MAX_NUMBER;
use crate::data_storage::registers::Registers;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;

extern "C" {
    fn launch_asm(first_process: fn(), initial_rsp: u64);
}

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
    id: ID,
    pid: ID,
    priority: Priority,
    quantum: u64,
    cr3: usize,
    rsp: u64,
    rip: u64,
    registers: Registers,
    state: State,
    //  children: Vec<Mutex<Process>>, // Maybe better to just store ID's
    children: Vec<ID>,
    value: usize,
    owner: u64,
}

impl Process {
    pub fn create_new(parent: ID, priority: Priority, owner: u64) -> Self {
        Self {
            id: ID::new(),
            pid: parent,
            priority,
            quantum: 0_u64,
            cr3: 42, // /!\
            rsp: 0,  // /!\
            rip: 0,  // /!\
            registers: Registers::new(),
            state: State::Runnable,
            children: Vec::new(),
            value: 0,
            owner,
        }
    }

    pub fn missing() -> Self {
        Self {
            id: ID(0),
            pid: ID(0),
            priority: Priority(0),
            quantum: 0_u64,
            cr3: 0,
            rsp: 0,
            rip: 0,
            registers: Registers::new(),
            state: State::SlotAvailable,
            children: Vec::new(),
            value: 0,
            owner: 0,
        }
    }

    pub fn spawn(&mut self, priority: Priority) -> &ID {
        // -> &Mutex<Self> {
        let child = Process::create_new(self.id, priority, self.owner);
        self.children.push(child.id); //Mutex::new(child));
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
    static ref CURRENT_PROCESS: Process = Process::missing();
}
