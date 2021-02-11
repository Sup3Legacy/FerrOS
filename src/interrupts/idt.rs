use x86_64::VirtAddr;
use x86_64::instructions::{segmentation, tables::{DescriptorTablePointer, lidt}};
use x86_64::structures::gdt::SegmentSelector;
use x86_64::PrivilegeLevel;
use core::mem::size_of;

pub struct Idt([Entry; 64]);

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

pub type HandlerFunc = extern "C" fn() -> !;

impl EntryOptions {

    fn minimal() -> Self {
        let mut options:u16 = 0;
        //options.set_bits(9..12, 0b111);
        options = options | 0b111_0000_0000;
        EntryOptions(options)
    }

    fn new() -> Self {
        let mut options = Self::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }

    pub fn set_present(&mut self, present : bool) -> &mut Self {
        if present {
            self.0 = self.0 | (1 << 15);
        } else {
            self.0 = self.0 & !(1 << 15);
        }
       // self.0.set(15, present);
        self
    }

    pub fn disable_interrupts(&mut self, disable : bool) -> &mut Self {
        //self.0.set(8, !disable);
        if disable {
            self.0 = self.0 & !(1 << 15);
        } else {
            self.0 = self.0 | (1 << 15);
        }
        self
    }

    pub fn set_privilege_level(&mut self, dpl : u16) -> &mut Self {
       // self.0.set_bits(13..15, dpl);
        self.0 = (self.0 & !(0b11 << 13))| ((dpl & 0b11) << 13);
        self
    }

    pub fn set_stack_index(&mut self, index : u16) -> &mut Self {
      //  self.0.set_bits(0..3, index);
      self.0 = (self.0 & !0b111) | (index & 0b111);
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
            options : EntryOptions::new(),
            reserved : 0,
        }
    }

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
    pub fn new() -> Self {
        Idt([Entry::missing(); 64])
    }

    pub fn set_handler_fn(&mut self, entry: u8, handler: HandlerFunc)
        -> &mut EntryOptions {
            self.0[entry as usize] = Entry::new(segmentation::cs(), handler);
            &mut self.0[entry as usize].options
        }
    
    pub fn load(&'static self) {
        let ptr = DescriptorTablePointer {
            base: VirtAddr::new(self as *const _ as u64),
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe { lidt(&ptr) };
    }
}