
use crate::{println, print};

#[repr(C)]
pub struct Registers {
    rax: u64, // syscall number
    rdi: u64, // syscall argument 0
    rsi: u64, // syscall argument 1
    rdx: u64, // syscall argument 2
    r10: u64, // syscall argument 3
    r8 : u64, // syscall argument 4
    r9 : u64, // syscall argument 5
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
extern "C" fn syscall_0_read(args: Registers) {
    panic!("not implemented")
}

/// write. arg0 : unsigned int fd, arg1 : const char *buf, size_t count
extern "C" fn syscall_1_write(args: Registers) {
    println!("rax:{} rdi:{} rsi:{} rdx:{} r10:{} r9:{}", args.rax, args.rdi, args.rsi, args.rdx, args.r10, args.r9)
}

/// open file. arg0 : const char *filename, arg1 : int flags, arg2 : umode_t mode
extern "C" fn syscall_2_open(args: Registers) {
    panic!("not implemented")
}

/// close file. arg0 : unsigned int fd
extern "C" fn syscall_3_close(args: Registers) {
    panic!("not implemented")
}

extern "C" fn syscall_not_implemented(args: Registers) {
    panic!("not implemented")
}

extern "C" fn syscall_dispatch(args: Registers) {
    println!("rax:{} rdi:{} rsi:{} rdx:{} r10:{} r9:{}", args.rax, args.rdi, args.rsi, args.rdx, args.r10, args.r9);
    if args.rax >= SYSCALL_NUMBER {
        panic!("no such syscall")
    } else {
        SYSCALL_TABLE[args.rax as usize](args)
    }
}

#[naked]
pub extern "C" fn naked_syscall_dispatch() -> ! {
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
        ::core::intrinsics::unreachable();
    }
}