//! Everything needed to setup a GDT that does nothing, so we can use paging instead.

use lazy_static::lazy_static;
use x86_64::instructions::segmentation::set_cs;
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

mod gdt_entry;

/// Index of the stack for double fault handling
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;


/// Structure to store every selector/segment to modify them at will afterward.
pub struct Selectors {
    code_selector: SegmentSelector, // kernel selector
    tss_selector: SegmentSelector, // TSS selector
    code_segment: SegmentSelector, // user code segment selector
    data_segment: SegmentSelector, // user data segment selector
}

lazy_static! {
    /// Defines the InterruptDescriptorTable and all the interruption handlers.
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        // create a new Global Descriptor Table
        let mut gdt = GlobalDescriptorTable::new();

        // Add the kernel code space and data space in the gdt
        let code_selector = gdt.add_entry(Descriptor::SystemSegment(
            gdt_entry::kernel_cs(),
            gdt_entry::kernel_ds(),
        ));

        // Add the TSS in the Global descriptor table
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        // Add the segments for user space in the table.
        let code_segment = gdt.add_entry(Descriptor::UserSegment(gdt_entry::new_cs()));
        let data_segment = gdt.add_entry(Descriptor::UserSegment(gdt_entry::new_ds()));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
                code_segment,
                data_segment,
            },
        )
    };
}

lazy_static! {
    /// Defines the InterruptDescriptorTable and all the interruption handlers
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // Should be improved by using the frame allocator to have a minimal protection against memory corruption and stack overflow
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

/// Initializes the GDT to be useless, and start the kernel.
pub fn init() {
    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
