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
use x86_64::structures::paging::PageTableFlags;
use x86_64::registers::control::Cr3;
/// # The core of the FerrOS operating system.
/// It's here that we perform the Frankenstein magic of assembling all the parts together.
use crate::task::{executor::Executor, Task};
use ferr_os::{
    allocator, data_storage, debug, errorln, filesystem, gdt, halt_loop, hardware, initdebugln,
    interrupts, keyboard, long_halt, memory, print, println, scheduler, serial, sound, task, test_panic, vga,
    warningln
};



extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;

/// # Panic handling
/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    errorln!("{}", _info);
    hardware::power::shutdown();
    halt_loop();
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
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoAllocator::init(&_boot_info.memory_map, phys_mem_offset) };
    allocator::init(&mut mapper, &mut frame_allocator).expect("Heap init failed :((");

    // I/O Initialization
    keyboard::init();
    //vga::init();

    println!(":(");

    // Interrupt initialisation put at the end to avoid messing up with I/O
    interrupts::init();
    println!(":( :(");

    long_halt(10);

    unsafe {
        match frame_allocator.allocate_level_4_frame() {
            Ok(level_4_addr) => {
                let addr: u64 = 54975581388800; 
                let data: *const [u64; 512] = VirtAddr::from_ptr(&ferr_os::LOL as *const u8).as_mut_ptr();

                println!("add entry to address : 0x{:X} {:#?}", addr, VirtAddr::new(addr));
                match frame_allocator.add_entry_to_table_with_data(level_4_addr, VirtAddr::new(addr),
                                PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT, &*data) {
                    Ok(()) => (),
                    Err(()) => {errorln!("error didn't allocate"); 
                        hardware::power::shutdown();
                        },

                };
                warningln!("worked");
                let addr2: u64 = addr * 2 - 1;
                let mut stackRaw : [u64; 512] = [0; 512];
                stackRaw[511] =  0; // arbitrary !
                stackRaw[510] =  addr; // rip
                stackRaw[509] =  8; // cs
                stackRaw[508] =  518; // cpu_f
                stackRaw[507] =  addr2 + 4096 - 8;
                stackRaw[506] =  0; // ss
                match frame_allocator.add_entry_to_table_with_data(level_4_addr, VirtAddr::new(addr2),
                                PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE | PageTableFlags::WRITABLE, &stackRaw) {
                    Ok(()) => (),
                    Err(()) => {errorln!("error didn't allocate 2"); 
                        hardware::power::shutdown();
                        },

                };
                let (cr3, cr3f) = Cr3::read();
                Cr3::write(level_4_addr, cr3f);
                hardware::power::shutdown();
                warningln!("jump addr : 0x{:X}", scheduler::process::towards_user as u64);
                //asm!("int 0");
                scheduler::process::towards_user(addr2, addr);
                hardware::power::shutdown();
            },

            Err(()) => hardware::power::shutdown()
        }
    }
    unsafe {
        asm!("jmp {0}", in(reg) &ferr_os::LOL as *const u8);
    }

    unsafe {
        asm!("mov rax, 1", "int 80h",);
    }
    debug!("{:?}", unsafe { hardware::clock::Time::get() });
    //hardware::power::shutdown();
    loop {}
    errorln!("Ousp");
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
