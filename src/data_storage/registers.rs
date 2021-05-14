//! Data-structures holdign registers states

/// Holds the value of a complete set of the `x86-64` GP registers.
/// It is used to read from/write into a process' registers during a syscall.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Registers {
    pub r9: u64,
    pub r8: u64,
    pub r10: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rbp: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

impl Registers {
    /// Returns a zero-initialized set of registers
    pub const fn new() -> Self {
        Registers {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// interface structure for syscalls
/// * r9,  syscall argument 5
/// * r8,  syscall argument 4
/// * r10, syscall argument 3
/// * rdx, syscall argument 2
/// * rsi, syscall argument 1
/// * rdi, syscall argument 0
/// * rax, syscall number
pub struct RegistersMini {
    pub r9: u64,  // syscall argument 5
    pub r8: u64,  // syscall argument 4
    pub r10: u64, // syscall argument 3
    pub rdx: u64, // syscall argument 2
    pub rsi: u64, // syscall argument 1
    pub rdi: u64, // syscall argument 0
    pub rax: u64, // syscall number
}
