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
#![feature(asm_sym)]

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
use x86_64::{
    addr::PhysAddr,
    addr::VirtAddr,
    structures::paging::{Page, PhysFrame},
}; //, VirtAddrNotValid};
   //use x86_64::structures::paging::Translate;
/// # The core of the FerrOS operating system.
/// It's here that we perform the Frankenstein magic of assembling all the parts together.
use ferr_os::{
    allocator, data_storage, debug, errorln, filesystem, gdt, halt_loop, hardware, initdebugln,
    interrupts, keyboard, long_halt, memory, print, println, scheduler, serial, sound, test_panic,
    vga, warningln, FIRST_PROGRAM, VGA_BUFFER,
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
/// # Safety
/// TODO
pub unsafe extern "C" fn test_syscall() {
    asm!(
        "mov rax, 42",
        "mov rax, 1", // syscall 1 == test (good syscall)
        "int 80h",
        "mov rax, 5", // syscall 5 == fork
        "int 80h",
        "2:", // the fathers loops
        "cmp rax, 0",
        "jnz 2b",
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
    println!("{:?}", x86_64::registers::control::Cr4::read());
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
            frame_allocator.deallocate_level_4_page(
                level_4_frame.start_address(),
                PageTableFlags::BIT_9,
                false,
            );
            VGA_BUFFER += _boot_info.physical_memory_offset;
            allocator::init(&mut mapper, frame_allocator).expect("Heap init failed :((");
        } else {
            panic!("Frame allocator wasn't initialized");
        }
    };
    // I/O Initialization
    hardware::mouse::init().unwrap();
    keyboard::init();
    //vga::init();

    //println!(":(");

    println!("Changing timer frequence");
    unsafe {
        hardware::timer::set_timer(0x8000); // 0 = 0x10000 = frequence min
    }

    // Interrupt initialisation put at the end to avoid messing up with I/O
    interrupts::init();
    //println!(":( :(");

    long_halt(0);

    //println!("Random : {:?}", RdRand::new().unwrap().get_u64().unwrap());

    debug!("{:?}", unsafe { hardware::clock::Time::get() });

    // Needs to be before `spawn_first_process`
    vga::init();

    unsafe {
        filesystem::init_vfs();
    }
    debug!("vfs initialised");
    scheduler::process::spawn_first_process();
}

entry_point!(kernel_main);
// We use it to check at compile time that we are doing everything correctly with the arguments of `kernel_main`

/// # Entry point
/// This is the starting function, it's here that the bootloader sends us to when starting the system.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);

    unsafe {
        if let Some(frame_allocator) = &mut memory::FRAME_ALLOCATOR {
            scheduler::process::disassemble_and_launch(
                FIRST_PROGRAM,
                frame_allocator,
                1,
                10,
                &Vec::new(),
                true,
            )
            .unwrap();
        }
    }

    //unsafe{asm!("mov rcx, 0","div rcx");}
    // This enables the tests
    #[cfg(test)]
    test_main();

    panic!("should not reach here !");
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
