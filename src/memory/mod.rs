//! Crate for managing the paging: allocating and desallocating pages and editing page tables
use crate::{print, println};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use core::cmp::{max, min};
use x86_64::structures::paging::OffsetPageTable;
use x86_64::structures::paging::{FrameAllocator, /*Page, Mapper,*/ PhysFrame, Size4KiB};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{PageTable, PageTableFlags},
    PhysAddr, VirtAddr,
};
//use core::ptr;

//use lazy_static::lazy_static;

pub static mut FRAME_ALLOCATOR: Option<BootInfoAllocator> = None;

pub struct MemoryError();

use crate::warningln;

/// Memory address translation (virtual -> physical) now has to be done with `Translate::translate_addr`
pub static mut PHYSICAL_OFFSET: u64 = 0;

/// Maximum number of pages allowed. Can hold 256MiB of RAM, must be increased to have a higher capacity
const MAX_PAGE_ALLOWED: usize = 65536;

/// Number of allocatable tables
static mut NUMBER_TABLES: u64 = 0;

/// Structure of all available pages
//static mut PAGE_AVAILABLE: [bool; MAX_PAGE_ALLOWED] = [false; MAX_PAGE_ALLOWED];

/// Read Cr3 to give the current level_4 table
/// Should only be called one
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
/// # Safety
/// It is unsafe as the complete mapping has to be guaranteed by the caller.
/// Must be called at least once to avoid `&mut` aliasing
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    PHYSICAL_OFFSET = physical_memory_offset.as_u64();
    let level_4_table = active_level_4_table(physical_memory_offset);

    // Just for the stats, can be removed
    let mut compte = 0;
    for i in 0..512 {
        if level_4_table[i].is_unused() {
            compte += 1
        } else {
            let addr = level_4_table[i].addr();
            let virt = physical_memory_offset + addr.as_u64();
            let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
            let level_3_table = &mut *page_table_ptr;
            for i2 in 0..512 {
                if !level_3_table[i2].is_unused() {
                    println!("{} at {} with {:?}", i2, i, level_3_table[i].flags());
                    //if
                }
            }
        }
    }
    println!("Nb Frame used : {}.", compte);
    print!("Phys_offset : {:?}", physical_memory_offset);
    // loop {}
    // End of stats

    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Structure of the page allocator, holding the data of every page available.
/// It can be improved in terms of place taken and performances
pub struct BootInfoAllocator {
    pages_available: [bool; MAX_PAGE_ALLOWED], // table of every pages with a bool. Should be changed to u8 to improve perfs
    next: usize, // next table entry to check for free place to improve perfs
    maxi: usize, //  maximal index in RAM to improve perfs
    level4_table: &'static PageTable, // level4_table : kernel's level 4 table
}

impl BootInfoAllocator {
    /// # Safety depends on the validity of the inputs
    /// Creates a new allocator from the RAM map given by the bootloader
    /// and the offset to the physical memory given also by the bootloader
    pub unsafe fn init(memory_map: &'static MemoryMap, physical_memory_offset: VirtAddr) {
        let mut pages_available = [false; MAX_PAGE_ALLOWED];
        let regions = memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        let mut maxi = 0;
        let mut next = 0;

        // Fill up the table with every addresses
        for i in frame_addresses {
            NUMBER_TABLES += 1;
            pages_available[(i >> 12) as usize] = true;
            maxi = max((i >> 12) as usize, maxi);
            next = min((i >> 12) as usize, next);
        }
        println!("Num tables : {}", NUMBER_TABLES); // just for show, should be removed

        FRAME_ALLOCATOR = Some(BootInfoAllocator {
            pages_available,
            next,
            maxi,
            level4_table: active_level_4_table(physical_memory_offset),
        });
    }

    pub fn empty() -> Self {
        unsafe {
            Self {
                pages_available: [false; MAX_PAGE_ALLOWED],
                next: 0,
                maxi: 0,
                level4_table: &*VirtAddr::zero().as_ptr(),
            }
        }
    }

