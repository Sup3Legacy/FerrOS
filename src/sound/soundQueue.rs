use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicU64, Ordering};

/// Max number of sounds in a session.
const MAX_SOUND: u64 = 4096;

#[derive(Copy, Clone, Debug)]
struct SoundID(u64);

#[derive(Copy, Clone, Debug)]
struct SoundElement {
    id: SoundID,
    tone: u64,
    length: u64,
}

pub struct SoundQueue(VecDeque<SoundElement>);

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
    fn new(tone: u64, length: u64) -> Self {
        Self {
            id: SoundID::new(),
            tone,
            length,
        }
    }
}

impl SoundQueue {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
    pub fn create_and_enqueue(&mut self, tone: u64, length: u64) {
        self.0.push_back(SoundElement::new(tone, length))
    }
}
