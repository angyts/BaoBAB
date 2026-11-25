//! Minimal BIO-generated I2S clock test for Dabao.
//!
//! DIN stays low (digital silence). This kernel does not play an audio tone.

use bao1x_api::bio_code;

pub const BIO_DIN: u8 = 24;   // PC8
pub const BIO_LRCLK: u8 = 26; // PC10
pub const BIO_BCLK: u8 = 27;  // PC11

pub const PIN_MASK: u32 = (1 << BIO_DIN) | (1 << BIO_LRCLK) | (1 << BIO_BCLK);
pub const TARGET_QUANTUM_HZ: u32 = 3_072_000;

/// With two quantum ticks per BCLK period:
/// BCLK = quantum / 2 = 1.536 MHz (nominal).
/// LRCLK toggles every 16 BCLK periods, hence one full LRCLK period
/// is 32 BCLK periods = 48 kHz (nominal).
///
/// BIO x23 uses clear-on-zero semantics: (x26 & !x23) is the set
/// of outputs to clear. Therefore use the full output mask excluding
/// the target pin when clearing one output without disturbing others.
#[rustfmt::skip]
bio_code!(i2s_clock_kernel, I2S_CLOCK_START, I2S_CLOCK_END,
    "li x26, 0x0D000000", // BIO24 + BIO26 + BIO27 output mask
    "mv x24, x26",        // configure all three pins as outputs
    "mv x23, x0",         // drive all three pins low initially
    "li x5, 0x08000000",  // BIO27: BCLK
    "li x6, 0x04000000",  // BIO26: LRCLK
    "li x10, 0x05000000", // clear BCLK only: PIN_MASK & !BCLK
    "li x11, 0x09000000", // clear LRCLK only: PIN_MASK & !LRCLK
    "li x7, 16",          // 16 BCLK periods per channel slot
    "li x9, 0",           // LRCLK state: start low
"10:",
    "mv x8, x7",
"20:",
    "mv x22, x5",         // BCLK high
    "mv x20, x0",          // wait one quantum
    "mv x23, x10",        // BCLK low, preserving DIN and LRCLK states
    "mv x20, x0",          // wait one quantum
    "addi x8, x8, -1",
    "bne x8, x0, 20b",
    "beq x9, x0, 30f",
    "mv x23, x11",        // LRCLK high -> low, preserving DIN and BCLK
    "li x9, 0",
    "j 10b",
"30:",
    "mv x22, x6",         // LRCLK low -> high
    "li x9, 1",
    "j 10b"
);
