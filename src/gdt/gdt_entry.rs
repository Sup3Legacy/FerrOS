
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

    fn set_limit_low(&mut self, val : u16) {
        self.limit_low = val
    }

    fn set_limit_high(&mut self, val: u8) {
        self.granularity = (self.granularity & 0xF0) | val
    }

    pub fn set_limit(&mut self, val: u32) -> &mut Self {
        self.set_limit_low(val as u16);
        self.set_limit_high( (val >> 16) as u8);
        self
    }

    fn set_base_low(&mut self, val: u32) {
        self.base_low1 = (val >> 16) as u8;
        self.base_low2 = val as u16
    }

    fn set_base_high(&mut self, val: u8) {
        self.base_high = val
    }

    pub fn set_base(&mut self, val: u32) -> &mut Self {
        self.set_base_low(val & 0xFFF);
        self.set_base_high((val >> 24) as u8);
        self
    }

    pub fn set_accessed(&mut self, access: bool) -> &mut Self {
        if access {
            self.attributes = self.attributes | 0x01
        } else {
            self.attributes = self.attributes & !0x01
        }
        self
    }

    /// readable for code, writable for data
    pub fn set_read_write(&mut self, can_rw: bool) -> &mut Self {
        if can_rw {
            self.attributes = self.attributes | 0x02
        } else {
            self.attributes = self.attributes & !0x02
        }
        self
    }

    /// conforming for code, expand down data
    pub fn set_conforming_expand_down(&mut self, conforming: bool) -> &mut Self {
        if conforming {
            self.attributes = self.attributes | 0x04
        } else {
            self.attributes = self.attributes & !0x04
        }
        self
    }

    /// 1 for code, 0 for data
    pub fn is_code(&mut self, is_code: bool) -> &mut Self {
        if is_code {
            self.attributes = self.attributes | 0x08
        } else {
            self.attributes = self.attributes & !0x08
        }
        self
    }

    /// set privilege level
    pub fn set_dpl(&mut self, dpl: u8) -> &mut Self {
        self.attributes = (self.attributes & 0x9F) | ( (dpl & 0b11) << 1);
        self
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        if present {
            self.attributes = self.attributes | 0x80
        } else {
            self.attributes = self.attributes & !0x80
        }
        self
    }

    pub fn set_available(&mut self, available: bool) -> &mut Self {
        if available {
            self.granularity = self.granularity | 0x10
        } else {
            self.granularity = self.granularity & !0x10
        }
        self
    }

    /// is used to indicate x86-64 code descriptor. * For data segments, this bit is reserved *
    pub unsafe fn set_x86_64_code_descriptor(&mut self, code_descriptor: bool) -> &mut Self {
        if code_descriptor {
            self.granularity = self.granularity | 0x20
        } else {
            self.granularity = self.granularity & !0x20
        }
        self
    }

    /// 32bit opcodes for code, uint32_t stack for data must be 0 is L is 1 !
    pub fn set_big(&mut self, big: bool) -> &mut Self {
        if big {
            self.granularity = self.granularity | 0x40
        } else {
            self.granularity = self.granularity & !0x40
        }
        self
    }

    /// 1 to use 4k page addressing, 0 for byte addressing
    pub fn set_gran(&mut self, gran: bool) -> &mut Self {
        if gran {
            self.granularity = self.granularity | 0x80
        } else {
            self.granularity = self.granularity & !0x80
        }
        self
    }

    pub fn as_u64(& self) -> u64 {
        ((self.limit_low as u64) << 0) | ((self.base_low1 as u64) << 16)
            | ((self.base_low2 as u64) << 32)
            | ((self.attributes as u64) << 40)
            | ((self.granularity as u64) << 48)
            | ((self.base_high as u64) << 54)
    }


}

pub fn new_ds() -> u64 {
    let mut ds = GdtEntryBits::new();
    ds.set_limit(0xFFFFF)
        .set_base(0)
        .set_present(true)
        .set_read_write(true)
        .is_code(false)
        .set_dpl(3)
        .set_available(true)
        .set_big(true)
        .set_gran(true);
    ds.as_u64()
}

pub fn new_cs() -> u64 {
    let mut cs = GdtEntryBits::new();
    cs.set_limit(0xFFFFF)
        .set_base(0)
        .set_present(true)
        .set_read_write(true)
        .is_code(true)
        .set_dpl(3)
        .set_available(true)
        .set_big(true)
        .set_gran(true);
    unsafe { cs.set_x86_64_code_descriptor(true);};
    cs.as_u64()
}

pub fn kernel_cs() -> u64 {
    let mut kernel_cs = GdtEntryBits::new();
    kernel_cs.set_limit(0xFFFFF)
        .set_base(0)
        .set_present(true)
        .set_read_write(true)
        .is_code(true)
        .set_dpl(0)
        .set_big(false)
        .set_available(true)
        .set_gran(true);
    unsafe { kernel_cs.set_x86_64_code_descriptor(true);};
    kernel_cs.as_u64()
}

pub fn kernel_ds() -> u64 {
    let mut kernel_ds = GdtEntryBits::new();
    kernel_ds.set_limit(0xFFFFF)
        .set_base(0)
        .set_present(true)
        .set_read_write(true)
        .is_code(false)
        .set_dpl(0)
        .set_big(true)
        .set_gran(true);
    kernel_ds.as_u64()
}