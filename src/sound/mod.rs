use lazy_static::lazy_static;
use spin::Mutex;

use crate::println;

mod sound_queue;

lazy_static! {
    static ref SOUND_QUEUE: Mutex<sound_queue::SoundQueue> =
        Mutex::new(sound_queue::SoundQueue::new());
}

pub fn handle() {
    SOUND_QUEUE.lock().handle();
}

pub fn add_sound(tone: u64, length: u64, begin: u64) {
    println!("Received : {}, {}, {}", tone, length, begin);
    SOUND_QUEUE
        .lock()
        .create_and_enqueue(tone as u32, length, begin)
}
