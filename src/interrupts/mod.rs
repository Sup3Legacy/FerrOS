//! Crate initialising every interrupts and putting it in the Interruption Descriptor Table

use x86_64::instructions::port::Port;

use x86_64::registers::control::{Cr2, Cr3};
use x86_64::structures::paging::PhysFrame;
use x86_64::PrivilegeLevel;
use x86_64::VirtAddr;

use crate::scheduler::QUANTUM;
use crate::hardware;

pub mod idt;
use idt::Idt as InterruptDescriptorTable;
use idt::{InterruptStackFrame, PageFaultErrorCode};

use crate::data_storage::registers::Registers;
use crate::gdt;
use crate::scheduler::process;
use crate::{print, println};
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;

mod syscalls;

static mut COUNTER: u64 = 0;

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

#[macro_export]
macro_rules! saveRegisters {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() {
            unsafe {
                asm!(
                "cli",
                "sub rsp, 32",
                "vmovapd [rsp], ymm0",
                "push r15",
                "push r14",
                "push r13",
                "push r12",
                "push r11",
                "push rbp",
                "push rcx",
                "push rbx",
                "push rax",
                "push rdi",
                "push rsi",
                "push rdx",
                "push r10",
                "push r8",
                "push r9",
                "mov rsi, rsp",
                "mov rdi, rsp",
                "add rdi, 15*8 + 32",
                "call {0}",
                "pop r9",
                "pop r8",
                "pop r10",
                "pop rdx",
                "pop rsi",
                "pop rdi",
                "pop rax",
                "pop rbx",
                "pop rcx",
                "pop rbp",
                "pop r11",
                "pop r12",
                "pop r13",
                "pop r14",
                "pop r15",
                "vmovapd ymm0, [rsp]",
                "add rsp, 32",
                "sti",
                "iretq",
                  sym $name
                );
            }
        }
        wrapper
    }};
}

