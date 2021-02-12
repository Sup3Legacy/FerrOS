use crate::{print, println};
use conquer_once::spin::OnceCell;
use crossbeam_queue::{ArrayQueue, PopError, PushError};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

mod keyboard_layout;

pub mod keyboard_interraction;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static KEY_QUEUE: OnceCell<ArrayQueue<keyboard_layout::KeyEvent>> = OnceCell::uninit();

const MAX_LAYOUT: u8 = 2;

lazy_static! {
    pub static ref KEYBOARD_STATUS: Mutex<keyboard_layout::KeyBoardStatus> =
        Mutex::new(keyboard_layout::KeyBoardStatus::new(0));
}

static SCANCODE_QUEUE_CAP: usize = 10;

pub struct ScancodeStream {
    _private: (), // Pour empêcher de contruire cette structure depuis l'extérieur
}

pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("Scancode queue full; dropping keyboard input.");
        }
    } else {
        println!("Scancode queue uninitialized.");
        ScancodeStream::new();
    }
}

#[allow(unused_must_use)]
pub fn process() {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Ok(queue2) = KEY_QUEUE.try_get() {
            match queue.pop() {
                Err(_) => (),
                Ok(key) => {
                    // Change layout when pressing TAB or Win-Space.
                    if key == 15 || (KEYBOARD_STATUS.lock().gui() && key == 57) {
                        let i = KEYBOARD_STATUS.lock().get_id();
                        set_layout((i + 1) % MAX_LAYOUT);
                    } else {
                        match KEYBOARD_STATUS.lock().process(key) {
                            keyboard_layout::Effect::Nothing => (),
                            keyboard_layout::Effect::Value(mut v) => {
                                while let Err(PushError(v2)) = queue2.push(v) {
                                    queue2.pop();
                                    v = v2;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn get_top_value() -> Result<keyboard_layout::KeyEvent, PopError> {
    process();
    if let Ok(queue) = KEY_QUEUE.try_get() {
        queue.pop()
    } else {
        Err(PopError)
    }
}

pub fn init() {
    println!("Scancode queue initialized.");
    ScancodeStream::new();
   // set_keyboard_responce(31, 3);
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(2 * SCANCODE_QUEUE_CAP))
            .expect("Scancode queue should only be initialized once.");
        KEY_QUEUE
            .try_init_once(|| ArrayQueue::new(SCANCODE_QUEUE_CAP))
            .expect("Scancode queue should only be initialized once.");
        ScancodeStream { _private: () }
    }
}

pub fn set_keyboard_responce(freq: u8, tim: u8) {
    let mut port = Port::new(0xF3);
    unsafe { port.write(((tim & 3) << 5) | (freq & 63)) }
}

pub fn set_layout(code: u8) {
    if code < MAX_LAYOUT {
        let mut k = KEYBOARD_STATUS.lock();
        k.set(code);
    }
}

/*
use alloc::collections:Box;

/// simple keyboard event element for use in the keyboard queue
#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
    code : u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Node<'a, T> {
    after : Option<&'a Node<'a, T>>,
    value : T
}

#[derive(Debug)]
pub struct Queue<'a, T> {
    begin : Option<Node<'a, T>>,
    end : Option<Node<'a, T>>,
    length : usize,
}

impl <'a, T> Queue<'a, T> {
    fn enqueue(&mut self, element : T) -> () {
        let new_node = Node{after : None, value : element};
        if let Some(end_node) = self.end {
            self.end = Some(new_node);
            end_node.after = Some(&new_node);
        } else {

        }
    }
}
*/
