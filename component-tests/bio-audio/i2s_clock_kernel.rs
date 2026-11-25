//! Minimal BIO-generated I2S clock test for Dabao.
//!
//! This is deliberately a clock-only phase: DIN stays low (digital silence).
//! It does NOT play the 440 Hz tone yet.

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
/// DIN (BIO24 / PC8) remains low. This is standard digital silence,
/// not an audio tone. The core is stopped by the host-side I2S clock
/// test object; do not leave it running after the test.
#[rustfmt::skip]
bio_code!(i2s_clock_kernel, I2S_CLOCK_START, I2S_CLOCK_END,
    "li x26, 0x0D000000", // BIO24 + BIO26 + BIO27 output mask
    "mv x24, x26",        // configure all three pins as outputs
    "mv x23, x0",         // drive all three pins low initially
    "li x5, 0x08000000",  // BIO27: BCLK
    "li x6, 0x04000000",  // BIO26: LRCLK
    "li x7, 16",          // 16 BCLK periods per channel slot
    "li x9, 0",           // LRCLK state: start low
"10:",
    "mv x8, x7",
"20:",
    "mv x22, x5",         // BCLK high
    "mv x20, x0",          // wait one quantum
    "mv x23, x5",         // BCLK low
    "mv x20, x0",          // wait one quantum
    "addi x8, x8, -1",
    "bne x8, x0, 20b",
    "beq x9, x0, 30f",
    "mv x23, x6",         // LRCLK high -> low
    "li x9, 0",
    "j 10b",
"30:",
    "mv x22, x6",         // LRCLK low -> high
    "li x9, 1",
    "j 10b"
);
