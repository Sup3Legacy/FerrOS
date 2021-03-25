//! Part of the OS responsible for handling syscalls

use super::idt::InterruptStackFrame;
use crate::data_storage::registers::{Registers, RegistersMini};
use crate::{debug};

/// type of the syscall interface inside the kernel
pub type SyscallFunc = extern "C" fn();

/// total number of syscalls
const SYSCALL_NUMBER: u64 = 10;

/// table containing every syscall functions
const SYSCALL_TABLE: [extern "C" fn(RegistersMini, InterruptStackFrame); SYSCALL_NUMBER as usize] = [
    syscall_0_read,
    syscall_1_write,
    syscall_2_open,
    syscall_3_close,
    syscall_4_dup2,
    syscall_5_fork,
    syscall_6_exec,
    syscall_7_exit,
    syscall_8_wait,
    syscall_test,
];

/// highly dangerous function should use only when knowing what you are doing
#[naked]
unsafe extern "C" fn convert_register_to_full(_args: RegistersMini) -> Registers {
    asm!("mov rax, rdi", "ret");
    loop {}
}

/// read. arg0 : unsigned int fd, arg1 : char *buf, size_t count
extern "C" fn syscall_0_read(_args: RegistersMini, _isf: InterruptStackFrame) {
    panic!("read not implemented")
}

/// write. arg0 : unsigned int fd, arg1 : const char *buf, size_t count
extern "C" fn syscall_1_write(_args: RegistersMini, _isf: InterruptStackFrame) {
    debug!("write congrats you just called the good syscall!")
}

/// open file. arg0 : const char *filename, arg1 : int flags, arg2 : umode_t mode
extern "C" fn syscall_2_open(_args: RegistersMini, _isf: InterruptStackFrame) {
    panic!("open not implemented")
}

/// close file. arg0 : unsigned int fd
extern "C" fn syscall_3_close(_args: RegistersMini, _isf: InterruptStackFrame) {
    panic!("close not implemented")
}

extern "C" fn syscall_4_dup2(_args: RegistersMini, _isf: InterruptStackFrame) {
    panic!("dup2 not implemented");
}

extern "C" fn syscall_5_fork(_args: RegistersMini, _isf: InterruptStackFrame) {
    panic!("fork not implemented");
}

extern "C" fn syscall_6_exec(_args: RegistersMini, _isf: InterruptStackFrame) {
    panic!("exec not implemented");
}

extern "C" fn syscall_7_exit(_args: RegistersMini, _isf: InterruptStackFrame) {
    panic!("exit not implemented");
}

extern "C" fn syscall_8_wait(_args: RegistersMini, _isf: InterruptStackFrame) {
    panic!("wait not implemented");
}

extern "C" fn syscall_test(_args: RegistersMini, _isf: InterruptStackFrame) {
    debug!("Test syscall.");
}

extern "C" fn syscall_not_implemented(_args: RegistersMini, _isf: InterruptStackFrame) {
    panic!("not implemented")
}

/// dispatch function who gives control to the good syscall function
extern "C" fn syscall_dispatch(args: RegistersMini, isf: InterruptStackFrame) {
    if args.rax >= SYSCALL_NUMBER {
        panic!("no such syscall : {:?}", args);
    } else {
        SYSCALL_TABLE[args.rax as usize](args, isf)
    }
}

/// interface function for syscalls, saves every register before giving control to the dispatch function
/// it disables interrupts at entry !
#[naked]
pub extern "C" fn naked_syscall_dispatch() {
    unsafe {
        asm!(
        "cli",
        "push r15",
        "push r14",
        "push r13",
        "push r12",
        "push r11",
        "push rbp",
        "push rcx",
        "push rbx",
        "push rax",
        "push rdi",
        "push rsi",
        "push rdx",
        "push r10",
        "push r8",
        "push r9",
        "mov rdi, rsp",
        "mov rsi, rsp",
        "add rsi, 15*8",
        "call {0}",
        "pop r9",
        "pop r8",
        "pop r10",
        "pop rdx",
        "pop rsi",
        "pop rdi",
        "pop rax",
        "pop rbx",
        "pop rcx",
        "pop rbp",
        "pop r11",
        "pop r12",
        "pop r13",
        "pop r14",
        "pop r15",
        "sti",
        "iretq",
          sym syscall_dispatch
        );
    }
}
