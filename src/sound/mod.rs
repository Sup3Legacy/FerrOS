use lazy_static::lazy_static;
use x86_64::instructions::port::Port;

mod soundQueue;

lazy_static! {
    static ref SOUND_QUEUE: soundQueue::SoundQueue = soundQueue::SoundQueue::new();
}
