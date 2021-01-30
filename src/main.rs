#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
mod vga;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &[u8] = b"Hello World!";

#[no_mangle] // start function
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;
    let mut screen = vga::SCREEN::new(
        vga::ColorCode::new(vga::Color::Yellow, vga::Color::Black),
        unsafe { &mut *(0xb8000 as *mut vga::BUFFER) },
    );

    screen.write_byte(b'H');
    screen.write_string("ello \n");
    screen.set_color(vga::ColorCode::new(vga::Color::Blue, vga::Color::Black));
    screen.write_string("Wörld! \n");
    write!(screen, "Test : {}", 42).unwrap();

    for i in 0..30 {
        writeln!(screen, "{}", i);
    }

    loop{}
}