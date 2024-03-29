#![allow(dead_code)]

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};
use hashbrown::hash_map::DefaultHashBuilder;
use priority_queue::PriorityQueue;

use super::virtual_screen::{ColorCode, VirtualScreen, VirtualScreenLayer, CHAR};
use crate::data_storage::screen::Coord;

pub static mut MAIN_SCREEN: Option<MainScreen> = None;

/// Height of the screen
const BUFFER_HEIGHT: usize = 25;

/// Width of the screen
const BUFFER_WIDTH: usize = 80;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct VirtualScreenID(u64);

impl VirtualScreenID {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let new = NEXT_ID.fetch_add(1, Ordering::Relaxed); // Maybe better to reallow previous numbers
        Self(new)
    }

    pub fn forge(id: usize) -> Self {
        Self(id as u64)
    }

    pub const fn null() -> Self {
        Self(0)
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}
impl Default for VirtualScreenID {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct MainScreen {
    /// Conversion id -> screen
    map: BTreeMap<VirtualScreenID, (usize, VirtualScreen)>,
    /// queue on id based on layer priority
    queue: PriorityQueue<VirtualScreenID, VirtualScreenLayer, DefaultHashBuilder>,
    /// back-up queue
    roll_queue: PriorityQueue<VirtualScreenID, VirtualScreenLayer, DefaultHashBuilder>,

    buffer: &'static mut [[CHAR; BUFFER_WIDTH]; BUFFER_HEIGHT],

    /// true if the case is occupied
    alpha: [[bool; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl MainScreen {
    pub fn new() -> Self {
        //let blank = CHAR::new(b' ', ColorCode(0_u8));
        Self {
            map: BTreeMap::new(),
            queue: PriorityQueue::with_default_hasher(),
            roll_queue: PriorityQueue::with_default_hasher(),
            buffer: unsafe {
                &mut *(crate::VGA_BUFFER as *mut [[CHAR; BUFFER_WIDTH]; BUFFER_HEIGHT])
            },
            alpha: [[false; BUFFER_WIDTH]; BUFFER_HEIGHT],
        }
    }
    pub fn get_vscreen_mut(&mut self, id: &VirtualScreenID) -> Option<&mut VirtualScreen> {
        if let Some((res0, _res1)) = self.queue.get(id) {
            match self.map.get_mut(&res0) {
                Some((_amount, vs)) => Some(vs),
                None => None,
            }
        } else {
            None
        }
    }
    pub fn resize_vscreen(&mut self, id: &VirtualScreenID, size: Coord) {
        crate::debug!("nb screen = {}", self.map.len());
        if let Some((res0, _res1)) = self.queue.get(id) {
            if let Some((_amount, vscreen)) = self.map.get_mut(&res0) {
                vscreen.resize(size);
            }
        }
    }
    pub fn replace_vscreen(&mut self, id: &VirtualScreenID, place: Coord) {
        if let Some((res0, _res1)) = self.queue.get(id) {
            if let Some((_amount, vscreen)) = self.map.get_mut(&res0) {
                vscreen.replace(place);
            }
        }
    }

    pub fn change_vscreen_layer(&mut self, id: &VirtualScreenID, layer: VirtualScreenLayer) {
        self.queue.change_priority(id, layer);
    }
    /// Draws the whole screen by displaying each v_screen ordered by layer
    ///
    /// A higher layer means the v_screen will be more on the foreground.
    pub fn draw(&mut self) {
        self.reset_alpha();
        while let Some((v_screen_id, layer)) = self.queue.pop() {
            if let Some((_amount, v_screen)) = self.map.get(&v_screen_id) {
                let position = v_screen.get_position();
                let size = v_screen.get_size();
                let row_origin = position.get_row();
                let col_origin = position.get_col();
                let row_size = size.get_row();
                let col_size = size.get_col();

                for i in 0..row_size {
                    for j in 0..col_size {
                        // The alpha layer helps ensuring we do not write to a previously
                        // written part of the screen (that is written from a v_screen
                        // with a higher layer). This is because we draw v_screens by order
                        // of decreasing layer.
                        if i + row_origin < BUFFER_HEIGHT
                            && j + col_origin < BUFFER_WIDTH
                            && self.alpha[i + row_origin][j + col_origin]
                        {
                            self.buffer[i + row_origin][j + col_origin] = v_screen.get_char(i, j);
                            self.alpha[i + row_origin][j + col_origin] = false;
                        }
                    }
                }
                self.roll_queue.push(v_screen_id, layer);
            }
        }
        self.spill_queue();
    }

    /// Puts all item in `roll_queue` back in the `queue`
    fn spill_queue(&mut self) {
        while let Some((v_screen_id, layer)) = self.roll_queue.pop() {
            self.queue.push(v_screen_id, layer);
        }
    }

    /// Resets the alpha layer of the screen
    fn reset_alpha(&mut self) {
        for i in 0..BUFFER_HEIGHT {
            for j in 0..BUFFER_WIDTH {
                self.alpha[i][j] = true;
            }
        }
    }

    pub fn new_screen(
        &mut self,
        row_top: usize,
        col_left: usize,
        height: usize,
        width: usize,
        layer: VirtualScreenLayer,
    ) -> VirtualScreenID {
        crate::debug!("New screen");
        let vs_id = VirtualScreenID::new();
        let screen = VirtualScreen::new(
            ColorCode(15),
            Coord::new(col_left, row_top),
            Coord::new(width, height),
            layer,
        );
        self.map.insert(vs_id, (1, screen));
        self.queue.push(vs_id, layer);
        vs_id
    }

    pub fn delete_screen(&mut self, vs_id: VirtualScreenID) -> bool {
        crate::debug!("deleted screen");
        if let Some((amount, mut screen)) = self.map.remove(&vs_id) {
            if amount == 1 {
                screen.delete();
            } else {
                self.map.insert(vs_id, (amount - 1, screen));
            }
        }
        false
    }

    pub fn duplicated(&mut self, vs_id: VirtualScreenID) {
        if let Some((amount, screen)) = self.map.remove(&vs_id) {
            self.map.insert(vs_id, (amount + 1, screen));
        }
    }

    /*pub fn get_screen(&mut self, id: &VirtualScreenID) -> Option<&'static mut VirtualScreen> {
        match self.map.get_mut(id) {
            Some((_amount, screen)) => Some(screen),
            None => None,
        }
    }*/
}

impl Default for MainScreen {
    fn default() -> Self {
        Self::new()
    }
}