    /// Returns a new unallocated Frame and marks it as allocated.
    pub fn allocate_4k_frame(&mut self) -> Option<PhysAddr> {
        for _i in 0..MAX_PAGE_ALLOWED {
            // doesn't use a loop to handle the case where everything is already allocated
            if self.pages_available[self.next] {
                self.pages_available[self.next] = false;
                self.next += 1; // thus next should be decreased before usage

                let phys = PhysAddr::new(((self.next as u64) - 1) << 12);
                return Some(phys);
            } else {
                self.next += 1;
                if self.next > self.maxi {
                    self.next = 0;
                }
            }
        }
        warningln!("memory is full");
        None
    }

    /// # No garbage collector you should think above deallocating !
    /// Creates a new level_4 table and taking into account the kernel adresses.
    pub unsafe fn allocate_level_4_frame(&mut self) -> Result<PhysFrame, MemoryError> {
        if let Some(phys) = self.allocate_4k_frame() {
            //warningln!("l.138 success");
            // let phys = frame.start_address();
            let virt = VirtAddr::new(phys.as_u64() + PHYSICAL_OFFSET);
            let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
            let level_4_table = &mut *page_table_ptr;
            for i in 0..512 {
                level_4_table[i]
                    .set_addr(self.level4_table[i].addr(), self.level4_table[i].flags());
                // copies the data from the kernels table
            }
            Ok(PhysFrame::containing_address(phys))
        } else {
            warningln!("l.150 failure");
            Err(MemoryError())
        }
    }

    /// # Beware of giving a valid level 4 table
    /// Adds an entry to the level 4 table with the given flags at the given virtual address
    pub unsafe fn add_entry_to_table(
        &mut self,
        table_4: PhysFrame,
        virt_4: VirtAddr,
        flags: PageTableFlags,
    ) -> Result<(), MemoryError> {
        let virt = VirtAddr::new(table_4.start_address().as_u64() + PHYSICAL_OFFSET);
        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
        self.add_entry_to_table_4(&mut *page_table_ptr, virt_4, flags)
    }

    /// Creates a new entry in the level_4 table at the given entry (virt) with the given flags
    /// # Safety
    /// You should mark it as USER_ACCESSIBLE and PRESENT !
    pub unsafe fn add_entry_to_table_4(
        &mut self,
        table_4: &'static mut PageTable,
        virt_4: VirtAddr,
        flags: PageTableFlags,
    ) -> Result<(), MemoryError> {
        let p_4 = virt_4.p4_index();
        let entry = table_4[p_4].flags();
        if entry.contains(PageTableFlags::PRESENT) {
            if entry.contains(PageTableFlags::USER_ACCESSIBLE) {
                table_4[p_4].set_flags(entry | flags);
                //warningln!("already existed for user l.178");
                let virt = VirtAddr::new(table_4[p_4].addr().as_u64() + PHYSICAL_OFFSET);
                let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                self.add_entry_to_table_3(&mut *page_table_ptr, virt_4, flags)
            } else {
                warningln!("already existed for kernel l.183 failure");
                warningln!("p4 address : {:#?} of {:#?}", p_4, virt_4);
                warningln!("{:#?}", entry);
                Err(MemoryError())
            }
        } else {
            //warningln!("l.187 new page");
            match self.allocate_4k_frame() {
                None => Err(MemoryError()),
                Some(addr) => {
                    //warningln!("l.191 goes in deaper");
                    //let addr = phys_frame.start_address();
                    table_4[p_4].set_addr(addr, flags);
                    let virt = VirtAddr::new(table_4[p_4].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    table_4[p_4].set_flags(entry | flags);
                    self.add_entry_to_table_3(&mut *page_table_ptr, virt_4, flags)
                }
            }
        }
    }

    /// Inner function of add_entry_to_table
    unsafe fn add_entry_to_table_3(
        &mut self,
        table_3: &'static mut PageTable,
        virt_3: VirtAddr,
        flags: PageTableFlags,
    ) -> Result<(), MemoryError> {
        let p_3 = virt_3.p3_index();
        let entry = table_3[p_3].flags();
        if entry.contains(PageTableFlags::PRESENT) {
            if entry.contains(PageTableFlags::USER_ACCESSIBLE) {
                table_3[p_3].set_flags(entry | flags);
                let virt = VirtAddr::new(table_3[p_3].addr().as_u64() + PHYSICAL_OFFSET);
                let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                table_3[p_3].set_flags(entry | flags);
                self.add_entry_to_table_2(&mut *page_table_ptr, virt_3, flags)
            } else {
                warningln!("line 240");
                Err(MemoryError())
            }
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError()),
                Some(addr) => {
                    // let addr = phys_frame.start_address();
                    table_3[p_3].set_addr(addr, flags);
                    let virt = VirtAddr::new(table_3[p_3].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    self.add_entry_to_table_2(&mut *page_table_ptr, virt_3, flags)
                }
            }
        }
    }

