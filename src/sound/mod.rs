use x86_64::instructions::port::Port;
use crate::{println, print};

fn play_sound( frequence: u32) {
    unsafe {
        let div: u32 = 1193180 / frequence;

        let mut port43 = Port::new(0x43);
        port43.write(0b1011_0110 as u8);// 0b1011_0110
        
        let mut port42 = Port::new(0x42);
        port42.write( div as u8);
        port42.write( (div >> 8) as u8);
        println!("div : {}", div);
        let mut port61 = Port::new(0x61);
        let tmp: u8 = port61.read();
        println!("tmp : {} {} {}", tmp, tmp & 3, tmp | 3);
        if (tmp & 3 != 3) {
            port61.write(tmp | 3);
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
    unsafe { play_sound(1000); }
    println!("sound playing");
    loop {}
    crate::long_halt(10);
    nosound();
}