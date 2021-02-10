#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(wake_trait)]
#![feature(gen_future)]
#![feature(custom_test_frameworks)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]
#![feature(llvm_asm)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(const_mut_refs)]

use core::panic::PanicInfo;
//use core::task::Poll;
use bootloader::{entry_point, BootInfo};
mod programs;
use x86_64::addr::VirtAddr; //, VirtAddrNotValid};
                            //use x86_64::structures::paging::Translate;
mod vga;
use vga::_print_at;
mod allocator;
mod gdt;
mod interrupts;
mod keyboard;
mod memory;
mod task;

use crate::task::{executor::Executor, Task};

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    halt_loop();
}

pub fn halt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn long_halt(i: usize) {
    for _ in 0..i {
        x86_64::instructions::hlt();
    }
}

pub fn init() {
    interrupts::init();
    gdt::init();
}

async fn task_1() {
    loop {
        print!("X");
        long_halt(16);
    }
}

async fn task_2() {
    loop {
        print!("0");
        long_halt(16);
    }
}

entry_point!(kernel_main);
/// This is the starting function.
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    let phys_mem_offset = VirtAddr::new(_boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoAllocator::init(&_boot_info.memory_map) };
    allocator::init(&mut mapper, &mut frame_allocator).expect("Heap init failed :((");

    keyboard::init();
    vga::init();

    // This enables the tests
    #[cfg(test)]
    test_main();

    programs::shell::main_shell();
    println!();
    for i in 0..5 {
        println!("{}", i);
    }
    for i in 0..5 {
        println!("{},", i);
    }

    for i in 0..10000 {
        print!("{}/1000000", i);
        vga::write_back();
    }
    println!();

    let _x = Box::new([0, 1]);
    let y = String::from("Loul");
    println!("{}", y);
    crate::_print_at(2, 2, "loul");
    let mut executor = Executor::new();
    executor.spawn(Task::new(task_1()));
    executor.spawn(Task::new(task_2()));
    executor.run();
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}
