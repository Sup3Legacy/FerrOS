use core::mem::size_of;
use x86_64::instructions::{segmentation, 
    tables::{lidt, DescriptorTablePointer},
};
//use x86_64::structures::gdt::SegmentSelector;
use x86_64::{PrivilegeLevel, VirtAddr};
use core::marker::PhantomData;
use core::ops::{ Index, IndexMut};
use core::fmt;
use bitflags::bitflags;
use bit_field::BitField;

const SYSCALL_POSITION: usize = 0x80;
const SYSCALL_POSITION_1: usize = 0x7E;
const SYSCALL_POSITION_2: usize = 0x81;

#[repr(C)]
#[repr(align(16))]
pub struct Idt {
    pub divide_error: Entry<HandlerFunc>,
    pub debug: Entry<HandlerFunc>,
    pub non_maskable_interrupt: Entry<HandlerFunc>,
    pub breakpoint: Entry<HandlerFunc>,
    pub overflow: Entry<HandlerFunc>,
    pub bound_range_exceeded: Entry<HandlerFunc>,
    pub invalid_opcode: Entry<HandlerFunc>,
    pub device_not_available: Entry<HandlerFunc>,
    pub double_fault: Entry<DivergingFuncWithErrorCode>,
    interrupt_09: Entry<HandlerFunc>,
    pub invalid_tss: Entry<HandlerFuncWithErrorCode>,
    pub segment_not_present: Entry<HandlerFuncWithErrorCode>,
    pub stack_segment_fault: Entry<HandlerFuncWithErrorCode>,
    pub general_protection_fault: Entry<HandlerFuncWithErrorCode>,
    pub page_fault: Entry<PageFaultHandler>,
    interrupt_15: Entry<HandlerFunc>, // reserved
    pub x87_floating_point: Entry<HandlerFunc>,
    pub alignment_check: Entry<HandlerFuncWithErrorCode>,
    pub machine_check: Entry<DivergingFunc>,
    pub simd_floating_point: Entry<HandlerFunc>,
    pub virtualization: Entry<HandlerFunc>,
    reserved_21_29: [Entry<HandlerFunc>; 9], // reserved
    pub security_exception: Entry<HandlerFuncWithErrorCode>,
    interrupt_31: Entry<HandlerFunc>, // reserved
    pub interrupt_32_: [Entry<HandlerFunc>; SYSCALL_POSITION-32],
    pub syscall: Entry<SyscallFunc>,
    pub interrupt_post_syscall_: [Entry<HandlerFunc>; 255-SYSCALL_POSITION],
}

impl Idt {
    pub fn new() -> Self {
        Idt {
            divide_error: Entry::missing(),
            debug: Entry::missing(),
            non_maskable_interrupt: Entry::missing(),
            breakpoint: Entry::missing(),
            overflow: Entry::missing(),
            bound_range_exceeded: Entry::missing(),
            invalid_opcode: Entry::missing(),
            device_not_available: Entry::missing(),
            double_fault: Entry::missing(),
            interrupt_09: Entry::missing(),
            invalid_tss: Entry::missing(),
            segment_not_present: Entry::missing(),
            stack_segment_fault: Entry::missing(),
            general_protection_fault: Entry::missing(),
            page_fault: Entry::missing(),
            interrupt_15: Entry::missing(),
            x87_floating_point: Entry::missing(),
            alignment_check: Entry::missing(),
            machine_check: Entry::missing(),
            simd_floating_point: Entry::missing(),
            virtualization: Entry::missing(),
            reserved_21_29: [Entry::missing(); 9],
            security_exception: Entry::missing(),
            interrupt_31: Entry::missing(),
            interrupt_32_: [Entry::missing(); SYSCALL_POSITION-32],
            syscall: Entry::missing(),
            interrupt_post_syscall_: [Entry::missing(); 255-SYSCALL_POSITION],
        }
    }

    pub fn load(&'static self) {
        let ptr = DescriptorTablePointer {
            base: VirtAddr::new(self as *const _ as u64),
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe { lidt(&ptr) };
    }

}

impl Index<usize> for Idt {
    type Output = Entry<HandlerFunc>;

