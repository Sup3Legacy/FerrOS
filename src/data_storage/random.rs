//! A very basic pseudo-RNG

/// Initital seed
static mut RAND_SEED: u8 = 42_u8;

/// Returns a pseudo-random `u8`, using a simple and naive algorihm.
/// We could use the CPU's built-in pRNG instructions.
///
/// # Safety
/// This is safe to call as long as the static mutable variable `seed` is defined, which is should always be
/// `seed` should not be modified by any other mean to guarantee that the randomness is satisfying
pub fn random_u8() -> u8 {
    unsafe {
        asm!(
            "shr {0}, 1",
            "jc 2f",
            "xor {0}, 0xB8",
            "2:",
            inout(reg_byte) RAND_SEED
        );
        RAND_SEED
    }
}
