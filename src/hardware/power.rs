use crate::{errorln, warningln};
use x86_64::instructions::port::Port;

/// Sends the shutdown signal.
/// It must obviously be the very last step in the shutdown process.
pub fn shutdown() -> ! {
    // This uses the special QEMU signal.
    // It is quite of a brute-force method but it works
    unsafe {
        warningln!("Sending shutdown signal to QEMU.");
        let mut shutdown = Port::new(0x604);
        shutdown.write(0x2000_u16);
    }

    // This uses the standard shutdown procedure
    // doesn't work (causes error : `Segment not found`)
    unsafe {
        asm!(
            "push rax",
            "push rbx",
            "push rcx",
            "push rsp",
            "mov ax, 0x1000",
            "mov ax, ss",
            "mov sp, 0xf000",
            "mov ax, 0x5307",
            "mov bx, 0x0001",
            "mov cx, 0x0003",
            "int 0x15",
            "pop rsp",
            "pop rcx",
            "pop rbx",
            "pop rax",
            "call failed_shutdown",
            "ret",
            options(noreturn,),
        )
    }
}

#[no_mangle]
fn failed_shutdown() {
    errorln!("Failed to shutdown!");
}