    #[inline]
    fn index(&self, position: usize) -> &Self::Output {
        match position {
            0 => &self.divide_error,
            1 => &self.debug,
            2 => &self.non_maskable_interrupt,
            3 => &self.breakpoint,
            4 => &self.overflow,
            5 => &self.bound_range_exceeded,
            6 => &self.invalid_opcode,
            7 => &self.device_not_available,
            8 => panic!("this function should be diverging"),
            9 => panic!("access not allowed! It is reserved"),
            _i @ 10..=14 => panic!("wrong function type"),
            15 => panic!("access not allowed! It is reserved"),
            16 => & self.x87_floating_point,
            17 => panic!("wrong function type"),
            18 => panic!("this function should be diverging"),
            19 => &self.simd_floating_point,
            20 => &self.virtualization,
            _i @ 21..=29 => panic!("access not allowed! It is reserved"),
            30 => panic!("wrong function type"),
            31 => panic!("access not allowed! It is reserved"),
            i @ 32..=SYSCALL_POSITION_1 => &self.interrupt_32_[i - 32],
            SYSCALL_POSITION => panic!("wrong function type"),
            i @ SYSCALL_POSITION_2..=255 => &self.interrupt_post_syscall_[i - SYSCALL_POSITION-1],
            _i => panic!("no such entry")
        }
    }
}

impl IndexMut<usize> for Idt {

    #[inline]
    fn index_mut(&mut self, position: usize) -> &mut Self::Output {
        match position {
            0 => &mut self.divide_error,
            1 => &mut self.debug,
            2 => &mut self.non_maskable_interrupt,
            3 => &mut self.breakpoint,
            4 => &mut self.overflow,
            5 => &mut self.bound_range_exceeded,
            6 => &mut self.invalid_opcode,
            7 => &mut self.device_not_available,
            8 => panic!("this function should be diverging"),
            9 => panic!("access not allowed! It is reserved"),
            _i @ 10..=14 => panic!("wrong function type"),
            15 => panic!("access not allowed! It is reserved"),
            16 => &mut self.x87_floating_point,
            17 => panic!("wrong function type"),
            18 => panic!("this function should be diverging"),
            19 => &mut self.simd_floating_point,
            20 => &mut self.virtualization,
            _i @ 21..=29 => panic!("access not allowed! It is reserved"),
            30 => panic!("wrong function type"),
            31 => panic!("access not allowed! It is reserved"),
            i @ 32..=SYSCALL_POSITION_1 => &mut self.interrupt_32_[i - 32],
            SYSCALL_POSITION => panic!("wrong function type"),
            i @ SYSCALL_POSITION_2..=255 => &mut self.interrupt_post_syscall_[i - SYSCALL_POSITION-1],
            _i => panic!("no such entry")
        }
    }
}

pub type HandlerFunc = extern "x86-interrupt" fn(&mut InterruptStackFrame);
pub type HandlerFuncWithErrorCode = extern "x86-interrupt" fn(&mut InterruptStackFrame, error_code: u64);
pub type PageFaultHandler = extern "x86-interrupt" fn(&mut InterruptStackFrame, PageFaultErrorCode);
pub type DivergingFunc = extern "x86-interrupt" fn(&mut InterruptStackFrame) -> !;
pub type DivergingFuncWithErrorCode = extern "x86-interrupt" fn(&mut InterruptStackFrame, error_code: u64) -> !;
pub type SyscallFunc = extern "C" fn();

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct EntryOptions(u16);

impl EntryOptions {

    #[inline]
    fn minimal() -> Self {
        //options.set_bits(9..12, 0b111);
        EntryOptions(0b1110_0000_0000)
    }

    fn new() -> Self {
        let mut options = Self::minimal();
        options.set_present(true);//.disable_interrupts(true);
        options
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        if present {
            self.0 = self.0 | (1 << 15);
        } else {
            self.0 = self.0 & !(1 << 15);
        }
        //self.0.set(15, present);
        self
    }

    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.0.set_bit(8, !disable);
        self
    }

    pub fn set_privilege_level(&mut self, dpl: PrivilegeLevel) -> &mut Self {
        self.0.set_bits(13..15, dpl as u16);
        self
    }

