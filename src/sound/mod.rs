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
        let mut port61 = Port::new(0x61);
        let tmp: u8 = port61.read();
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
    crate::long_halt(3);
    nosound();
}

pub fn set_freq_100( frequence: u32) {
    unsafe {
        let div: u32 = 1193180*100 / frequence;

        let mut port43 = Port::new(0x43);
        port43.write(0b1011_0110 as u8);// 0b1011_0110
        
        let mut port42 = Port::new(0x42);
        port42.write( div as u8);
        port42.write( (div >> 8) as u8);
        let mut port61 = Port::new(0x61);
        let tmp: u8 = port61.read();
        if (tmp & 3 != 3) {
            port61.write(tmp | 3);
        }
    }
}

pub fn open_output() {
    unsafe {
        let mut port61 = Port::new(0x61);
        let tmp: u8 = port61.read();
        port61.write(tmp | 3);
    }
}

pub fn close_output() {
    unsafe {
        let mut port61 = Port::new(0x61);
        let tmp: u8 = port61.read() & 0xFC;

        port61.write(tmp);
    }
}

fn waiting() {
    crate::long_halt(2);
}

pub fn Do() {
    set_freq_100(104650);
    waiting();
}

pub fn do_() {
    set_freq_100(110873);
    waiting();
}

pub fn re() {
    set_freq_100(117466);
    waiting();
}

pub fn re_() {
    set_freq_100(124451);
    waiting();
}

pub fn mi() {
    set_freq_100(1318_51);
    waiting();
}

pub fn fa() {
    set_freq_100(1396_91);
    waiting();
}

pub fn fa_() {
    set_freq_100(1479_98);
    waiting();
}

pub fn sol() {
    set_freq_100(1567_98);
    waiting();
}

pub fn sol_() {
    set_freq_100(1661_22);
    waiting();
}

pub fn la() {
    set_freq_100(1760_00);
    waiting();
}

pub fn la_() {
    set_freq_100(1864_66);
    waiting();
}

pub fn si() {
    set_freq_100(1975_53);
    waiting();
}