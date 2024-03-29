//! Crate that holds the structure repesenting the entry of a GDT

/// Implementation of the structure
#[repr(C)]
pub struct GdtEntryBits {
    base_high: u8,
    granularity: u8,
    attributes: u8,
    base_low1: u8,
    base_low2: u16,
    limit_low: u16,
}

impl GdtEntryBits {
    /// Create a new minimal entry
    pub fn new() -> Self {
        GdtEntryBits {
            base_high: 0,
            granularity: 0,
            attributes: 0x10,
            base_low1: 0,
            base_low2: 0,
            limit_low: 0,
        }
    }

    /// set the low part of the limit
    fn set_limit_low(&mut self, val: u16) {
        self.limit_low = val
    }

    /// set the highest part of the limit
    fn set_limit_high(&mut self, val: u8) {
        self.granularity = (self.granularity & 0xF0) | (val & 0x0F)
    }

    /// Set the limit in the right place (separate higher from the lower part)
    pub fn set_limit(&mut self, val: u32) -> &mut Self {
        self.set_limit_low(val as u16);
        self.set_limit_high((val >> 16) as u8);
        self
    }

    /// set the low part of the base
    fn set_base_low(&mut self, val: u32) {
        self.base_low1 = (val >> 16) as u8;
        self.base_low2 = val as u16
    }

    /// set the high part of the base
    fn set_base_high(&mut self, val: u8) {
        self.base_high = val
    }

    /// set the base in the right place (separate higher from lower part)
    pub fn set_base(&mut self, val: u32) -> &mut Self {
        self.set_base_low(val & 0xFFF);
        self.set_base_high((val >> 24) as u8);
        self
    }

    /// set the acessed attribute
    pub fn set_accessed(&mut self, access: bool) -> &mut Self {
        if access {
            self.attributes |= 0x01
        } else {
            self.attributes &= !0x01
        }
        self
    }

    /// set if the segment is readable for code, writable for data
    pub fn set_read_write(&mut self, can_rw: bool) -> &mut Self {
        if can_rw {
            self.attributes |= 0x02
        } else {
            self.attributes &= !0x02
        }
        self
    }

    /// set if the segment uses conforming for code, expand down for data (refer to OSdev)
    pub fn set_conforming_expand_down(&mut self, conforming: bool) -> &mut Self {
        if conforming {
            self.attributes |= 0x04
        } else {
            self.attributes &= !0x04
        }
        self
    }

    /// set wether the segment is code or data -> 1 for code, 0 for data
    pub fn is_code(&mut self, is_code: bool) -> &mut Self {
        if is_code {
            self.attributes |= 0x08
        } else {
            self.attributes &= !0x08
        }
        self
    }

    /// set privilege level from 0 to 3
    pub fn set_dpl(&mut self, dpl: u8) -> &mut Self {
        self.attributes = (self.attributes & 0x9F) | ((dpl & 0b11) << 5);
        self
    }

    /// set wether the segment is present or not
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        if present {
            self.attributes |= 0x80
        } else {
            self.attributes &= !0x80
        }
        self
    }

    /// set the available attribute of the segment
    pub fn set_available(&mut self, available: bool) -> &mut Self {
        if available {
            self.granularity |= 0x10
        } else {
            self.granularity &= !0x10
        }
        self
    }

    /// is used to indicate x86-64 code descriptor. * For data segments, this bit is reserved *
    pub unsafe fn set_x86_64_code_descriptor(&mut self, code_descriptor: bool) -> &mut Self {
        if code_descriptor {
            self.granularity |= 0x20
        } else {
            self.granularity &= !0x20
        }
        self
    }

    /// 32bit opcodes for code, uint32_t stack for data must be 0 is L is 1 !
    pub fn set_big(&mut self, big: bool) -> &mut Self {
        if big {
            self.granularity |= 0x40
        } else {
            self.granularity &= !0x40
        }
        self
    }

    /// 1 to use 4k page addressing, 0 for byte addressing
    pub fn set_gran(&mut self, gran: bool) -> &mut Self {
        if gran {
            self.granularity |= 0x80
        } else {
            self.granularity &= !0x80
        }
        self
    }

    /// convert the representation structure to u64 (should be improved !)
    pub fn as_u64(&self) -> u64 {
        (self.limit_low as u64) // << 0
            | ((self.base_low1 as u64) << 16)
            | ((self.base_low2 as u64) << 32)
            | ((self.attributes as u64) << 40)
            | ((self.granularity as u64) << 48)
            | ((self.base_high as u64) << 54)
    }
}

/// Creates a user data segment
pub fn new_ds() -> u64 {
    let mut ds = GdtEntryBits::new();
    ds.set_present(true)
        .set_read_write(true)
        .is_code(false)
        .set_dpl(3)
        .set_big(false)
        .set_available(false)
        .set_gran(true);
    ds.as_u64()
}

/// Creates a new user code segment
pub fn new_cs() -> u64 {
    let mut cs = GdtEntryBits::new();
    cs.set_present(true)
        .set_read_write(false)
        .is_code(true)
        .set_dpl(0)
        .set_big(false)
        .set_conforming_expand_down(true)
        .set_available(false)
        .set_gran(false);
    unsafe {
        cs.set_x86_64_code_descriptor(true);
    };
    cs.as_u64()
}

/// Creates a kernel code segment
pub fn kernel_cs() -> u64 {
    let mut kernel_cs = GdtEntryBits::new();
    kernel_cs
        .set_present(true)
        .set_read_write(false)
        .is_code(true)
        .set_dpl(0)
        .set_big(false)
        .set_conforming_expand_down(false)
        .set_available(false)
        .set_gran(false);
    unsafe {
        kernel_cs.set_x86_64_code_descriptor(true);
    };
    kernel_cs.as_u64()
}

/// Creates a kernel data segment
pub fn kernel_ds() -> u64 {
    let mut kernel_ds = GdtEntryBits::new();
    kernel_ds
        .set_present(true)
        .set_read_write(true)
        .is_code(false)
        .set_dpl(0)
        .set_big(false)
        .set_available(false)
        .set_gran(true);
    kernel_ds.as_u64()
}
