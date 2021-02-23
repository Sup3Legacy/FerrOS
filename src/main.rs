#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(gen_future)]
#![feature(custom_test_frameworks)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]
#![feature(asm)]
#![test_runner(ferr_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(const_mut_refs)]

use core::panic::PanicInfo;
// use os_test::println;  TODO
//use core::task::Poll;
use bootloader::{entry_point, BootInfo};
extern crate vga as vga_video;
//use vga as vga_video;
mod programs;
use x86_64::addr::VirtAddr; //, VirtAddrNotValid};
                            //use x86_64::structures::paging::Translate;
mod filesystem;
/// # The core of the FerrOS operating system.
/// It's here that we perform the Frankenstein magic of assembling all the parts together.
use crate::task::{executor::Executor, Task};
use ferr_os::{
    allocator, gdt, halt_loop, interrupts, keyboard, long_halt, memory, print, println, serial,
    task, test_panic, vga, sound, data_storage
};

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;

/// # Panic handling
/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    halt_loop();
}

/// # Initialization
/// Initializes the configurations
pub fn init(_boot_info: &'static BootInfo) {
    gdt::init();

    // Memory allocation Initialization
    let phys_mem_offset = VirtAddr::new(_boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoAllocator::init(&_boot_info.memory_map) };
    allocator::init(&mut mapper, &mut frame_allocator).expect("Heap init failed :((");

    // I/O Initialization
    keyboard::init();
    vga::init();

    // Interrupt initialisation put at the end to avoid messing up with I/O
    interrupts::init();
}

// test taks, to move out of here
async fn task_1() {
    loop {
        ferr_os::print!("X");
        long_halt(16);
    }
}

// test task, to move out of here
async fn task_2() {
    loop {
        print!("0");
        long_halt(16);
    }
}

entry_point!(kernel_main);
// We use it to check a)t compile time that we are doing everything correctly with the arguments of `kernel_main`

/// # Entry point
/// This is the starting function, it's here that the bootloader sends us to when starting the system.
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    init(_boot_info);
    // Why is this not in the init function ?


    // quelques tests de drive

    filesystem::test::test();

    // fin des tests

    // This enables the tests
    #[cfg(test)]
    test_main();

    sound::beep();
    // Yet again, some ugly tests in main
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
    vga::_print_at(2, 2, "loul");
    let mut executor = Executor::new();
    executor.spawn(Task::new(task_1()));
    executor.spawn(Task::new(task_2()));
    executor.run();
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    test_panic(_info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
