#![allow(dead_code)]

//! Part of the OS responsible for handling syscalls

use super::idt::InterruptStackFrame;
use crate::data_storage::registers::{Registers, RegistersMini};
use crate::filesystem;
use crate::hardware;
use crate::interrupts;
use crate::scheduler::process;
use crate::{data_storage::path::Path};
use crate::{debug, errorln, warningln, println};
use alloc::string::String;
use alloc::vec::Vec;
use core::char;
use core::cmp::min;
use x86_64::registers::control::Cr3;
use x86_64::VirtAddr;

/// type of the syscall interface inside the kernel
pub type SyscallFunc = extern "C" fn();

/// total number of syscalls
const SYSCALL_NUMBER: u64 = 21;

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
    syscall_10_get_puid,
    syscall_11_get_screen,
    syscall_12_set_screen,
    syscall_13_getcwd,
    syscall_14_chdir,
    syscall_15_mkdir,
    syscall_16_rmdir,
    syscall_17_get_layer,
    syscall_18_set_layer,
    syscall_19_set_focus,
    syscall_20_debug,
];

/// highly dangerous function should use only when knowing what you are doing
#[naked]
unsafe extern "C" fn convert_register_to_full(_args: &mut RegistersMini) -> &'static mut Registers {
    asm!("mov rax, rdi", "ret", options(noreturn));
}

/// read. arg0 : unsigned int fd, arg1 : char *buf, size_t count
extern "C" fn syscall_0_read(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    if args.rdi == 0 {
        args.rax = 0;
        let mut address = VirtAddr::new(args.rsi) + 1_u64;
        for _i in 0..min(1023, args.rsi) {
            if let Ok(k) = crate::keyboard::get_top_key_event() {
                println!("About to print : {}", k);
                unsafe {
                    *(address.as_mut_ptr::<u8>()) = k;
                }
                address += 1_u64;
                args.rax += 1;
            }
        }
        
    } else {
        warningln!("Unkown file descriptor in read");
        args.rax = 0;
    }
}

/// write. arg0 : unsigned int fd, arg1 : const char *buf, size_t count
extern "C" fn syscall_1_write(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    //warningln!("printing");
    if args.rdi == 1 {
        let address = args.rsi;
        let mut data_addr = VirtAddr::new(address);
        let mut t = Vec::new();
        let mut index = 0_u64;
        if args.rdx > 0 {
            debug!("Got bytes to write!");
        }
        unsafe {
            while index < args.rdi && index < 1024 && ((*(data_addr.as_ptr::<u8>())) != 0) {
                t.push(*(data_addr.as_ptr::<u8>()));
                data_addr += 1_usize;
                index += 1;
            }
            if let Some(vfs) = &mut filesystem::VFS {
                vfs.write(Path::from("screen/screenfull"), t);
            } else {
                errorln!("Could not find VFS");
            }
        }
        args.rax = index;
    } else if args.rdi == 2 {
        let mut address = args.rsi;
        //let mut data_addr = VirtAddr::new(address);
        let mut t = String::new();
        let mut index = 0_u64;
        unsafe {
            while index < args.rdx && index < 1024 && *(address as *const u8) != 0 {
                t.push(*(address as *const u8) as char);
                address += 1_u64;
                index += 1;
            }
        }
        debug!("on shell : {}", t);
        args.rax = index;
    } else {
        warningln!("Unknow file descriptor");
        args.rax = 0;
    }
}

/// open file. arg0 : const char *filename, arg1 : int flags, arg2 : umode_t mode
extern "C" fn syscall_2_open(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    args.rax = 1;
    warningln!("test1");
}

/// close file. arg0 : unsigned int fd
extern "C" fn syscall_3_close(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    warningln!("close not implemented")
}

extern "C" fn syscall_4_dup2(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    warningln!("dup2 not implemented");
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
        let next: u64 = process::fork().0;
        args.rax = next;
        process::leave_context(current.rsp);
    }
}

/// arg0 : address of file name
extern "C" fn syscall_6_exec(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    debug!("exec");
    let addr: *const String = VirtAddr::new(args.rdi).as_ptr();
    unsafe {
        process::elf::load_elf_for_exec(&*addr);
    }
}

extern "C" fn syscall_7_exit(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("exit not implemented");
}

extern "C" fn syscall_8_wait(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    unsafe {
        interrupts::COUNTER = interrupts::QUANTUM - 1;
    }
}

extern "C" fn syscall_9_shutdown(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    debug!("Shutting the computer of with output {}", args.rdi);
    hardware::power::shutdown();
}

extern "C" fn syscall_10_get_puid(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("puid not implemented");
}

extern "C" fn syscall_11_get_screen(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("Get screen not implemented");
}

extern "C" fn syscall_12_set_screen(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("Set screen not implemented");
}

extern "C" fn syscall_13_getcwd(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("Get cwd not implemented");
}

extern "C" fn syscall_14_chdir(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("Chdir not implemented");
}

extern "C" fn syscall_15_mkdir(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("mkdir not implemented");
}

extern "C" fn syscall_16_rmdir(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("rmdir not implemented");
}

extern "C" fn syscall_17_get_layer(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("get layer not implemented");
}

extern "C" fn syscall_18_set_layer(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("set layer not implemented");
}

extern "C" fn syscall_19_set_focus(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    panic!("set focus not implemented");
}

extern "C" fn syscall_20_debug(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    debug!("rdi : {}, rsi : {}", args.rdi, args.rsi);
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
