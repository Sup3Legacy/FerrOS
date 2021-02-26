

pub mod process;

/// Maximum number of co-existing processes.
///
/// /!\ For now, it is the maximum number of processes that can be spawned

const PROCESS_MAX_NUMBER: u64 = 32;

/// Number of consecutive time slices a process can use
/// before another one is executed
const QUANTUM: u64 = 10;
