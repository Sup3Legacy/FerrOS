use crate::println;
use conquer_once::spin::OnceCell;
use crossbeam_queue::{ArrayQueue, PopError, PushError};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

mod keyboard_layout;

pub mod keyboard_interaction;

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
        if queue.push(scancode).is_err() {
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

pub fn get_top_key_event() -> Result<u8, PopError> {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
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

impl Default for ScancodeStream {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::empty_loop)]
pub fn set_keyboard_responce(freq: u8, _tim: u8) {
    disable_keyboard();
    let mut command_port = Port::new(0x64);
    let mut data_port = Port::new(0x60);
    unsafe {
        let mut i2: u8 = 0xFE;
        while i2 == 0xFE {
            command_port.write(0xF3_u8);
            i2 = data_port.read();
            println!("{}", i2);
        }
        i2 = 0xFE;
        while i2 == 0xFE {
            data_port.write(0_u8);
            command_port.write(freq);
            i2 = data_port.read();
            println!("{}", i2);
        }

        let i1 = command_port.read();
        let i2 = data_port.read();
        println!("{} {}", i1, i2);
        println!("{} {}", 0xFA, 0xFE)
    }
    //loop {};
    enable_keyboard();
    loop {}
}

pub fn set_layout(code: u8) {
    if code < MAX_LAYOUT {
        let mut k = KEYBOARD_STATUS.lock();
        k.set(code);
    }
}

pub fn disable_keyboard() {
    let mut command_port = Port::new(0x64);
    let mut data_port = Port::new(0x60);

    unsafe {
        data_port.write(0_u8);
        command_port.write(0xF5_u8);
        let i2: u8 = data_port.read();
        println!("disable : {}", i2);
    }
}

pub fn enable_keyboard() {
    let mut command_port = Port::new(0x64);
    let mut data_port = Port::new(0x60);
    unsafe {
        data_port.write(0_u8);
        command_port.write(0xF4_u8);
        let i2: u8 = data_port.read();
        println!("{} = disable", i2);
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
