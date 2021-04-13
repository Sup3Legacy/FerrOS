#![allow(dead_code)]

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use hashbrown::hash_map::DefaultHashBuilder;
use priority_queue::PriorityQueue;
use x86_64::instructions::port::Port;

use crate::{debug, println};

/// Max number of sounds in a session.
const MAX_SOUND: u64 = 4096;

/// Current sound tick. Incremented at each clock tick.
/// TODO add guard to avoid overflowing this variable!
static mut TICK: u64 = 0;

fn get_tick() -> u64 {
    unsafe { TICK }
}

/// Never call this function without a very good reason
fn incr_tick() {
    unsafe {
        TICK += 1;
    }
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
struct SoundID(u64);

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
struct SoundElement {
    id: SoundID,
    tone: u32,
    length: u64,
    begin: u64,
}

/// First element is the beginning time, second the priority (tick at which is was enqueued)
#[derive(Hash, Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
struct SoundPriority(u64, SoundID);

impl SoundPriority {
    pub fn new(begin: u64) -> Self {
        Self(begin, SoundID::new())
    }
}

pub struct SoundQueue(
    PriorityQueue<SoundElement, SoundPriority, DefaultHashBuilder>,
    Option<SoundElement>,
);

impl SoundID {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(core::u64::MAX - 1);
        let new = NEXT_ID.fetch_sub(1, Ordering::Relaxed); // Maybe better to reallow previous numbers
        if new <= (core::u64::MAX - MAX_SOUND) {
            panic!("Reached maximum number of sounds!");
        }
        Self(new)
    }
}

impl SoundElement {
    fn new(tone: u32, length: u64, begin: u64) -> Self {
        Self {
            id: SoundID::new(),
            tone,
            length,
            begin,
        }
    }
}

impl SoundQueue {
    pub fn new() -> Self {
        Self(PriorityQueue::with_default_hasher(), None)
    }
    pub fn create_and_enqueue(&mut self, tone: u32, length: u64, begin: u64) {
        debug!(
            "Pushed sound : {}, {}, {}",
            tone,
            length,
            begin + get_tick()
        );
        self.0.push(
            SoundElement::new(tone, length, begin + get_tick()),
            SoundPriority::new(core::u64::MAX - (begin + get_tick())),
        );
    }
    pub fn mute(&self) {
        unsafe {
            let mut port61 = Port::new(0x61);
            let tmp: u8 = port61.read() & 0xFC;

            port61.write(tmp);
        }
    }

    pub fn handle(&mut self) {
        incr_tick();
        if let Some(sound) = self.1 {
            if sound.begin + sound.length > get_tick() {
                // Meaning the sound is still playing
                return;
            }
        }
        while let Some((sound_element, sound_priority)) = self.0.pop() {
            // If there is at least one sound
            if sound_element.begin <= get_tick() {
                if sound_element.begin + sound_element.length > get_tick() {
                    // Meaning there is now a sound to play
                    self.1 = Some(sound_element);
                    self.play_sound(sound_element);
                    return;
                } else {
                    println!("Outdated sound");
                    // The sound is outdated
                    // Simply keep poping elements
                }
            } else {
                // If there is no sound to play for now
                self.0.push(sound_element, sound_priority);
                self.1 = None;
                self.mute();
                return;
            }
        }
        self.mute();
        self.1 = None;
    }

    fn play_sound(&self, element: SoundElement) {
        println!("Gonna play sound");
        unsafe {
            let div: u32 = 1193180 / element.tone;

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
