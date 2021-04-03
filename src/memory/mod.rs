//! Crate for managing the paging: allocating and desallocating pages and editing page tables
use crate::{debug, println};
use alloc::string::String;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use core::cmp::{max, min};
use x86_64::structures::paging::OffsetPageTable;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{PageTable, PageTableFlags},
    PhysAddr, VirtAddr,
};

use crate::warningln;

/// Static structure holding the frame allocator. You can borrow it but never place it back to None !.
/// You can asume it is never None.
pub static mut FRAME_ALLOCATOR: Option<BootInfoAllocator> = None;

#[derive(Debug)]
pub struct MemoryError(pub String);

/// Memory address translation (virtual -> physical) now has to be done with `Translate::translate_addr`
pub static mut PHYSICAL_OFFSET: u64 = 0;

/// Maximum number of pages allowed. Can hold 256MiB of RAM, must be increased to have a higher capacity
const MAX_PAGE_ALLOWED: usize = 65536;

/// Number of allocatable tables
static mut NUMBER_TABLES: u64 = 0;

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
    let level_4_table: &'static mut PageTable = active_level_4_table(physical_memory_offset);

    // Just for the stats, can be removed
    let mut compte = 512;
    for i in 0..512 {
        if level_4_table[i].flags().contains(PageTableFlags::PRESENT) {
            compte -= 1;
            /*let addr = level_4_table[i].addr();
            let virt = physical_memory_offset + addr.as_u64();
            let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
            let level_3_table = &mut *page_table_ptr;
            println!("{} with {:?}", i, level_4_table[i].flags());
            for i2 in 0..512 {
                if !level_3_table[i2].is_unused() {
                    println!("{} at {} with {:?}", i2, i, level_3_table[i2].flags());
                    if i < 250 {
                        for i3 in 0..512 {
                            let virt = physical_memory_offset + level_3_table[i2].addr().as_u64();
                            let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                            let level_2_table = &mut *page_table_ptr;
                            if !level_2_table[i3].is_unused() {
                                println!(" -> {} at {} with {:?}", i3, i2, level_2_table[i3].flags());
                            }
                        }
                    }
                }
            }*/
            if i < 256 {
                let flags = level_4_table[i].flags();
                level_4_table[i].set_flags(flags | PageTableFlags::BIT_9);
                let virt = physical_memory_offset + level_4_table[i].addr().as_u64();
                let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                let level_3_table = &mut *page_table_ptr;
                for i3 in 0..512 {
                    if level_3_table[i3].flags().contains(PageTableFlags::PRESENT) {
                        let flags = level_3_table[i3].flags();
                        level_3_table[i3].set_flags(flags | PageTableFlags::BIT_9);
                        let virt = physical_memory_offset + level_3_table[i3].addr().as_u64();
                        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                        let level_2_table = &mut *page_table_ptr;
                        for i2 in 0..512 {
                            if level_2_table[i2].flags().contains(PageTableFlags::PRESENT) {
                                let flags = level_2_table[i2].flags();
                                level_2_table[i2].set_flags(flags | PageTableFlags::BIT_9);
                                let virt =
                                    physical_memory_offset + level_2_table[i2].addr().as_u64();
                                let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                                let level_1_table = &mut *page_table_ptr;
                                for i1 in 0..512 {
                                    if level_1_table[i1].flags().contains(PageTableFlags::PRESENT) {
                                        let flags = level_1_table[i1].flags();
                                        level_1_table[i1].set_flags(flags | PageTableFlags::BIT_9);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("Nb Frame unused in level 4 table : {}.", compte);
    println!("Phys_offset : {:?}", physical_memory_offset);
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
        println!("Number of available tables in RAM : {}", NUMBER_TABLES); // just for show, should be removed

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
            Err(MemoryError(String::from("Could not create level 4 table")))
        }
    }

    /// # Beware of giving a valid level 4 table
    /// Adds an entry to the level 4 table with the given flags at the given virtual address
    pub unsafe fn add_entry_to_table(
        &mut self,
        table_4: PhysFrame,
        virt_4: VirtAddr,
        flags: PageTableFlags,
        allow_duplicate: bool,
    ) -> Result<(), MemoryError> {
        let virt = VirtAddr::new(table_4.start_address().as_u64() + PHYSICAL_OFFSET);
        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
        self.add_entry_to_table_4(&mut *page_table_ptr, virt_4, flags, allow_duplicate)
    }

    /// Creates a new entry in the level_4 table at the given entry (virt) with the given flags
    /// # Safety
    /// You should mark it as USER_ACCESSIBLE and PRESENT !
    pub unsafe fn add_entry_to_table_4(
        &mut self,
        table_4: &'static mut PageTable,
        virt_4: VirtAddr,
        flags: PageTableFlags,
        allow_duplicate: bool,
    ) -> Result<(), MemoryError> {
        let p_4 = virt_4.p4_index();
        let entry = table_4[p_4].flags();
        if entry.contains(PageTableFlags::PRESENT) {
            if entry.contains(PageTableFlags::USER_ACCESSIBLE) {
                table_4[p_4].set_flags(entry | flags);
                //warningln!("already existed for user l.178");
                let virt = VirtAddr::new(table_4[p_4].addr().as_u64() + PHYSICAL_OFFSET);
                let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                self.add_entry_to_table_3(&mut *page_table_ptr, virt_4, flags, allow_duplicate)
            } else {
                warningln!("already existed for kernel l.183 failure");
                warningln!("p4 address : {:#?} of {:#?}", p_4, virt_4);
                warningln!("{:#?}", entry);
                Err(MemoryError(String::from(
                    "Level 4 entry is not USER_ACCESSIBLE",
                )))
            }
        } else {
            //warningln!("l.187 new page");
            match self.allocate_4k_frame() {
                None => Err(MemoryError(String::from("Could not allocate 4k @ level 4"))),
                Some(addr) => {
                    //warningln!("l.191 goes in deaper");
                    //let addr = phys_frame.start_address();
                    table_4[p_4].set_addr(addr, flags);
                    let virt = VirtAddr::new(table_4[p_4].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    for i in 0..512 {
                        (*page_table_ptr)[i].set_flags(PageTableFlags::empty());
                    }
                    table_4[p_4].set_flags(entry | flags);
                    self.add_entry_to_table_3(&mut *page_table_ptr, virt_4, flags, allow_duplicate)
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
        allow_duplicate: bool,
    ) -> Result<(), MemoryError> {
        let p_3 = virt_3.p3_index();
        let entry = table_3[p_3].flags();
        if entry.contains(PageTableFlags::PRESENT) {
            if entry.contains(PageTableFlags::USER_ACCESSIBLE) {
                table_3[p_3].set_flags(entry | flags);
                let virt = VirtAddr::new(table_3[p_3].addr().as_u64() + PHYSICAL_OFFSET);
                let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                table_3[p_3].set_flags(entry | flags);
                self.add_entry_to_table_2(&mut *page_table_ptr, virt_3, flags, allow_duplicate)
            } else {
                warningln!("line 240");
                Err(MemoryError(String::from(
                    "Level 3 entry is not USER_ACCESSIBLE",
                )))
            }
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError(String::from(
                    "Could not allocate 4k frame @ level 3",
                ))),
                Some(addr) => {
                    // let addr = phys_frame.start_address();
                    table_3[p_3].set_addr(addr, flags);
                    let virt = VirtAddr::new(table_3[p_3].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    for i in 0..512 {
                        (*page_table_ptr)[i].set_flags(PageTableFlags::empty());
                    }
                    self.add_entry_to_table_2(&mut *page_table_ptr, virt_3, flags, allow_duplicate)
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
        allow_duplicate: bool,
    ) -> Result<(), MemoryError> {
        let p_2 = virt_2.p2_index();
        let entry = table_2[p_2].flags();
        if entry.contains(PageTableFlags::PRESENT) {
            if entry.contains(PageTableFlags::USER_ACCESSIBLE) {
                table_2[p_2].set_flags(entry | flags);
                let virt = VirtAddr::new(table_2[p_2].addr().as_u64() + PHYSICAL_OFFSET);
                let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                table_2[p_2].set_flags(entry | flags);
                self.add_entry_to_table_1(&mut *page_table_ptr, virt_2, flags, allow_duplicate)
            } else {
                warningln!("line 274");
                Err(MemoryError(String::from(
                    "Level 2 entry is not USER_ACCESSIBLE",
                )))
            }
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError(String::from("Could not allocate 4k #2"))),
                Some(addr) => {
                    //let addr = phys_frame.start_address();
                    table_2[p_2].set_addr(addr, flags);
                    let virt = VirtAddr::new(table_2[p_2].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    for i in 0..512 {
                        (*page_table_ptr)[i].set_flags(PageTableFlags::empty());
                    }
                    self.add_entry_to_table_1(&mut *page_table_ptr, virt_2, flags, allow_duplicate)
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
        allow_duplicate: bool,
    ) -> Result<(), MemoryError> {
        let p_1 = virt_1.p1_index();
        let entry = table_1[p_1].flags();
        if entry.contains(PageTableFlags::PRESENT) {
            if allow_duplicate {
                table_1[p_1].set_flags(entry | flags);
                Ok(())
            } else {
                warningln!("already here, l.301 {:#?}", virt_1);
                Err(MemoryError(String::from(
                    "Level 1 entry is already present",
                )))
            }
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError(String::from(
                    "Could not allocate 4k frame @ level 1",
                ))),
                Some(addr) => {
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
                warningln!("already existed for kernel l.183 failure");
                warningln!("p4 address : {:#?} of {:#?}", p_4, virt_4);
                warningln!("{:#?}", entry);
                Err(MemoryError(String::from(
                    "Level 4 entry is not USER_ACCESSIBLE with data",
                )))
            }
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError(String::from(
                    "Could not allocate 4k frame @ level 4 with data",
                ))),
                Some(addr) => {
                    //let addr = phys_frame.start_address();
                    table_4[p_4].set_addr(addr, flags);
                    let virt = VirtAddr::new(table_4[p_4].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    for i in 0..512 {
                        (*page_table_ptr)[i].set_flags(PageTableFlags::empty());
                    }
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
                Err(MemoryError(String::from(
                    "Level 3 entry is not USER_ACCESSIBLE with data",
                )))
            }
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError(String::from(
                    "Could not allocate 4k frame @ level 3 with data",
                ))),
                Some(addr) => {
                    // let addr = phys_frame.start_address();
                    table_3[p_3].set_addr(addr, flags);
                    let virt = VirtAddr::new(table_3[p_3].addr().as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    for i in 0..512 {
                        (*page_table_ptr)[i].set_flags(PageTableFlags::empty());
                    }
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
                Err(MemoryError(String::from(
                    "Level 2 entry is not USER_ACCESSIBLE with data",
                )))
            }
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError(String::from(
                    "Could not allocate 4k frame @ level 2 with data",
                ))),
                Some(addr) => {
                    //let addr = phys_frame.start_address();
                    table_2[p_2].set_addr(addr, flags);
                    let virt = VirtAddr::new(addr.as_u64() + PHYSICAL_OFFSET);
                    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                    for i in 0..512 {
                        (*page_table_ptr)[i].set_flags(PageTableFlags::empty());
                    }
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
            Err(MemoryError(String::from(
                "Level 1 entry is already present with data",
            )))
        } else {
            match self.allocate_4k_frame() {
                None => Err(MemoryError(String::from(
                    "Could not allocate 4k frame @ level 1 with data",
                ))),
                Some(addr) => {
                    table_1[p_1].set_addr(addr, flags);
                    let virt = VirtAddr::new(addr.as_u64() + PHYSICAL_OFFSET);
                    let content: *mut [u64; 512] = virt.as_mut_ptr();
                    (*content).clone_from_slice(&data[..512]);
                    Ok(())
                }
            }
        }
    }

    /// # Safety
    /// Function to duplicate an level 4 table into a new one.
    /// Give a level 4 table, it gives you a new one holding the same datas
    pub unsafe fn copy_table_entries(
        &mut self,
        table_4: PhysAddr,
    ) -> Result<PhysAddr, MemoryError> {
        let virt = VirtAddr::new(table_4.as_u64() + PHYSICAL_OFFSET);
        let table4: *mut PageTable = virt.as_mut_ptr();
        match self.copy_table_4(&*table4) {
            Ok(phys) => Ok(phys),
            Err(MemoryError(a)) => Err(MemoryError(a)),
        }
    }

    /// Inner function to copy a table of level 4 in order to allow fork operations
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
                            return Err(MemoryError(String::from("Could not copy level 3 table")));
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
            Err(MemoryError(String::from(
                "Could not allocate 4k frame in copy_table_4",
            )))
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
                            return Err(MemoryError(String::from("Could not copy level 2 table")));
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
            Err(MemoryError(String::from(
                "Could not allocate 4k frame in copy_table_3",
            )))
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
                            return Err(MemoryError(String::from("Could not copy level 1 table")));
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
            Err(MemoryError(String::from(
                "Could not allocate 4k frame in copy_table_2",
            )))
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
                            return Err(MemoryError(String::from(
                                "Could not allocate 4k frame in level 1 copy #0",
                            )));
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
            Err(MemoryError(String::from(
                "Could not allocate 4k frame in level 1 copy #1",
            )))
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
                    debug!(
                        "0x{:x?}, {:x?}, {}",
                        table_4[i].addr().as_u64() + PHYSICAL_OFFSET,
                        PHYSICAL_OFFSET,
                        i
                    );
                    let virt = VirtAddr::new(table_4[i].addr().as_u64() + PHYSICAL_OFFSET);
                    debug!("qsd");
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

                        Err(MemoryError(_)) => failed = true,
                    }
                } else if flags.contains(PageTableFlags::PRESENT) {
                    is_empty = false;
                }
            }
        }

        if failed {
            Err(MemoryError(String::from(
                "Could not deallocate level 4 page",
            )))
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

                        Err(MemoryError(_)) => failed = true,
                    }
                } else if flags.contains(PageTableFlags::PRESENT) {
                    flags_left = flags_left | flags;
                }
            }
        }
        if failed {
            Err(MemoryError(String::from(
                "Could not deallocate level 3 page",
            )))
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

                        Err(MemoryError(_)) => failed = true,
                    }
                } else if flags.contains(PageTableFlags::PRESENT) {
                    flags_left = flags_left | flags;
                }
            }
        }
        if failed {
            Err(MemoryError(String::from(
                "Could not deallocate level 2 page",
            )))
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
            Err(MemoryError(String::from(
                "Could not deallocate level 1 page",
            )))
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

/// This may be totally wrong
pub unsafe fn write_into_virtual_memory(
    table_4: PhysFrame,
    virt_4: VirtAddr,
    data: &[u8],
) -> Result<(), MemoryError> {
    let offset: u64 = virt_4.page_offset().into();
    let length = data.len();
    let mut virtaddr: VirtAddr = virt_4;
    let mut physaddr: PhysAddr = match translate_addr(table_4, virtaddr) {
        Some(a) => a,
        None => {
            return Err(MemoryError(String::from(
                "Could not convert process-virtual to physical memory",
            )))
        }
    };
    for i in 0..length {
        let content: *mut u8 = (physaddr.as_u64() + i as u64 + PHYSICAL_OFFSET) as *mut u8;
        (*content) = data[i];
        virtaddr += 1_u64;
        if (i as u64 + offset) % 4096 == 0 {
            physaddr = match translate_addr(table_4, virtaddr) {
                Some(a) => a,
                None => {
                    return Err(MemoryError(String::from(
                        "Could not convert process-virtual to physical memory",
                    )))
                }
            };
        }
    }
    Ok(())
}

pub unsafe fn translate_addr_inner(table_4: PhysFrame, addr: VirtAddr) -> Option<PhysAddr> {
    translate_addr(table_4, addr)
}

unsafe fn translate_addr(table_4: PhysFrame, addr: VirtAddr) -> Option<PhysAddr> {
    //let (level_4_table_frame, _) = Cr3::read();
    let mut virt = VirtAddr::new(table_4.start_address().as_u64() + PHYSICAL_OFFSET).as_u64();

    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = PhysFrame::containing_address(PhysAddr::new(0_u64));

    for &index in &table_indexes {
        let table_ptr: *const PageTable = virt as *const PageTable;
        let table = &*table_ptr;

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(_) => return None,
        };
        virt = PHYSICAL_OFFSET + frame.start_address().as_u64();
    }
    Some(frame.start_address() + u64::from(addr.page_offset()))
}
