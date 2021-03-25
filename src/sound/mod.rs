use lazy_static::lazy_static;


mod soundQueue;

lazy_static! {
    static ref SOUND_QUEUE: soundQueue::SoundQueue = soundQueue::SoundQueue::new();
}
