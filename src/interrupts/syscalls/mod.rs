#![allow(dead_code)]

//! Part of the OS responsible for handling syscalls

use super::idt::InterruptStackFrame;
use crate::data_storage::{
    path,
    registers::{Registers, RegistersMini},
};
use crate::filesystem;
use crate::filesystem::{descriptor, open_mode_from_flags};
use crate::hardware;
use crate::interrupts;
use crate::memory;
use crate::scheduler::process;

use crate::{scheduler};
use crate::{debug, warningln};
use alloc::string::String;
use alloc::vec::Vec;
use core::char;
use core::cmp::min;
use x86_64::{registers::control::Cr3, structures::paging::PageTableFlags, VirtAddr};

/// type of the syscall interface inside the kernel
pub type SyscallFunc = extern "C" fn();

/// total number of syscalls
const SYSCALL_NUMBER: u64 = 24;

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
    syscall_11_set_screen_size,
    syscall_12_set_screen_position,
    syscall_13_getcwd,
    syscall_14_chdir,
    syscall_15_mkdir,
    syscall_16_rmdir,
    syscall_17_get_layer,
    syscall_18_set_layer,
    syscall_19_set_focus,
    syscall_20_debug,
    syscall_21_memrequest,
    syscall_22_listen,
    syscall_23_kill,
];

/// highly dangerous function should use only when knowing what you are doing
#[naked]
unsafe extern "C" fn convert_register_to_full(_args: &mut RegistersMini) -> &'static mut Registers {
    asm!("mov rax, rdi", "ret", options(noreturn));
}

/// # Safety
/// The caller must be sure that the pointer corresponds to a valid string, that is, what's more, ended by `\u{0}`.
unsafe fn read_string_from_pointer(ptr: u64) -> String {
    let mut buf = Vec::new();
    let mut addr = ptr;
    let mut reading = *(addr as *mut u8) as char;
    while reading != '\u{0}' {
        buf.push(reading);
        addr += 1_u64;
        reading = *(addr as *mut u8) as char
    }
    let res = buf.into_iter().collect();
    debug!("read: {}", res);
    res
}

/// read. arg0 : unsigned int fd, arg1 : char *buf, size_t count
extern "C" fn syscall_0_read(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    let (cr3, _) = Cr3::read();
    let mut size = min(args.rdx, 1024) - 1;
    if memory::check_if_has_flags(
        cr3,
        VirtAddr::new(args.rsi),
        PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE,
    ) {
        if !memory::check_if_has_flags(
            cr3,
            VirtAddr::new(args.rsi + size),
            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE,
        ) {
            size = (0xFFF - args.rsi) & 0xFFF;
        }
        if args.rdi == 0 {
            args.rax = 0;
            let mut address = VirtAddr::new(args.rsi);
            for _i in 0..size {
                if let Ok(k) = crate::keyboard::get_top_key_event() {
                    unsafe {
                        *(address.as_mut_ptr::<u8>()) = k;
                    }
                    address += 1_u64;
                    args.rax += 1;
                }
            }
            unsafe {
                *(address.as_mut_ptr::<u8>()) = 0;
            }
        } else {
            let fd = args.rdi;
            args.rax = 0;
            let process = process::get_current();
            let oft_res = process
                .open_files
                .get_file_table(descriptor::FileDescriptor::new(fd as usize));
            if let Ok(oft) = oft_res {
                let res = filesystem::read_file(oft, size as usize);
                let mut address = VirtAddr::new(args.rsi);
                for item in res.iter().take(min(size as usize, res.len())) {
                    unsafe {
                        *(address.as_mut_ptr::<u8>()) = *item;
                    }
                    address += 1_u64;
                    args.rax += 1;
                }
            } else {
                warningln!("Could not get OpenFileTable");
            }
        }
    } else {
        warningln!("Address not allowed");
        args.rax = 0;
    }
}

/// write. arg0 : unsigned int fd, arg1 : const char *buf, size_t count
extern "C" fn syscall_1_write(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    let (cr3, _) = Cr3::read();
    let mut size = min(args.rdx, 1024);
    if memory::check_if_has_flags(
        cr3,
        VirtAddr::new(args.rsi),
        PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE,
    ) {
        if !memory::check_if_has_flags(
            cr3,
            VirtAddr::new(args.rsi + size),
            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE,
        ) {
            size = (0x1000 - args.rsi) & 0xFFF;
        }
        let mut address = args.rsi;
        let mut t = Vec::new();
        let mut index = 0_u64;
        unsafe {
            while index < size && index < 1024 {
                t.push(*(address as *const u8));
                address += 1_u64;
                index += 1;
            }
        }
        let fd = args.rdi;
        args.rax = 0;
        let process = process::get_current();
        let oft_res = process
            .open_files
            .get_file_table(descriptor::FileDescriptor::new(fd as usize));
        if let Ok(oft) = oft_res {
            let res = filesystem::write_file(oft, t);
            args.rax = res as u64;
        } else {
            warningln!("Could not get OpenFileTable");
            for _i in 0..10 {
                let _oft_res = process
                    .open_files
                    .get_file_table(descriptor::FileDescriptor::new(fd as usize));
            }
        }
        //}
    } else {
        warningln!("no a valid address");
        args.rax = 0;
    }
}

