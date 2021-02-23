use crate::{print, println};
use x86_64::instructions::port::Port;

fn play_sound(frequence: u32) {
    unsafe {
        let div: u32 = 1193180 / frequence;

        let mut port43 = Port::new(0x43);
        port43.write(0b1011_0110 as u8); // 0b1011_0110

        let mut port42 = Port::new(0x42);
        port42.write(div as u8);
        port42.write((div >> 8) as u8);
        let mut port61 = Port::new(0x61);
        let tmp: u8 = port61.read();
        if (tmp & 3 != 3) {
            port61.write(tmp | 0b1111_1111);
        }
    }
}

fn nosound() {
    unsafe {
        let mut port61 = Port::new(0x61);
        let tmp: u8 = port61.read() & 0xFC;

        port61.write(tmp);
    }
}

pub fn beep() {
    unsafe {
        play_sound(1000);
    }
    crate::long_halt(3);
    nosound();
}
