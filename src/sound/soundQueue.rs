use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::instructions::port::Port;

/// Max number of sounds in a session.
const MAX_SOUND: u64 = 4096;

#[derive(Copy, Clone, Debug)]
struct SoundID(u64);

#[derive(Copy, Clone, Debug)]
struct SoundElement {
    id: SoundID,
    tone: u32,
    length: u64,
    end: u64,
}

pub struct SoundQueue(Vec<SoundElement>);

impl SoundID {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let new = NEXT_ID.fetch_add(1, Ordering::Relaxed); // Maybe better to reallow previous numbers
        if new >= MAX_SOUND {
            panic!("Reached maximum number of processes!");
        }
        Self(new)
    }
}

impl SoundElement {
    fn new(tone: u32, length: u64, end: u64) -> Self {
        Self {
            id: SoundID::new(),
            tone,
            length,
            end,
        }
    }
}

impl SoundQueue {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn create_and_enqueue(&mut self, tone: u32, length: u64, end: u64) {
        self.0.push(SoundElement::new(tone, length, end))
    }
    pub fn mute(&self) {
        unsafe {
            let mut port61 = Port::new(0x61);
            let tmp: u8 = port61.read() & 0xFC;

            port61.write(tmp);
        }
    }
    fn play_sound(self, tone: u32) {
        unsafe {
            let div: u32 = 1193180 / tone;

            let mut port43 = Port::new(0x43);
            port43.write(0b1011_0110_u8); // 0b1011_0110

            let mut port42 = Port::new(0x42);
            port42.write(div as u8);
            port42.write((div >> 8) as u8);
            let mut port61 = Port::new(0x61);
            let tmp: u8 = port61.read();
            if tmp & 3 != 3 {
                port61.write(tmp | 0b1111_1111);
            }
        }
    }
}
