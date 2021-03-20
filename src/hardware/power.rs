use x86_64::instructions::port::Port;
use crate::{errorln, print, warningln};

pub fn shutdown() {
    unsafe {
        warningln!("Sending shutdown signal to QEMU.");
        let mut shutdown = Port::new(0x604);
        shutdown.write(0x2000_u16);
    }
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
        )
    }
}

#[no_mangle]
fn failed_shutdown() {
    errorln!("Failed to shutdown!");
}