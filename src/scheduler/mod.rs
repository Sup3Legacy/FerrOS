pub mod process;

/// Maximum number of co-existing processes.
///
/// /!\ For now, it is the maximum number of processes that can be spawned

pub const PROCESS_MAX_NUMBER: u64 = 32;

/// Number of consecutive time slices a process can use
/// before another one is executed
pub const QUANTUM: u64 = 20;
