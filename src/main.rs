#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::addr::{VirtAddr, VirtAddrNotValid};
use x86_64::structures::paging::Translate;
mod vga;
mod interrupts;
mod gdt;
mod memory;
mod allocator;

extern crate alloc;

use alloc::boxed::Box;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

pub fn halt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init() {
    interrupts::init();
    gdt::init();
}

entry_point!(kernel_main);
/// This is the starting function. Its name must not be changeed by the compiler, hence the `#![no_mangle]`
fn kernel_main(_boot_info : &'static BootInfo) -> ! {
    init();
    let phys_mem_offset = VirtAddr::new(_boot_info.physical_memory_offset);
    let mut mapper = unsafe {memory::init(phys_mem_offset)};
    let mut frame_allocator = unsafe {
        memory::BootInfoAllocator::init(&_boot_info.memory_map)
    };
    allocator::init(&mut mapper, &mut frame_allocator).expect("Heap init failed :((");
    for i in 0..10 {
        println!("{}", i);
    }
    for i in 0..30 {
        println!("{},", i);
    }

    for i in 0..10000 {
        print!("{}/1000000", i);
        vga::write_back();
    }

    let ptr = 0xdeadbeaf as *mut u32;
    //unsafe { *ptr = 42; }

    let x = Box::new([0, 1]);

    halt_loop();
}