    pub unsafe fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0.set_bits(0..3, index + 1);
        self
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Entry<FunctionType> {
    pointer_low: u16,
    gdt_selector: u16,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
    phantom: PhantomData<FunctionType>,
}



impl<FunctionType> Entry<FunctionType> {
    fn missing() -> Self {
        Entry {
            gdt_selector: 0,
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
            phantom: PhantomData,
        }
    }
}

impl Entry<HandlerFunc> {
    pub fn set_handler_fn(&mut self, handler: HandlerFunc) -> &mut EntryOptions {
        let handler = handler as u64;
        self.pointer_low = handler as u16;
        self.pointer_middle = (handler >> 16) as u16;
        self.pointer_high = (handler >> 32) as u32;
        self.gdt_selector = segmentation::cs().0;
        self.options.set_present(true);
        &mut self.options
    }
}

impl Entry<HandlerFuncWithErrorCode> {
    pub fn set_handler_fn(&mut self, handler: HandlerFuncWithErrorCode) -> &mut EntryOptions {
        let handler = handler as u64;
        self.pointer_low = handler as u16;
        self.pointer_middle = (handler >> 16) as u16;
        self.pointer_high = (handler >> 32) as u32;
        self.gdt_selector = segmentation::cs().0;
        self.options.set_present(true);
        &mut self.options
    }
}

impl Entry<DivergingFunc> {
    pub fn set_handler_fn(&mut self, handler: DivergingFunc) -> &mut EntryOptions {
        let handler = handler as u64;
        self.pointer_low = handler as u16;
        self.pointer_middle = (handler >> 16) as u16;
        self.pointer_high = (handler >> 32) as u32;
        self.gdt_selector = segmentation::cs().0;
        self.options.set_present(true);
        &mut self.options
    }
}

impl Entry<DivergingFuncWithErrorCode> {
    pub fn set_handler_fn(&mut self, handler: DivergingFuncWithErrorCode) -> &mut EntryOptions {
        let handler = handler as u64;
        self.pointer_low = handler as u16;
        self.pointer_middle = (handler >> 16) as u16;
        self.pointer_high = (handler >> 32) as u32;
        self.gdt_selector = segmentation::cs().0;
        self.options.set_present(true);
        &mut self.options
    }
}

impl Entry<PageFaultHandler> {
    pub fn set_handler_fn(&mut self, handler: PageFaultHandler) -> &mut EntryOptions {
        let handler = handler as u64;
        self.pointer_low = handler as u16;
        self.pointer_middle = (handler >> 16) as u16;
        self.pointer_high = (handler >> 32) as u32;
        self.gdt_selector = segmentation::cs().0;
        self.options.set_present(true);
        &mut self.options
    }
}

impl Entry<SyscallFunc> {
    pub fn set_handler_fn(&mut self, handler: SyscallFunc) -> &mut EntryOptions {
        let handler = handler as u64;
        self.pointer_low = handler as u16;
        self.pointer_middle = (handler >> 16) as u16;
        self.pointer_high = (handler >> 32) as u32;
        self.gdt_selector = segmentation::cs().0;
        self.options.set_present(true);
        &mut self.options
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1;
        const CAUSED_BY_WRITE = 2;
        const USER_MODE = 4;
        const MALFORMED_TABLE = 8;
        const INSTRUCTION_FETCH = 16;
    }
}

#[repr(C)]
pub struct InterruptStackFrame {
    value: InterruptStackFrameValue,
}

impl InterruptStackFrame {
    pub unsafe fn as_mut(&mut self) -> &mut InterruptStackFrameValue {
        &mut self.value
    }
}
impl fmt::Debug for InterruptStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct InterruptStackFrameValue {
    pub instruction_pointer: VirtAddr,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: VirtAddr,
    pub stack_segment: u64,
}

impl fmt::Debug for InterruptStackFrameValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("InterruptStackFrameValue");
        s.field("instruction_pointer", &self.instruction_pointer);
        s.field("code_segment", &self.code_segment);
        s.field("cpu_flags", &self.cpu_flags);
        s.field("stack_pointer", &self.stack_pointer);
        s.field("stack_segment", &self.stack_segment);
        s.finish()
    }
}