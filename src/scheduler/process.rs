use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use super::PROCESS_MAX_NUMBER;

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
#[derive(Clone, Debug)]
pub struct Process {
    id: ID,
    pid: ID,
    priority: Priority,
    quantum : u64,
    cr3: usize,
    state: State,
    children : Vec<Process>,
    value : usize
}

/// A process's priority, used by the scheduler
#[derive(Clone, Copy, Debug)]
pub struct Priority(usize);

/// All different states a process can reach
#[derive(Clone, Copy, Debug)]
pub enum State {
    Runnable,
    Running,
    Zombie,
    SleepInterruptible,
    SleepUninterruptible,
    Stopped,
}

/// A process's ID.
///
/// It's uniqueness throughout the system is ensured by the atomic `fetch_add` operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ID (u64);

impl ID {
    /// Returns a fresh ID. It uses an atomic operation to make sure no two processes can have the same id.
    ///
    /// For now, a process's id isn't freed when it exits.
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let new = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        if new >= PROCESS_MAX_NUMBER {
            panic!("Reached maximum number of processes!");
        }
        Self(new)
    }
}
