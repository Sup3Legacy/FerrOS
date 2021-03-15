use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};
use hashbrown::hash_map::DefaultHashBuilder;
use priority_queue::PriorityQueue;

use super::virtual_screen::{ColorCode, VirtualScreen, VirtualScreenLayer, CHAR};

/// Height of the screen
const BUFFER_HEIGHT: usize = 25;

/// Width of the screen
const BUFFER_WIDTH: usize = 80;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct VirtualScreenID(u64);

impl VirtualScreenID {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let new = NEXT_ID.fetch_add(1, Ordering::Relaxed); // Maybe better to reallow previous numbers
        Self(new)
    }
}

#[derive(Debug)]
pub struct MainScreen {
    /// Conversion id -> screen
    map: BTreeMap<VirtualScreenID, VirtualScreen>,
    /// queue on id based on layer priority
    queue: PriorityQueue<VirtualScreenID, VirtualScreenLayer, DefaultHashBuilder>,
    /// back-up queue
    roll_queue: PriorityQueue<VirtualScreenID, VirtualScreenLayer, DefaultHashBuilder>,
}

impl MainScreen {}
