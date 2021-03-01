use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::OffsetPageTable;
use x86_64::structures::paging::{FrameAllocator, /*Page, Mapper,*/ PhysFrame, Size4KiB};
use x86_64::{registers::control::Cr3, structures::paging::{PageTable, PageTableFlags}, PhysAddr, VirtAddr};
use crate::print;
use core::cmp::{max, min};
use lazy_static::lazy_static;
// Memory address translation (virtual -> physical) now has to be done with `Translate::translate_addr`
pub static mut PHYSICAL_OFFSET: u64 = 0;

const MAX_PAGE_ALLOWED: usize = 65536;

static mut NUMBER_TABLES: u64 = 0;

static mut PAGE_AVAILABLE:[bool; MAX_PAGE_ALLOWED] = [false; MAX_PAGE_ALLOWED];


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
    PHYSICAL_OFFSET = physical_memory_offset.as_u64();
    let level_4_table = active_level_4_table(physical_memory_offset);
    let mut compte = 0;
    for i in 0..512 {
        if level_4_table[i].is_unused() {
            compte += 1
        } else {
            let addr = level_4_table[i].addr();
            let virt = physical_memory_offset + addr.as_u64();
            let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
            let level_3_table = &mut * page_table_ptr;
            for i2 in 0..512 {
                if !level_3_table[i2].is_unused() {
                    print!("{} at {} with {:?}\n", i2, i, level_3_table[i].flags());
                    //if 
                }
            }
       }
    }
    print!("Nb Frame used : {}.\n", compte);
    print!("Phys_offset : {:?}", physical_memory_offset);
    //loop {}
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub struct BootInfoAllocator {
    pages_available: [bool; MAX_PAGE_ALLOWED],
    next: usize,
    maxi: usize,
    level4_table: &'static PageTable,
}

impl BootInfoAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap, physical_memory_offset: VirtAddr) -> Self {
        let mut pages_available = [false; MAX_PAGE_ALLOWED];
        let regions = memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        let mut maxi = 0;
        let mut next = 0;
        unsafe {
            for i in frame_addresses {
                NUMBER_TABLES += 1;
                pages_available[(i >> 12) as usize] = true;
                maxi = max(i >> 12, maxi);
                next = min(i >> 12, next);
            };
            print!("Num tables : {}\n", NUMBER_TABLES);
        }

        BootInfoAllocator {
            pages_available,
            next: next as usize,
            maxi: maxi as usize,
            level4_table: active_level_4_table(physical_memory_offset),
        }
    }

    pub fn allocate_4k_frame(&mut self) -> Option<PhysFrame> {
        for _i in 0..MAX_PAGE_ALLOWED {
            if self.pages_available[self.next] {
                self.pages_available[self.next] = false;
                self.next += 1;
                if let Ok(frame) = PhysFrame::from_start_address(PhysAddr::new(((self.next as u64) - 1) << 12)) {
                    return Some(frame)
                } else {
                    return None
                }
            } else {
                self.next += 1;
                if self.next > self.maxi {
                    self.next = 0;
                }
            }
        }
        None
    }

    pub unsafe fn allocate_level_4_frame(&mut self) -> Result<&'static mut PageTable,()> {
        if let Some(frame) = self.allocate_4k_frame() {
            let phys = frame.start_address();
            let virt = VirtAddr::new(phys.as_u64() + PHYSICAL_OFFSET);
            let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
            let level_4_table = &mut *page_table_ptr;
            for i in 0..512 {   
                level_4_table[i].set_addr(self.level4_table[i].addr(), self.level4_table[i].flags());
            }
            Ok(level_4_table)
        } else {
            Err(())
        }
    } 

    pub unsafe fn allocate_table_at_address(&mut self, table_4: &'static mut PageTable, addr: VirtAddr) -> Result<(),()> {
        panic!("not implemented");
    }

    pub fn deallocate_level_4_page(&mut self, table_4: PageTable) -> Result<(),()> {
        panic!("not implemented");
    }

    pub fn deallocate_4k_frame(&mut self, next: PhysFrame) -> Result<(),()> {
        let next = next.start_address().as_u64();
        self.pages_available[next as usize] = false;
        return Ok(())
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoAllocator {
    #[inline]
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.allocate_4k_frame()
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
}*/