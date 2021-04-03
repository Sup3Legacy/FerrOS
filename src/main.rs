#![allow(unused_imports)]
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

use alloc::vec;
use alloc::vec::Vec;
use bit_field::BitArray;
use core::panic::PanicInfo;
use lazy_static::lazy_static;

// use os_test::println;  TODO
//use core::task::Poll;
use bootloader::{entry_point, BootInfo};
extern crate vga as vga_video;
//use vga as vga_video;
mod programs;
use x86_64::addr::VirtAddr; //, VirtAddrNotValid};
                            //use x86_64::structures::paging::Translate;
/// # The core of the FerrOS operating system.
/// It's here that we perform the Frankenstein magic of assembling all the parts together.
use ferr_os::{
    allocator, data_storage, debug, errorln, filesystem, gdt, halt_loop, hardware, initdebugln,
    interrupts, keyboard, long_halt, memory, print, println, scheduler, serial, sound, test_panic,
    vga, warningln, _TEST_PROGRAM,
};
use x86_64::instructions::random::RdRand;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PageTableFlags;
use xmas_elf::ElfFile;

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;

/// # Panic handling
/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
#[allow(unreachable_code)]
fn panic(_info: &PanicInfo) -> ! {
    errorln!("{}", _info);
    hardware::power::shutdown();
    halt_loop();
}

#[naked]
/// # Just don't call it
/// Test function that is given to launcher
/// It forks itselfs :
/// - the father loops
/// - the son shuts down the computer
/// Result : SUCCESS :D
pub unsafe extern "C" fn test_syscall() {
    asm!(
        "mov rax, 42",
        "mov rax, 1", // syscall 1 == test (good syscall)
        "int 80h",
        "mov rax, 5", // syscall 5 == fork
        "int 80h",
        "loop:", // the fathers loops
        "cmp rax, 0",
        "jnz loop",
        "mov rdi, rax",
        "mov rax, 9", // syscall 9 == shutdown
        "int 80h",
        "ret",
        options(noreturn)
    )
}

/// # Initialization
/// Initializes the configurations
pub fn init(_boot_info: &'static BootInfo) {
    initdebugln!();
    println!("Ceci est simplement un debug :)");
    warningln!("Ceci est un warning :|");
    errorln!("Ceci est une erreur :(");
    gdt::init();

    // Memory allocation Initialization
    let phys_mem_offset = VirtAddr::new(_boot_info.physical_memory_offset);
    println!("Physical memory offset : 0x{:x?}", phys_mem_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    unsafe {
        memory::BootInfoAllocator::init(&_boot_info.memory_map, phys_mem_offset);
        if let Some(frame_allocator) = &mut memory::FRAME_ALLOCATOR {
            let (level_4_frame, _) = Cr3::read();
            frame_allocator
                .deallocate_level_4_page(level_4_frame.start_address(), PageTableFlags::BIT_9)
                .expect("Didn't manage to clean bootloader data");
            allocator::init(&mut mapper, frame_allocator).expect("Heap init failed :((");
        } else {
            panic!("Frame allocator wasn't initialized");
        }
    };

    // I/O Initialization
    keyboard::init();
    //vga::init();

    println!(":(");

    println!("try to change counter");
    unsafe {
        hardware::timer::set_timer(0x0000); // 0 = 0x10000 = frequence min
    }
    println!("checked");

    // Interrupt initialisation put at the end to avoid messing up with I/O
    interrupts::init();
    println!(":( :(");

    long_halt(0);

    println!("Random : {:?}", RdRand::new().unwrap().get_u64().unwrap());

    /* unsafe {
        asm!(
            "mov rdi, 42",
            "mov rax, 9", "int 80h",);
    }*/
    debug!("{:?}", unsafe { hardware::clock::Time::get() });
    scheduler::process::spawn_first_process();
    unsafe {
        filesystem::init_vfs();
    }
    //hardware::power::shutdown();
    //loop {}
    //errorln!("Ousp");
    //filesystem::init();
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
// We use it to check at compile time that we are doing everything correctly with the arguments of `kernel_main`

/// # Entry point
/// This is the starting function, it's here that the bootloader sends us to when starting the system.
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    init(_boot_info);

    unsafe {
        if let Some(frame_allocator) = &mut memory::FRAME_ALLOCATOR {
            scheduler::process::disassemble_and_launch(_TEST_PROGRAM, frame_allocator, 1, 2);
        }
    }

    //unsafe{asm!("mov rcx, 0","div rcx");}
    // This enables the tests
    #[cfg(test)]
    test_main();
    // Yet again, some ugly tests in main
    println!(":( :( :(");
    programs::shell::main_shell();
    println!();
    for i in 0..5 {
        println!("{}", i);
    }
    for i in 0..5 {
        println!("{},", i);
    }

    println!();

    let _x = Box::new([0, 1]);
    let y = String::from("Loul");
    println!("{}", y);
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
