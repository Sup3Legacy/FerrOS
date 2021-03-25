use lazy_static::lazy_static;


mod sound_queue;

lazy_static! {
    static ref SOUND_QUEUE: sound_queue::SoundQueue = sound_queue::SoundQueue::new();
}
