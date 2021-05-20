//! Sound driver and logic. Used by the VFS.

use lazy_static::lazy_static;
use spin::Mutex;



mod sound_queue;

lazy_static! {
    /// Sound driver
    static ref SOUND_QUEUE: Mutex<sound_queue::SoundQueue> =
        Mutex::new(sound_queue::SoundQueue::new());
}

/// Updates the sound-driver. This need to happen periodically (e.g. at each Timer-interrupt).
pub fn handle() {
    SOUND_QUEUE.lock().handle();
}

/// Main entry-point to add a sound into the sound-driver.
///
/// This fucntion can be called directly by the kernel or indirectly by a program through the `VFS`
pub fn add_sound(tone: u64, length: u64, begin: u64) {
    SOUND_QUEUE
        .lock()
        .create_and_enqueue(tone as u32, length, begin)
}
