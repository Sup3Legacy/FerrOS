use crate::{print, errorln};

pub fn shutdown() {
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
            "call {0}",
            "ret",
            sym failed_shutdown
        )
    }
}

fn failed_shutdown() {
    errorln!("Failed to shutdown!");
}