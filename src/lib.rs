#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![feature(asm)]
#![feature(const_btree_new)]
#![feature(option_result_unwrap_unchecked)]
#![feature(const_raw_ptr_deref)]
#![feature(slice_as_chunks)]
#![feature(vec_into_raw_parts)]

use core::panic::PanicInfo;
extern crate vga as vga_video;

pub mod allocator;
pub mod data_storage;
pub mod filesystem;
pub mod gdt;
pub mod hardware;
pub mod interrupts;
pub mod keyboard;
pub mod memory;
pub mod programs;
pub mod scheduler;
pub mod serial;
pub mod sound;
pub mod vga;

extern crate alloc;

pub static _TEST_PROGRAM: &[u8] = include_bytes!("user_program/test_io");

pub static _TEST_PROGRAM2: &[u8] = include_bytes!("user_program/test_io2");

pub static LOL: [u8; 10] = [0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00, 0xcd, 0x80, 0xc3];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("{}...\t", core::any::type_name::<T>());
        self();
        println!("\x1B[32m[ok]\x1B[0m");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    println!("Running {} tests.", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

#[allow(clippy::empty_loop)]
pub fn test_panic(_info: &PanicInfo) -> ! {
    println!("[failed]\nError: {}\n", _info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
#[no_mangle]
#[allow(clippy::empty_loop)]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic(info)
}

/// Halts forever
pub fn halt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Halts for some time
pub fn long_halt(i: usize) {
    for _ in 0..i {
        x86_64::instructions::hlt();
    }
}
