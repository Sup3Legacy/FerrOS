use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};
use hashbrown::hash_map::DefaultHashBuilder;
use priority_queue::PriorityQueue;

use super::virtual_screen::{ColorCode, VirtualScreen, VirtualScreenLayer, CHAR};

use crate::{print, println};

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

    buffer: [[CHAR; BUFFER_WIDTH]; BUFFER_HEIGHT],

    /// true if the case is occupied
    alpha: [[bool; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl MainScreen {
    /// Draws the whole screen by displaying each vScreen ordered by layer
    ///
    /// A higher layer means the vScreen will be more on the foreground.
    pub fn draw(&mut self) {
        self.reset_alpha();
        while let Some((vScreenID, _layer)) = self.queue.pop() {
            if let Some(vScreen) = self.map.get(&vScreenID) {
                let position = vScreen.get_position();
                let size = vScreen.get_size();
                let row_origin = position.get_row();
                let col_origin = position.get_col();
                let row_size = size.get_row();
                let col_size = size.get_col();
                for i in 0..row_size {
                    for j in 0..col_size {
                        // The alpha layer helps ensuring we do not write to a previously
                        // written part of the screen (that is written from a vScreen
                        // with a higher layer). This is because we draw vScreens by order
                        // of decreasing layer.
                        if i + row_origin < BUFFER_HEIGHT
                            && j + col_origin < BUFFER_WIDTH
                            && !self.alpha[i + row_origin][j + col_origin]
                        {
                            self.buffer[i + row_origin][j + col_origin] = vScreen.get_char(i, j);
                            self.alpha[i + row_origin][j + col_origin] = true;
                        }
                    }
                }
            } else {
                println!("MainScreen : could not map ID to vScreen : {:?}", vScreenID);
            }
            self.roll_queue.push(vScreenID, _layer);
        }
    }

    /// Puts all item in `roll_queue` back in the `queue`
    fn spill_queue(&mut self) {
        while let Some((vScreenID, layer)) = self.roll_queue.pop() {
            self.queue.push(vScreenID, layer);
        }
    }

    /// Resets the alpha layer of the screen
    fn reset_alpha(&mut self) {
        for i in 0..BUFFER_HEIGHT {
            for j in 0..BUFFER_WIDTH {
                self.alpha[i][j] = false;
            }
        }
    }
}
