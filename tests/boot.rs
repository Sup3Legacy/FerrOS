#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ferr_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use ferr_os::print;
use ferr_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ferr_os::test_panic(info)
}

#[test_case]
fn test_println() {
    println!("testing println")
}
