#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
mod vga;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    use core::fmt::Write;
    let mut screen = vga::SCREEN::new(
        vga::ColorCode::new(vga::Color::Yellow, vga::Color::Black),
        unsafe { &mut *(0xb8000 as *mut vga::BUFFER) },
    );
    writeln!(screen, "{}", _info).unwrap();
    loop {}
}

/// This is the starting function. Its name must not be changeed by the compiler, hence the `#![no_mangle]`
#[no_mangle]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;
    let mut screen = vga::SCREEN::new(
        vga::ColorCode::new(vga::Color::Yellow, vga::Color::Black),
        unsafe { &mut *(0xb8000 as *mut vga::BUFFER) },
    );

    screen.clear().unwrap();

    screen.write_byte(b'H');
    screen.write_string("ello \n");
    screen.write_string("Wörld! \n");
    write!(screen, "Test : {}", 42).unwrap();

    for i in 0..10 {
        writeln!(screen, "{}", i).unwrap();
    }

    loop{}
}