    /// Inner function of add_entry_to_table
    unsafe fn add_entry_to_table_2(
        &mut self,
        table_2: &'static mut PageTable,
        virt_2: VirtAddr,
        flags: PageTableFlags,
    ) -> Result<(), MemoryError> {
        let p_2 = virt_2.p2_index();
        let entry = table_2[p_2].flags();
        if entry.contains(PageTableFlags::PRESENT) {
            if entry.contains(PageTableFlags::USER_ACCESSIBLE) {
                table_2[p_2].set_flags(entry | flags);
                let virt = VirtAddr::new(table_2[p_2].addr().as_u64() + PHYSICAL_OFFSET);
                let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                table_2[p_2].set_flags(entry | flags);
                self.add_entry_to_table_1(&mut *page_table_ptr, virt_2, flags)
            } else {
                warningln!("line 274");
                Err(MemoryError())
            }
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError()),
                Some(addr) => {
                    //let addr = phys_frame.start_address();
                    table_2[p_2].set_addr(addr, flags);
                    let virt = VirtAddr::new(table_2[p_2].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    self.add_entry_to_table_1(&mut *page_table_ptr, virt_2, flags)
                }
            }
        }
    }

    /// Inner function of add_entry_to_table
    unsafe fn add_entry_to_table_1(
        &mut self,
        table_1: &'static mut PageTable,
        virt_1: VirtAddr,
        flags: PageTableFlags,
    ) -> Result<(), MemoryError> {
        let p_1 = virt_1.p1_index();
        let entry = table_1[p_1].flags();
        if entry.contains(PageTableFlags::PRESENT) {
            warningln!("already here, l.301 {:#?}", virt_1);
            Err(MemoryError())
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError()),
                Some(addr) => {
                    //let addr = phys_frame.start_address();
                    table_1[p_1].set_addr(addr, flags);
                    Ok(())
                }
            }
        }
    }

    /// # Safety
    /// TODO
    pub unsafe fn add_entry_to_table_with_data(
        &mut self,
        table_4: PhysFrame,
        virt_4: VirtAddr,
        flags: PageTableFlags,
        data: &[u64; 512],
    ) -> Result<(), MemoryError> {
        let virt = VirtAddr::new(table_4.start_address().as_u64() + PHYSICAL_OFFSET);
        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
        self.add_entry_to_table_4_with_data(&mut *page_table_ptr, virt_4, flags, data)
    }

    /// Creates a new entry in the level_4 table at the given entry (virt) with the given flags and the given data
    /// # Safety
    /// You should mark it as USER_ACCESSIBLE and PRESENT !
    pub unsafe fn add_entry_to_table_4_with_data(
        &mut self,
        table_4: &'static mut PageTable,
        virt_4: VirtAddr,
        flags: PageTableFlags,
        data: &[u64; 512],
    ) -> Result<(), MemoryError> {
        //warningln!("entered level 4");
        let p_4 = virt_4.p4_index();
        let entry = table_4[p_4].flags();
        if entry.contains(PageTableFlags::PRESENT) {
            if entry.contains(PageTableFlags::USER_ACCESSIBLE) {
                table_4[p_4].set_flags(entry | flags);
                let virt = VirtAddr::new(table_4[p_4].addr().as_u64() + PHYSICAL_OFFSET);
                let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                self.add_entry_to_table_3_with_data(&mut *page_table_ptr, virt_4, flags, data)
            } else {
                Err(MemoryError())
            }
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError()),
                Some(addr) => {
                    //let addr = phys_frame.start_address();
                    table_4[p_4].set_addr(addr, flags);
                    let virt = VirtAddr::new(table_4[p_4].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    table_4[p_4].set_flags(entry | flags);
                    self.add_entry_to_table_3_with_data(&mut *page_table_ptr, virt_4, flags, data)
                }
            }
        }
    }

    /// Inner function of add_entry_to_table with data
    unsafe fn add_entry_to_table_3_with_data(
        &mut self,
        table_3: &'static mut PageTable,
        virt_3: VirtAddr,
        flags: PageTableFlags,
        data: &[u64; 512],
    ) -> Result<(), MemoryError> {
        //warningln!("entered level 3");
        let p_3 = virt_3.p3_index();
        let entry = table_3[p_3].flags();
        if entry.contains(PageTableFlags::PRESENT) {
            if entry.contains(PageTableFlags::USER_ACCESSIBLE) {
                table_3[p_3].set_flags(entry | flags);
                let virt = VirtAddr::new(table_3[p_3].addr().as_u64() + PHYSICAL_OFFSET);
                let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                table_3[p_3].set_flags(entry | flags);
                self.add_entry_to_table_2_with_data(&mut *page_table_ptr, virt_3, flags, data)
            } else {
                Err(MemoryError())
            }
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError()),
                Some(addr) => {
                    // let addr = phys_frame.start_address();
                    table_3[p_3].set_addr(addr, flags);
                    let virt = VirtAddr::new(table_3[p_3].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    self.add_entry_to_table_2_with_data(&mut *page_table_ptr, virt_3, flags, data)
                }
            }
        }
    }

    /// Inner function of add_entry_to_table with data
    unsafe fn add_entry_to_table_2_with_data(
        &mut self,
        table_2: &'static mut PageTable,
        virt_2: VirtAddr,
        flags: PageTableFlags,
        data: &[u64; 512],
    ) -> Result<(), MemoryError> {
        //warningln!("entered level 2");
        let p_2 = virt_2.p2_index();
        let entry = table_2[p_2].flags();
        if entry.contains(PageTableFlags::PRESENT) {
            if entry.contains(PageTableFlags::USER_ACCESSIBLE) {
                table_2[p_2].set_flags(entry | flags);
                let virt = VirtAddr::new(table_2[p_2].addr().as_u64() + PHYSICAL_OFFSET);
                let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                table_2[p_2].set_flags(entry | flags);
                self.add_entry_to_table_1_with_data(&mut *page_table_ptr, virt_2, flags, data)
            } else {
                Err(MemoryError())
            }
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError()),
                Some(addr) => {
                    //let addr = phys_frame.start_address();
                    table_2[p_2].set_addr(addr, flags);
                    let virt = VirtAddr::new(addr.as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    self.add_entry_to_table_1_with_data(&mut *page_table_ptr, virt_2, flags, data)
                }
            }
        }
    }

    /// Inner function of add_entry_to_table with data
    unsafe fn add_entry_to_table_1_with_data(
        &mut self,
        table_1: &'static mut PageTable,
        virt_1: VirtAddr,
        flags: PageTableFlags,
        data: &[u64; 512],
    ) -> Result<(), MemoryError> {
        //warningln!("entered level 1");
        let p_1 = virt_1.p1_index();
        let entry = table_1[p_1].flags();
        if entry.contains(PageTableFlags::PRESENT) {
            Err(MemoryError())
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError()),
                Some(addr) => {
                    table_1[p_1].set_addr(addr, flags);
                    let virt = VirtAddr::new(addr.as_u64() + PHYSICAL_OFFSET);
                    let content: *mut [u64; 512] = virt.as_mut_ptr();
                    //warningln!("starts copying");
                    (*content).clone_from_slice(&data[..512]);
                    //warningln!("copied");
                    Ok(())
                }
            }
        }
    }

    /// # Safety
    /// TODO
    pub unsafe fn copy_table_entries(
        &mut self,
        table_4: PhysAddr,
    ) -> Result<PhysAddr, MemoryError> {
        let virt = VirtAddr::new(table_4.as_u64() + PHYSICAL_OFFSET);
        let table4: *mut PageTable = virt.as_mut_ptr();
        match self.copy_table_4(&*table4) {
            Ok(phys) => Ok(phys),
            Err(MemoryError()) => Err(MemoryError()),
        }
    }

    /// Function to copy a table of level 4 in order to allow fork operations
    unsafe fn copy_table_4(
        &mut self,
        table_4: &'static PageTable,
    ) -> Result<PhysAddr, MemoryError> {
        if let Some(new_table_addr) = self.allocate_4k_frame() {
            let virt_table = VirtAddr::new(new_table_addr.as_u64() + PHYSICAL_OFFSET);
            let new_table: *mut PageTable = virt_table.as_mut_ptr();
            for index in 0..512 {
                let flags = table_4[index].flags();
                if flags.contains(PageTableFlags::PRESENT) {
                    if flags.contains(PageTableFlags::USER_ACCESSIBLE) {
                        let virt = VirtAddr::new(table_4[index].addr().as_u64() + PHYSICAL_OFFSET);
                        let level_3: *mut PageTable = virt.as_mut_ptr();
                        if let Ok(level_3_addr) = self.copy_table_3(&*level_3) {
                            (*new_table)[index].set_addr(level_3_addr, flags);
                        } else {
                            for i in index..512 {
                                (*new_table)[i].set_flags(PageTableFlags::empty());
                            }
                            return Err(MemoryError());
                        }
                    } else {
                        (*new_table)[index].set_addr(table_4[index].addr(), flags);
                    }
                } else {
                    (*new_table)[index].set_flags(flags);
                }
            }
            println!("new cr3 address : {:#?}", new_table_addr);
            Ok(new_table_addr)
        } else {
            Err(MemoryError())
        }
    }

    /// Function to copy a table of level 3 in order to allow fork operations
    unsafe fn copy_table_3(
        &mut self,
        table_3: &'static PageTable,
    ) -> Result<PhysAddr, MemoryError> {
        if let Some(new_table_addr) = self.allocate_4k_frame() {
            let virt_table = VirtAddr::new(new_table_addr.as_u64() + PHYSICAL_OFFSET);
            let new_table: *mut PageTable = virt_table.as_mut_ptr();
            for index in 0..512 {
                let flags = table_3[index].flags();
                if flags.contains(PageTableFlags::PRESENT) {
                    if flags.contains(PageTableFlags::USER_ACCESSIBLE) {
                        let virt = VirtAddr::new(table_3[index].addr().as_u64() + PHYSICAL_OFFSET);
                        let level_2: *mut PageTable = virt.as_mut_ptr();
                        if let Ok(level_2_addr) = self.copy_table_2(&*level_2) {
                            (*new_table)[index].set_addr(level_2_addr, flags);
                        } else {
                            for i in index..512 {
                                (*new_table)[i].set_flags(PageTableFlags::empty());
                            }
                            return Err(MemoryError());
                        }
                    } else {
                        (*new_table)[index].set_addr(table_3[index].addr(), flags);
                    }
                } else {
                    (*new_table)[index].set_flags(flags);
                }
            }
            Ok(new_table_addr)
        } else {
            Err(MemoryError())
        }
    }

    /// Function to copy a table of level 2 in order to allow fork operations
    unsafe fn copy_table_2(
        &mut self,
        table_2: &'static PageTable,
    ) -> Result<PhysAddr, MemoryError> {
        if let Some(new_table_addr) = self.allocate_4k_frame() {
            let virt_table = VirtAddr::new(new_table_addr.as_u64() + PHYSICAL_OFFSET);
            let new_table: *mut PageTable = virt_table.as_mut_ptr();
            for index in 0..512 {
                let flags = table_2[index].flags();
                if flags.contains(PageTableFlags::PRESENT) {
                    if flags.contains(PageTableFlags::USER_ACCESSIBLE) {
                        let virt = VirtAddr::new(table_2[index].addr().as_u64() + PHYSICAL_OFFSET);
                        let level_1: *mut PageTable = virt.as_mut_ptr();
                        if let Ok(level_1_addr) = self.copy_table_1(&*level_1) {
                            (*new_table)[index].set_addr(level_1_addr, flags);
                        } else {
                            for i in index..512 {
                                (*new_table)[i].set_flags(PageTableFlags::empty());
                            }
                            return Err(MemoryError());
                        }
                    } else {
                        (*new_table)[index].set_addr(table_2[index].addr(), flags);
                    }
                } else {
                    (*new_table)[index].set_flags(flags);
                }
            }
            Ok(new_table_addr)
        } else {
            Err(MemoryError())
        }
    }

    /// Function to copy a table of level 1 in order to allow fork operations
    unsafe fn copy_table_1(
        &mut self,
        table_1: &'static PageTable,
    ) -> Result<PhysAddr, MemoryError> {
        if let Some(new_table_addr) = self.allocate_4k_frame() {
            let virt_table = VirtAddr::new(new_table_addr.as_u64() + PHYSICAL_OFFSET);
            let new_table: *mut PageTable = virt_table.as_mut_ptr();
            for index in 0..512 {
                let flags = table_1[index].flags();
                if flags.contains(PageTableFlags::PRESENT) {
                    if flags.contains(PageTableFlags::USER_ACCESSIBLE) {
                        if let Some(data_table) = self.allocate_4k_frame() {
                            let virt =
                                VirtAddr::new(table_1[index].addr().as_u64() + PHYSICAL_OFFSET);
                            let old_table: *mut [u64; 512] = virt.as_mut_ptr();
                            let virt_next = VirtAddr::new(data_table.as_u64() + PHYSICAL_OFFSET);
                            let next_table: *mut [u64; 512] = virt_next.as_mut_ptr();
                            for i in 0..512 {
                                (*next_table)[i] = (*old_table)[i];
                            }
                            (*new_table)[index].set_addr(data_table, flags);
                        } else {
                            for i in index..512 {
                                (*new_table)[i].set_flags(PageTableFlags::empty());
                            }
                            return Err(MemoryError());
                        }
                    } else {
                        (*new_table)[index].set_addr(table_1[index].addr(), flags);
                    }
                } else {
                    (*new_table)[index].set_flags(flags);
                }
            }
            Ok(new_table_addr)
        } else {
            Err(MemoryError())
        }
    }

    /// Deallocator, from a given level 4 table, deallocates every thing containing the given flags.
    /// # Safety : Always put PageTableFlags::PRESENT in the given flags !
    /// You must give a level 4 table and the flags for which you want to remove the entries.
    /// For exemple you can use PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE as default
    /// Returns a boolean wethere the table is empty or not
    pub unsafe fn deallocate_level_4_page(
        &mut self,
        table_4_addr: PhysAddr,
        remove_flags: PageTableFlags,
    ) -> Result<bool, MemoryError> {
        let mut failed = false;
        let virt = VirtAddr::new(table_4_addr.as_u64() + PHYSICAL_OFFSET);
        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
        let table_4 = &mut *page_table_ptr;
        let mut is_empty = true;
        for i in 0..512 {
            if !table_4[i].is_unused() {
                let flags = table_4[i].flags();
                if flags.contains(remove_flags) {
                    let virt = VirtAddr::new(table_4[i].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    match self.deallocate_level_3_page(&mut *page_table_ptr, remove_flags) {
                        Ok(flags_level_3) => {
                            if flags_level_3.is_empty() {
                                table_4[i].set_flags(PageTableFlags::empty());
                                if self.deallocate_4k_frame(table_4[i].addr()).is_err() {
                                    failed = true;
                                }
                            } else {
                                is_empty = false;
                                table_4[i].set_flags(flags & flags_level_3);
                            }
                        }

                        Err(MemoryError()) => failed = true,
                    }
                } else if flags.contains(PageTableFlags::PRESENT) {
                    is_empty = false;
                }
            }
        }

        if failed {
            Err(MemoryError())
        } else {
            Ok(is_empty)
        }
    }

    /// Inner function of deallocate_level_4_page
    unsafe fn deallocate_level_3_page(
        &mut self,
        table_3: &'static mut PageTable,
        remove_flags: PageTableFlags,
    ) -> Result<PageTableFlags, MemoryError> {
        let mut failed = false;
        let mut flags_left = PageTableFlags::empty();
        for i in 0..512 {
            if !table_3[i].is_unused() {
                let flags = table_3[i].flags();
                if flags.contains(remove_flags) {
                    let virt = VirtAddr::new(table_3[i].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    match self.deallocate_level_2_page(&mut *page_table_ptr, remove_flags) {
                        Ok(flags_level_2) => {
                            if flags_level_2.is_empty() {
                                table_3[i].set_flags(PageTableFlags::empty());
                                if self.deallocate_4k_frame(table_3[i].addr()).is_err() {
                                    failed = true;
                                }
                            } else {
                                table_3[i].set_flags(flags & flags_level_2);
                                flags_left = flags_left | (flags & flags_level_2);
                            }
                        }

                        Err(MemoryError()) => failed = true,
                    }
                } else if flags.contains(PageTableFlags::PRESENT) {
                    flags_left = flags_left | flags;
                }
            }
        }
        if failed {
            Err(MemoryError())
        } else {
            Ok(flags_left)
        }
    }

    /// Inner function of deallocate_level_4_page
    unsafe fn deallocate_level_2_page(
        &mut self,
        table_2: &'static mut PageTable,
        remove_flags: PageTableFlags,
    ) -> Result<PageTableFlags, MemoryError> {
        let mut failed = false;
        let mut flags_left = PageTableFlags::empty();
        for i in 0..512 {
            if !table_2[i].is_unused() {
                let flags = table_2[i].flags();
                if flags.contains(remove_flags) {
                    let virt = VirtAddr::new(table_2[i].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    match self.deallocate_level_1_page(&mut *page_table_ptr, remove_flags) {
                        Ok(flags_level_1) => {
                            if flags_level_1.is_empty() {
                                table_2[i].set_flags(PageTableFlags::empty());
                                if self.deallocate_4k_frame(table_2[i].addr()).is_err() {
                                    failed = true;
                                }
                            } else {
                                table_2[i].set_flags(flags & flags_level_1);
                                flags_left = flags_left | (flags & flags_level_1);
                            }
                        }

                        Err(MemoryError()) => failed = true,
                    }
                } else if flags.contains(PageTableFlags::PRESENT) {
                    flags_left = flags_left | flags;
                }
            }
        }
        if failed {
            Err(MemoryError())
        } else {
            Ok(flags_left)
        }
    }

    /// Inner function of deallocate_level_4_page
    unsafe fn deallocate_level_1_page(
        &mut self,
        table_1: &'static mut PageTable,
        remove_flags: PageTableFlags,
    ) -> Result<PageTableFlags, MemoryError> {
        let mut failed = false;
        let mut flags_left = PageTableFlags::empty();
        for i in 0..512 {
            if !table_1[i].is_unused() {
                let flags = table_1[i].flags();
                if flags.contains(remove_flags) {
                    table_1[i].set_flags(PageTableFlags::empty());
                    if self.deallocate_4k_frame(table_1[i].addr()).is_err() {
                        failed = true;
                    }
                } else {
                    flags_left = flags_left | flags
                }
            }
        }
        if failed {
            Err(MemoryError())
        } else {
            Ok(flags_left)
        }
    }

    /// Can be used to deallocate a specific 4Ki frame
    pub fn deallocate_4k_frame(&mut self, addr: PhysAddr) -> Result<(), MemoryError> {
        let table_index = addr.as_u64() >> 12;
        self.pages_available[table_index as usize] = false;
        Ok(())
    }
}

/// Implementation of the trait FrameAllocator for the global API
unsafe impl FrameAllocator<Size4KiB> for BootInfoAllocator {
    #[inline]
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        match self.allocate_4k_frame() {
            Some(addr) => match PhysFrame::from_start_address(addr) {
                Ok(frame) => Some(frame),
                Err(_) => None,
            },
            None => None,
        }
        // PhysFrame::from_start_address(self.allocate_4k_frame())
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
