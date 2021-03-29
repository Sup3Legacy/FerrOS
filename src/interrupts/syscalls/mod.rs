#![allow(dead_code)]

//! Part of the OS responsible for handling syscalls

use super::idt::InterruptStackFrame;
use crate::data_storage::registers::{Registers, RegistersMini};
use crate::debug;
use crate::hardware;
use crate::scheduler::process;
use x86_64::registers::control::Cr3;
use x86_64::VirtAddr;

/// type of the syscall interface inside the kernel
pub type SyscallFunc = extern "C" fn();

/// total number of syscalls
const SYSCALL_NUMBER: u64 = 11;

/// table containing every syscall functions
const SYSCALL_TABLE: [extern "C" fn(&mut RegistersMini, &mut InterruptStackFrame);
    SYSCALL_NUMBER as usize] = [
    syscall_0_read,
    syscall_1_write,
    syscall_2_open,
    syscall_3_close,
    syscall_4_dup2,
    syscall_5_fork,
    syscall_6_exec,
    syscall_7_exit,
    syscall_8_wait,
    syscall_9_shutdown,
    syscall_test,
];

/// highly dangerous function should use only when knowing what you are doing
#[naked]
unsafe extern "C" fn convert_register_to_full(_args: &mut RegistersMini) -> &'static mut Registers {
    asm!("mov rax, rdi", "ret", options(noreturn));
}

/// read. arg0 : unsigned int fd, arg1 : char *buf, size_t count
extern "C" fn syscall_0_read(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("read not implemented")
}

/// write. arg0 : unsigned int fd, arg1 : const char *buf, size_t count
extern "C" fn syscall_1_write(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    debug!("write congrats you just called the good syscall!")
}

/// open file. arg0 : const char *filename, arg1 : int flags, arg2 : umode_t mode
extern "C" fn syscall_2_open(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    args.rax = 1;
    debug!("test1");
}

/// close file. arg0 : unsigned int fd
extern "C" fn syscall_3_close(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("close not implemented")
}

extern "C" fn syscall_4_dup2(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("dup2 not implemented");
}

extern "C" fn syscall_5_fork(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    debug!("fork");
    let _rax = args.rax;
    unsafe {
        args.rax = 0;
        let mut current = process::get_current_as_mut();
        let (cr3, cr3f) = Cr3::read();
        current.cr3 = cr3.start_address();
        current.cr3f = cr3f;
        current.rsp = VirtAddr::from_ptr(args).as_u64();
        let next: u64 = process::fork();
        args.rax = next;
        process::leave_context(current.rsp);
    }
}

extern "C" fn syscall_6_exec(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("exec not implemented");
}

extern "C" fn syscall_7_exit(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("exit not implemented");
}

extern "C" fn syscall_8_wait(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("wait not implemented");
}

extern "C" fn syscall_9_shutdown(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    debug!("Shutting the computer of with output {}", args.rdi);
    hardware::power::shutdown();
}

extern "C" fn syscall_test(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    debug!("Test syscall.");
}

extern "C" fn syscall_not_implemented(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("not implemented")
}

/// dispatch function who gives control to the good syscall function
pub extern "C" fn syscall_dispatch(isf: &mut InterruptStackFrame, args: &mut RegistersMini) {
    if args.rax >= SYSCALL_NUMBER {
        panic!("no such syscall : {:?}", args);
    } else {
        SYSCALL_TABLE[args.rax as usize](args, isf)
    }
}

/// interface function for syscalls, saves every register before giving control to the dispatch function
/// it disables interrupts at entry !
/// DEPRECIATED
#[naked]
pub extern "C" fn naked_syscall_dispatch() {
    unsafe {
        asm!(
            "cli",
            "sub rsp, 32",
            "vmovapd [rsp], ymm0",
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
            "mov rsi, rsp",
            "mov rdi, rsp",
            "add rdi, 15*8 + 32",
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
            "vmovapd ymm0, [rsp]",
            "add rsp, 32",
            "sti",
            "iretq",
            sym syscall_dispatch,
            options(noreturn)
        );
    }
}
