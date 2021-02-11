mod idt;

use x86_64::addr::VirtAddr;
//use x86_64::addr::VirtAddr;
use x86_64::instructions::port::Port;
use x86_64::registers::control::{Cr2, Cr3};
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
//use x86_64::structures::idt::InterruptDescriptorTable;
//use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyboardLayout, KeyCode, Modifiers};
use crate::gdt;
use crate::{print, println};
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;

#[macro_export]
macro_rules! handler {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!(
                    "push rdi",
                    "mov rdi, rsp",
                    "add rdi, 8",
                    "push rsi",
                    "push rax",
                    "push rcx",
                    "push rdx",
                    "push r8",
                    "push r9",
                    "push r10",
                    "push r11",
                    "sub rsp, 8", // align the stack pointer
                      "call {0}",
                    "add rsp, 8",
                    "pop r11",
                    "pop r10",
                    "pop r9",
                    "pop r8",
                    "pop rdx",
                    "pop rcx",
                    "pop rax",
                    "pop rsi",
                    "pop rdi",
                    "iretq",
                      sym $name
                    );
                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    }};
}

#[macro_export]
macro_rules! handler_with_error_code {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!(
                    "push rdi",
                    "mov rdi, rsp",
                    "add rdi, 16",
                    "push rsi",
                    "mov rsi, [rsp + 16]",
                    "push rax",
                    "push rcx",
                    "push rdx",
                    "push r8",
                    "push r9",
                    "push r10",
                    "push r11",
                    "sub rsp, 8", // align the stack pointer
                      "call {0}",
                    "add rsp, 8",
                    "pop r11",
                    "pop r10",
                    "pop r9",
                    "pop r8",
                    "pop rdx",
                    "pop rcx",
                    "pop rax",
                    "pop rsi",
                    "pop rdi",
                    "iretq",
                    sym $name
                    );
                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    }};
}

#[derive(Clone, Debug)]
#[repr(C)]
pub enum InterruptIndex {
    Timer = 32
}

impl InterruptIndex {
    fn as_usize(self) -> usize {
        self as usize
    }
}

lazy_static! {
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler_fn(0, handler!(divide_by_zero_handler));
        idt.set_handler_fn(1, handler!(debug_handler));
        idt.set_handler_fn(2, handler!(non_maskable_interrupt_handler));
        idt.set_handler_fn(3, handler!(breakpoint_handler));
        idt.set_handler_fn(4, handler!(overflow_handler));
        idt.set_handler_fn(5, handler!(bound_range_exceeded_handler));
        idt.set_handler_fn(6, handler!(invalid_opcode_handler));
        idt.set_handler_fn(7, handler!(device_not_available_handler));
        idt.set_handler_fn(10, handler_with_error_code!(invalid_tss_handler));
        idt.set_handler_fn(11, handler_with_error_code!(segment_not_present_handler));
        idt.set_handler_fn(12, handler_with_error_code!(stack_segment_fault_handler));
        idt.set_handler_fn(
            13,
            handler_with_error_code!(general_protection_fault_handler),
        );
        idt.set_handler_fn(14, handler_with_error_code!(page_fault_handler));
        idt.set_handler_fn(16, handler!(x87_floating_point_handler));
        idt.set_handler_fn(17, handler_with_error_code!(alignment_check_handler));
        idt.set_handler_fn(19, handler!(simd_floating_point_handler));
        idt.set_handler_fn(20, handler!(virtualization_handler));
        idt.set_handler_fn(30, handler_with_error_code!(security_exception_handler));
        idt.set_handler_fn(InterruptIndex::Timer.as_usize() as u8, handler!(timer_interrupt_handler));
        //idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        unsafe {
            idt.set_handler_fn(8, handler_with_error_code!(double_fault_handler))
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

pub fn init() {
    IDT.load();
    unsafe {
        PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
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

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

extern "C" fn divide_by_zero_handler(stack_frame: &ExceptionStackFrame) {
    println!("\nEXCEPTION: DIVIDE BY ZERO{:#?}", unsafe { &*stack_frame });
    loop {}
}

extern "C" fn debug_handler(_stack_frame: &InterruptStackFrame) {
    panic!("DEBUG");
}

extern "C" fn non_maskable_interrupt_handler(_stack_frame: &InterruptStackFrame) {
    panic!("Non maskable Stack Frame");
}

extern "C" fn breakpoint_handler(stack_frame: &InterruptStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    println!("BREAKPOINT : {:#?}", stack_frame.instruction_pointer);
}

extern "C" fn overflow_handler(_stack_frame: &InterruptStackFrame) {
    panic!("OVERFLOW");
}

extern "C" fn bound_range_exceeded_handler(_stack_frame: &InterruptStackFrame) {
    panic!("BOUND RANGE EXCEEDED");
}

extern "C" fn invalid_opcode_handler(_stack_frame: &InterruptStackFrame) {
    panic!("INVALID OPCODE");
}

extern "C" fn device_not_available_handler(_stack_frame: &InterruptStackFrame) {
    panic!("DEVICE NOT AVAILABLE");
}

extern "C" fn double_fault_handler(stack_frame: &InterruptStackFrame, error_code: u64) {
    println!("ERROR : {:#?}", error_code);
    loop {};
    panic!("EXCEPTION : DOUBLE FAULT : \n {:#?}", stack_frame);
}

extern "C" fn invalid_tss_handler(_stack_frame: &InterruptStackFrame, _error_code: u64) {
    panic!("INVALID TSS");
}

extern "C" fn segment_not_present_handler(_stack_frame: &InterruptStackFrame, _error_code: u64) {
    panic!("SEGMENT NOT PRESENT");
}

extern "C" fn stack_segment_fault_handler(_stack_frame: &InterruptStackFrame, _error_code: u64) {
    panic!("STACK SEGMENT FAULT");
}

extern "C" fn general_protection_fault_handler(
    _stack_frame: &InterruptStackFrame,
    _error_code: u64,
) {
    panic!("GENERAL PROTECTION FAULT");
}

extern "C" fn x87_floating_point_handler(_stack_frame: &InterruptStackFrame) {
    panic!("x87 FLOATING POINT");
}

extern "C" fn alignment_check_handler(_stack_frame: &InterruptStackFrame, _error_code: u64) {
    panic!("ALIGNMENT CHECK");
}

extern "C" fn simd_floating_point_handler(_stack_frame: &InterruptStackFrame) {
    panic!("SIMD FLOATING POINT");
}

extern "C" fn virtualization_handler(_stack_frame: &InterruptStackFrame) {
    panic!("VIRTUALIZATION");
}

extern "C" fn security_exception_handler(_stack_frame: &InterruptStackFrame, _error_code: u64) {
    panic!("SECURITY EXCEPTION");
}

extern "C" fn timer_interrupt_handler(stack_frame: &mut InterruptStackFrame) {
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
        PICS.lock().notify_end_of_interrupt(0 as u8);
    }
}

extern "C" fn page_fault_handler(stack_frame: &InterruptStackFrame, error_code: u64) -> ! {
    println!(
        "\nEXCEPTION: PAGE FAULT while accessing ?\
        \nerror code: {:?}\n{:#?}",
        //unsafe { Cr2::read() },
        error_code, // PageFaultErrorCode::from_bits(error_code).unwrap(),
        unsafe { &*stack_frame }
    );
    loop {}
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let _keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock().notify_end_of_interrupt(1 as u8);
    };
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