/// open file. arg0 : const char *filename, arg1 : int flags, arg2 : umode_t mode
extern "C" fn syscall_2_open(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    debug!("open");
    let filename = unsafe { read_string_from_pointer(args.rdi) };
    let fd = descriptor::open(filename, open_mode_from_flags(args.rsi));
    args.rax = fd.into_u64();
    let path = unsafe { read_string_from_pointer(args.rdi) };
    let current_process = unsafe { process::get_current_as_mut() };

    let fd = current_process
        .open_files
        .create_file_table(path::Path::from(&path), 0_u64)
        .into_u64();

    // Puts the fd into rax
    args.rax = fd;
}

/// close file. arg0 : unsigned int fd
extern "C" fn syscall_3_close(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    let descriptor = args.rdi;
    let ret_code = descriptor::close(descriptor);
    match ret_code {
        Ok(()) => args.rax = 0,
        Err(_) => args.rax = 1,
    };
}

extern "C" fn syscall_4_dup2(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    args.rax = process::dup2(args.rdi as usize, args.rsi as usize) as u64;
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
    let _addr: *const String = VirtAddr::new(args.rdi).as_ptr();
    let path = unsafe {
        String::from_raw_parts(args.rdi as *mut u8, args.rsi as usize, args.rsi as usize)
    };
    unsafe {
        process::elf::load_elf_for_exec(&path);
    }
}

extern "C" fn syscall_7_exit(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    unsafe {
        let new = process::process_died(interrupts::COUNTER, args.rdi);
        interrupts::COUNTER = 0;
        process::leave_context_cr3(new.cr3.as_u64() | new.cr3f.bits(), new.rsp);
    }
}

extern "C" fn syscall_8_wait(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    unsafe {
        let (next, mut old) = process::gives_switch(interrupts::COUNTER);
        interrupts::COUNTER = 0;

        let (cr3, cr3f) = Cr3::read();
        old.cr3 = cr3.start_address();
        old.cr3f = cr3f;

        old.rsp = VirtAddr::from_ptr(args).as_u64();

        process::leave_context_cr3(next.cr3.as_u64() | next.cr3f.bits(), next.rsp);
    }
}

extern "C" fn syscall_9_shutdown(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    debug!("Shutting the computer of with output {}", args.rdi);
    hardware::power::shutdown();
}

extern "C" fn syscall_10_get_puid(_args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    _args.rax = unsafe { process::CURRENT_PROCESS } as u64
}

extern "C" fn syscall_11_set_screen_size(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    let height = args.rdi as usize;
    let width = args.rsi as usize;
    debug!("resize {} {}", height, width);
    let process = process::get_current();
    let oft_res = process
        .open_files
        .get_file_table(descriptor::FileDescriptor::new(1));
    if let Ok(oft) = oft_res {
        let res = filesystem::modify_file(oft, (1 << 63) | (height << 32) | width);
        args.rax = res as u64;
    } else {
        args.rax = u64::MAX;
    }
}

extern "C" fn syscall_12_set_screen_position(
    args: &mut RegistersMini,
    _isf: &mut InterruptStackFrame,
) {
    let height = args.rdi as usize;
    let width = args.rsi as usize;
    debug!("move {} {}", height, width);
    let process = process::get_current();
    let oft_res = process
        .open_files
        .get_file_table(descriptor::FileDescriptor::new(1));
    if let Ok(oft) = oft_res {
        let res = filesystem::modify_file(oft, (height << 32) | width);
        args.rax = res as u64;
    } else {
        args.rax = u64::MAX;
    }
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

/// Syscall for requesting additionnal heap frames
/// We might want to change the maximum
extern "C" fn syscall_21_memrequest(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    // Number of requested frames
    let additional = core::cmp::max(args.rdi, 256);
    let current_process = unsafe { scheduler::process::get_current_as_mut() };
    let current_heap_size = current_process.heap_size;
    // TODO out this in a cosntant
    if current_heap_size >= 1024 {
        warningln!("Process got max allocatable heap.");
        args.rax = 0;
        return;
    }
    unsafe {
        if let Some(ref mut frame_allocator) = crate::memory::FRAME_ALLOCATOR {
            scheduler::process::allocate_additional_heap_pages(
                frame_allocator,
                current_process.heap_address + current_heap_size * 0x1000,
                additional,
                &current_process,
            );
        }
    }
    debug!("Fullfilled memrequest");
    current_process.heap_size += additional;
    args.rax = additional
}

extern "C" fn syscall_22_listen(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    let (rax, rdi) = scheduler::process::listen();
    args.rax = rax;
    args.rdi = rdi;
}

extern "C" fn syscall_23_kill(args: &mut RegistersMini, _isf: &mut InterruptStackFrame) {
    args.rax = unsafe { scheduler::process::kill(args.rdi as usize) as u64 };
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
