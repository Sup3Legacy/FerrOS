use x86_64::instructions::port::Port;

pub unsafe fn set_timer(freq: u16) {
    let mut port = Port::new(0x40);
    port.write((freq & 0xFF) as u8);
    port.write((freq >> 8) as u8)
}
