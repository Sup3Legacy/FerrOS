use crossbeam_queue::ArrayQueue;
use conquer_once::spin::OnceCell;
use crate::print;
use crate::println;

static SCANCODE_QUEUE : OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

static SCANCODE_QUEUE_CAP : usize = 10;

pub struct ScancodeStream {
    _private : () // Pour empêcher de contruire cette structure depuis l'extérieur
}

pub fn add_scancode(scancode : u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("Scancode queue full; dropping keyboard input.");
        } 
    } else {
        println!("Scancode queue uninitialized.");
        ScancodeStream::new();
    }
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(SCANCODE_QUEUE_CAP)).expect("Scancode queue should only be initialized once.");
        ScancodeStream {_private : ()}
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