lazy_static! {
    /// Defines the InterruptDescriptorTable and all the interruption handlers.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.debug.set_handler_fn(debug_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.non_maskable_interrupt
            .set_handler_fn(non_maskable_interrupt_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.breakpoint.set_handler_fn(breakpoint_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.overflow.set_handler_fn(overflow_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.bound_range_exceeded
            .set_handler_fn(bound_range_exceeded_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.device_not_available
            .set_handler_fn(device_not_available_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.page_fault.set_handler_fn(page_fault_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.x87_floating_point
            .set_handler_fn(x87_floating_point_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.alignment_check.set_handler_fn(alignment_check_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.simd_floating_point
            .set_handler_fn(simd_floating_point_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.virtualization.set_handler_fn(virtualization_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.security_exception
            .set_handler_fn(security_exception_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.timer.set_handler_fn(saveRegisters!(timer_interrupt_handler))
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.syscall.set_handler_fn(syscalls::naked_syscall_dispatch)
                    .set_privilege_level(PrivilegeLevel::Ring3);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

lazy_static! {
    static ref KEYBOARD: spin::Mutex<u8> = spin::Mutex::new(1);
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
    println!("div 0");
    panic!("DIVISION BY ZERO {:#?}", _stack_frame);
} // Rust catches this before the CPU, but it's a safeguard for asm/extern code.

// probably would not need to panic ?
// This interruption should pause the current process until the father restarts it
extern "x86-interrupt" fn debug_handler(_stack_frame: &mut InterruptStackFrame) {
    println!("debug");
    panic!("DEBUG");
}

extern "x86-interrupt" fn non_maskable_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    println!("non maskable");
    panic!("Non maskable Stack Frame");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("BREAKPOINT : {:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn overflow_handler(_stack_frame: &mut InterruptStackFrame) {
    println!("overflow");
    panic!("OVERFLOW");
}

extern "x86-interrupt" fn bound_range_exceeded_handler(_stack_frame: &mut InterruptStackFrame) {
    println!("bound range");
    panic!("BOUND RANGE EXCEEDED");
}

extern "x86-interrupt" fn invalid_opcode_handler(_stack_frame: &mut InterruptStackFrame) {
    println!("opcode");
    panic!("INVALID OPCODE");
}

extern "x86-interrupt" fn device_not_available_handler(_stack_frame: &mut InterruptStackFrame) {
    println!("device");
    panic!("DEVICE NOT AVAILABLE");
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) -> ! {
    println!("ERROR : {:#?}", error_code);
    println!("saved rsp : {:#?}", unsafe { process::CURRENT_PROCESS.rsp });
    panic!("EXCEPTION : DOUBLE FAULT : \n {:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_tss_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    println!("tss");
    panic!("INVALID TSS");
}

extern "x86-interrupt" fn segment_not_present_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    println!("segment {}", _error_code);
    println!("error : {}", _error_code);
    panic!("SEGMENT NOT PRESENT {}", _error_code);
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    println!("stack");
    panic!("STACK SEGMENT FAULT");
}

extern "x86-interrupt" fn general_protection_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    println!("Protection {}", _error_code);
    unsafe {
        let stack = _stack_frame.as_mut();
        println!("cs {} ss {}", stack.code_segment, stack.stack_segment);
        println!("ip : {}", stack.instruction_pointer.as_u64());
        println!("sp : {}", stack.stack_pointer.as_u64());
        //println!("cs2 {}", segmentation::cs().0);
        println!("GENERAL PROTECTION FAULT! {:#?}", stack);
    }
    println!("TRIED TO READ : {:#?}", Cr2::read());
    println!("ERROR : {:#?}", _error_code);
    loop {}
}

extern "x86-interrupt" fn x87_floating_point_handler(_stack_frame: &mut InterruptStackFrame) {
    panic!("x87 FLOATING POINT ERROR");
}

extern "x86-interrupt" fn alignment_check_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    println!("alignement");
    panic!("ALIGNMENT CHECK ERROR");
}

extern "x86-interrupt" fn simd_floating_point_handler(_stack_frame: &mut InterruptStackFrame) {
    println!("simd");
    panic!("SIMD FLOATING POINT ERROR");
}

extern "x86-interrupt" fn virtualization_handler(_stack_frame: &mut InterruptStackFrame) {
    println!("virtualization");
    panic!("VIRTUALIZATION ERROR");
}

extern "x86-interrupt" fn security_exception_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    println!("security");
    panic!("SECURITY EXCEPTION");
}

// Should be entirely rewritten for multi-process handling
unsafe extern "C" fn timer_interrupt_handler(
    stack_frame: &mut InterruptStackFrame,
    registers: &mut Registers,
) {
    print!(".");
    //println!("{:#?}", stack_frame);
    //println!("rax:{} rdi:{} rsi:{} r10:{}", registers.rax, registers.rdi, registers.rsi, registers.r10);
    //println!("r8:{} r9:{} r15:{} r14:{} r13:{}", registers.r8, registers.r9, registers.r15, registers.r14, registers.r13);
    //println!("r12:{} r11:{} rbp:{} rcx:{} rbx:{}", registers.r12, registers.r11, registers.rbp, registers.rcx, registers.rbx);
    if COUNTER == QUANTUM {
        COUNTER = 0;
        //println!("{:#?}", stack_frame);
        let _stack_frame_2 = stack_frame.as_mut();
        //println!("entered");
        let (next, mut old) = process::gives_switch(COUNTER);
        //println!("here");
        let (cr3, cr3f) = Cr3::read();
        old.cr3 = cr3.start_address();
        old.cr3f = cr3f;
        Cr3::write(PhysFrame::containing_address(next.cr3), next.cr3f);

        old.rsp = VirtAddr::from_ptr(registers).as_u64();

        println!("Tick");
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
        //print!("here {:X} stored {:X}\n", VirtAddr::from_ptr(registers).as_u64(), rsp_store);
        //println!("other data {:X}", VirtAddr::from_ptr(stack_frame).as_u64());
        process::leave_context(next.rsp);
        loop {}
        return;
    } else {
        COUNTER += 1;
    }

    PICS.lock()
        .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
}

/// Page fault handler, should verify wether killing the current process or allocating a new page !
extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: PageFaultErrorCode,
) {
    let read_addr = Cr2::read();
    if read_addr.as_u64() == 0 {
        println!("terminated normally");
        /* // This launch a new process when the other one has finished.
        unsafe {
            let (next, mut old) = process::terminated_normally(COUNTER);
            COUNTER = 0;
            let (cr3, cr3f) = CR"::read();
            old.cr3 = cr3.start_address();
            old.cr3f = cr3f;
            Cr3::write(PhysFrame::containing_address(next.cr3), next.cr3f);
            old.rsp = VirtAddr::from_ptr(registers).as_u64();
            process::leave_context(next.rsp);
        }
        */
        hardware::power::shutdown();
    } else {
        println!("PAGE FAULT! {:#?}", _stack_frame);
        println!("TRIED TO READ : {:#?}", Cr2::read());
        println!("ERROR : {:#?}", _error_code);
    }
    crate::halt_loop();
}

/// Keyboard interrupt handler
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let _keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

/// Start position for external interrupts (such as keyboard)
pub const PIC_1_OFFSET: u8 = 32;

/// Unused data
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Models the two-chips chained programmable interrupt controller of the 8259/AT PIC
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
