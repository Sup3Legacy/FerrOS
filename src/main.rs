#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

use core::panic::PanicInfo;
mod vga;
mod interrupts;
//mod keyboard;
//mod allocator;

//extern crate alloc;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

/// This is the starting function. Its name must not be changeed by the compiler, hence the `#![no_mangle]`
#[no_mangle]
pub extern "C" fn _start() -> ! {
    interrupts::init_idt();
    /*
    SCREEN.lock().clear().unwrap();

    SCREEN.lock().write_byte(b'H');
    SCREEN.lock().write_string("ello \n");
    SCREEN.lock().write_string("Wörld! \n");
    write!(SCREEN.lock(), "Test : {}", 42).unwrap();
    */

    

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

    //x86_64::instructions::interrupts::int3();

    loop{}
}
