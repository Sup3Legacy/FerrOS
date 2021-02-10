use x86_64::instructions::{segmentation, tables::{DescriptorTablePointer, lidt}};
use x86_64::structures::gdt::SegmentSelector;
use x86_64::PrivilegeLevel;
use core::mem::size_of;

pub struct Idt([Entry; 16]);

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Entry {
    pointer_low: u16,
    gdt_selector: SegmentSelector,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct EntryOptions(u16);

impl EntryOptions {

    fn minimal() -> Self {
        let mut options = 0;
        options.set_bits(9..12, 0b111);
        EntryOptions(options)
    }

    fn new() -> Self {
        let mut options = Self::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }

    pub fn set_present(&mut self, present : bool) -> &mut Self {
        self.0.set_bit(15, present);
        self
    }

    pub fn disable_interrupts(&mut self, disable : bool) -> &mut Self {
        self.0.set_bit(8, !disable);
    }

    pub fn set_privilege_level(&mut self, dpl : u16) -> &mut Self {
        self.0.set_bits(13..15, dpl);
        self
    }

    pub fn set_stack_index(&mut self, index : u16) -> &mut Self {
        self.0.set_bits(0..3, index);
        self
    }
}

impl Entry {
    fn new(gdt_selector : SegmentSelector, handler : HandlerFunc) -> Self {
        let pointer = handler as u64;
        Entry {
            gdt_selector,
            pointer_low : pointer as u16,
            pointer_middle: (pointer >> 16) as u16,
            pointer_high: (pointer >> 32) as u32,
            options : EnteryOptions::new(),
            reserved : 0,
        }
    }
}

pub type HandlerFunc = extern "C" fn() -> !;

impl  Idt {
    pub fn new() -> Idt {
        Idt([Entry::missing(); 16])
    }
}

impl Entry {
    fn missing() -> Self {
        Entry {
            gdt_selector: SegmentSelector::new(0, PrivilegeLevel::Ring0),
            pointer_low : 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
        }
    }
}

impl Idt {
    pub fn set_handler_fn(&mut self, entry: u8, handler: HandlerFunc)
        -> &mut EntryOptions {
            self.0[entry as usize] = Entry::new(segmentation::cs(), handler);
            &mut self.0[entry as usize].options
        }
    
    pub fn load(&'static self) {
        let ptr = DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe { lidt(&ptr) };
    }
}