use x86_64::instructions::port::Port;
use x86_64::registers::control::{Cr2, Cr3};
mod idt;
use idt::Idt as InterruptDescriptorTable;
use idt::{InterruptStackFrame, PageFaultErrorCode};
//use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
//use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyboardLayout, KeyCode, Modifiers};
use crate::gdt;
use crate::{print, println};
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;

mod syscalls;
//use crate::keyboard_layout;

#[derive(Clone, Debug, Copy)]
#[repr(u8)]
/// Representation of the interupts that need explicit mapping to the IDT
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
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
    /// Defines the InterruptDescriptorTable and all the interruption handlers.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt
            .set_handler_fn(non_maskable_interrupt_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded
            .set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available
            .set_handler_fn(device_not_available_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point
            .set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.simd_floating_point
            .set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.security_exception
            .set_handler_fn(security_exception_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt.syscall.set_handler_fn(syscalls::naked_syscall_dispatch);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

lazy_static! {
    static ref KEYBOARD : spin::Mutex<u8> = spin::Mutex::new(1);
   /* static ref KEYBOARD : spin::Mutex<Keyboard<Fr104Key, ScancodeSet1>> =
    spin::Mutex::new(
        Keyboard::new(
            Fr104Key, ScancodeSet1, HandleControl::Ignore
        )
    );*/
}

/// Loads the IDT into the kernel, starts the PIC and listens to the interruptions.
pub fn init() {
    IDT.load();
    unsafe {
        PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn divide_error_handler(_stack_frame: &mut InterruptStackFrame) {
    panic!("DIVISION BY ZERO");
} // Rust catches this before the CPU, but it's a safeguard for asm/extern code.

// probably would not need to panic ?
extern "x86-interrupt" fn debug_handler(_stack_frame: &mut InterruptStackFrame) {
    panic!("DEBUG");
}

extern "x86-interrupt" fn non_maskable_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    panic!("Non maskable Stack Frame");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("BREAKPOINT : {:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(_stack_frame: &mut InterruptStackFrame) {
    panic!("OVERFLOW");
}

extern "x86-interrupt" fn bound_range_exceeded_handler(_stack_frame: &mut InterruptStackFrame) {
    panic!("BOUND RANGE EXCEEDED");
}

extern "x86-interrupt" fn invalid_opcode_handler(_stack_frame: &mut InterruptStackFrame) {
    panic!("INVALID OPCODE");
}

extern "x86-interrupt" fn device_not_available_handler(_stack_frame: &mut InterruptStackFrame) {
    panic!("DEVICE NOT AVAILABLE");
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) -> ! {
    println!("ERROR : {:#?}", error_code);
    panic!("EXCEPTION : DOUBLE FAULT : \n {:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_tss_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    panic!("INVALID TSS");
}

extern "x86-interrupt" fn segment_not_present_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    panic!("SEGMENT NOT PRESENT");
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    panic!("STACK SEGMENT FAULT");
}

extern "x86-interrupt" fn general_protection_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    panic!("GENERAL PROTECTION FAULT");
}

extern "x86-interrupt" fn x87_floating_point_handler(_stack_frame: &mut InterruptStackFrame) {
    panic!("x87 FLOATING POINT ERROR");
}

extern "x86-interrupt" fn alignment_check_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    panic!("ALIGNMENT CHECK ERROR");
}

extern "x86-interrupt" fn simd_floating_point_handler(_stack_frame: &mut InterruptStackFrame) {
    panic!("SIMD FLOATING POINT ERROR");
}

extern "x86-interrupt" fn virtualization_handler(_stack_frame: &mut InterruptStackFrame) {
    panic!("VIRTUALIZATION ERROR");
}

extern "x86-interrupt" fn security_exception_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    panic!("SECURITY EXCEPTION");
}

extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: &mut InterruptStackFrame) {
    //print!(".");
    //println!("Timer {:#?}", _stack_frame);
    let stack_frame2 = unsafe { stack_frame.as_mut() };
    let (pf, cr_f) = Cr3::read();
    let state = crate::task::executor::Status {
        cs: stack_frame2.code_segment,
        cf: stack_frame2.cpu_flags,
        sp: stack_frame2.stack_pointer,
        ss: stack_frame2.stack_segment,
        ip: stack_frame2.instruction_pointer,
        cr3: (pf, cr_f),
    };
    let next = crate::task::executor::next_task(state);
    stack_frame2.code_segment = next.cs;
    stack_frame2.cpu_flags = next.cf;
    stack_frame2.stack_pointer = next.sp;
    stack_frame2.stack_segment = next.ss;
    stack_frame2.instruction_pointer = next.ip;
    unsafe {
        Cr3::write(next.cr3.0, next.cr3.1);
    };
    /*
    println!("test1");
    stack_frame2.instruction_pointer = VirtAddr::new(8); // cette ligne fait tout planter
    println!("test2");*/
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: PageFaultErrorCode,
) {
    println!("PAGE FAULT! {:#?}", _stack_frame);
    println!("TRIED TO READ : {:#?}", Cr2::read());
    println!("ERROR : {:#?}", _error_code);
    crate::halt_loop();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let _keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::keyboard::add_scancode(scancode);

    /* Character printing : useful for keyboard layout debug
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }
    */

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Models the two-chips chained programmable interrupt controller of the 8259/AT PIC
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
