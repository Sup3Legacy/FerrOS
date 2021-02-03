
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use pic8259_simple::ChainedPics;
use spin;
use lazy_static::lazy_static;
use crate::{println, print};

#[derive(Clone, Debug, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT : InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
    unsafe {
        PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame : &mut InterruptStackFrame) {
    println!("BREAKPOINT : {:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame : &mut InterruptStackFrame, error_code : u64) -> ! {
        println!("ERROR : {:#?}", error_code);
        panic!("EXCEPTION : DOUBLE FAULT : \n {:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame : &mut InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });