static mut RAND_SEED: u8 = 42_u8;

/// # Safety
/// This is safe to call as long as the static mutable variable `seed` is defined, which is should always be
/// `seed` should not be modified by any other mean to guarantee that the randomness is satisfying
pub fn random_u8() -> u8 {
    unsafe{
        asm!(
            "lsr {0} 1",
            "jc random_end",
            "xor {0} 0xB8",
            "label random_end",
            inout(reg_byte) RAND_SEED
        );
        RAND_SEED
    }
}
