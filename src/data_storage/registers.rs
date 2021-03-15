#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Registers {
    rbx: u64,
    rcx: u64,
    rbp: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    r9: u64,
    r8: u64,
    r10: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rax: u64,
}

impl Registers {
    pub fn new() -> Self {
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