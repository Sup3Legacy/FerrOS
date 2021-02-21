
use crate::{println, print};

#[repr(C)]
pub struct Registers {
    r9 : u64, // syscall argument 5
    r8 : u64, // syscall argument 4
    r10: u64, // syscall argument 3
    rdx: u64, // syscall argument 2
    rsi: u64, // syscall argument 1
    rdi: u64, // syscall argument 0
    rax: u64, // syscall number
}

pub type SysCallFunc = extern "C" fn();

const SYSCALL_NUMBER: u64 = 5;

const SYSCALL_TABLE : [extern "C" fn(Registers); SYSCALL_NUMBER as usize] = [
    syscall_0_read,
    syscall_1_write,
    syscall_2_open,
    syscall_3_close,

    syscall_not_implemented
];


/// read. arg0 : unsigned int fd, arg1 : char *buf, size_t count
extern "C" fn syscall_0_read(_args: Registers) {
    panic!("not implemented")
}

/// write. arg0 : unsigned int fd, arg1 : const char *buf, size_t count
extern "C" fn syscall_1_write(_args: Registers) {
    println!("congrats you just called the good syscall!")
}

/// open file. arg0 : const char *filename, arg1 : int flags, arg2 : umode_t mode
extern "C" fn syscall_2_open(_args: Registers) {
    panic!("not implemented")
}

/// close file. arg0 : unsigned int fd
extern "C" fn syscall_3_close(_args: Registers) {
    panic!("not implemented")
}

extern "C" fn syscall_not_implemented(_args: Registers) {
    panic!("not implemented")
}

extern "C" fn syscall_dispatch(args: Registers) {
    if args.rax >= SYSCALL_NUMBER {
        panic!("no such syscall")
    } else {
        SYSCALL_TABLE[args.rax as usize](args)
    }
}


#[naked]
pub extern "C" fn naked_syscall_dispatch() {
    unsafe {
        asm!(
            "push rcx",
            "push r11",
            "push rax",
            "push rdi",
            "push rsi",
            "push rdx",
            "push r10",
            "push r8",
            "push r9",
            "mov rdi, rsp",
            "call {0}",
            "pop r9",
            "pop r8",
            "pop r10",
            "pop rdx",
            "pop rsi",
            "pop rdi",
            "pop rax",
            "pop r11",
            "pop rcx",
            "iretq",
              sym syscall_dispatch
            );
    }
}
