#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;

mod vga;

lazy_static! { 
    pub static ref SCREEN : Mutex<vga::Screen> = Mutex::new(vga::Screen{
        col_pos : 0,
        row_pos : 0,
        color : vga::ColorCode::new(vga::Color::Yellow, vga::Color::Black),
        buffer : unsafe { &mut *(0xb8000 as *mut vga::BUFFER) },
    });
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    writeln!(SCREEN.lock(), "{}", _info).unwrap();
    loop {}
}

/// This is the starting function. Its name must not be changeed by the compiler, hence the `#![no_mangle]`
#[no_mangle]
pub extern "C" fn _start() -> ! {
    SCREEN.lock().clear().unwrap();

    SCREEN.lock().write_byte(b'H');
    SCREEN.lock().write_string("ello \n");
    SCREEN.lock().write_string("Wörld! \n");
    write!(SCREEN.lock(), "Test : {}", 42).unwrap();

    for i in 0..10 {
        writeln!(SCREEN.lock(), "{}", i).unwrap();
    }

    for i in 0..1000000 {
        write!(SCREEN.lock(), "{}/1000000", i).unwrap();
        SCREEN.lock().write_byte(b'\r');
    }

    loop{}
}
