#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;

mod vga;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

/// This is the starting function. Its name must not be changeed by the compiler, hence the `#![no_mangle]`
#[no_mangle]
pub extern "C" fn _start() -> ! {
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

    for i in 0..1000000 {
        print!("{}/1000000", i);
        vga::write_back();
    }

    loop{}
}
