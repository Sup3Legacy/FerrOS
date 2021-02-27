use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::OffsetPageTable;
use x86_64::structures::paging::{FrameAllocator, /*Page, Mapper,*/ PhysFrame, Size4KiB};
use x86_64::{registers::control::Cr3, structures::paging::PageTable, PhysAddr, VirtAddr};
use crate::print;
// Memory address translation (virtual -> physical) now has to be done with `Translate::translate_addr`

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    //! Returns the virtual address of the level 4 page table, which is currently active (given by the CR3 register). The `physical_memory_offset` is needed as the model used is a `map_physical_memory` scheme.
    //! Is unsafe because the caller has to guarantee that the virtual memory is completely mapped as a physical_memory_offset.
    //! Should not be called more than once to avoid `&mut` aliasing.

    let (level_4_frame, _) = Cr3::read();
    let phys = level_4_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr
}

/// Returns a new `OffsetPageTable`.
/// It is based on the 4-level active table.
/// It is unsafe as the complete mapping has to be guaranteed by the caller.
/// Must be called at least once to avoid `&mut` aliasing
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

/// Create a FrameAllocator from the passed memory map.
///
/// This function is unsafe because the caller must guarantee that the passed
/// memory map is valid. The main requirement is that all frames that are marked
/// as `USABLE` in it are really unused.
impl BootInfoAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoAllocator {
            memory_map,
            next: 0,
        }
    }
    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

// Not needed as implemented in x86_64 crate ; keep it for later reference
/*
pub unsafe fn translate_addr(addr : VirtAddr, physical_memory_offset : VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(addr : VirtAddr, physical_memory_offset : VirtAddr) -> Option<PhysAddr> {
    let (level_4_table_frame, _) = Cr3::read();
    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    for &index in &table_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr : *const PageTable = virt.as_ptr();
        let table = unsafe {&*table_ptr};

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Huge frames not supported..."),
        };
    }
    Some(frame.start_address() + u64::from(addr.page_offset()))
}
*/
