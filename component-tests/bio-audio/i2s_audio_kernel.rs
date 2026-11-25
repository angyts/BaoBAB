//! BIO I2S transmitter for MAX98357A.
//!
//! FIFO1 format: first word is frame_count, then one u16 sample word for
//! left and one u16 sample word for right for each frame.
//! 32-bit I2S slots are used; samples are 16-bit, MSB-aligned with the
//! standard one-BCLK I2S delay. After the one-bit delay, the sample occupies bits 30..15; the unused low 15 bits are zero.
//!
//! BIO24/PC8 = DIN, BIO26/PC10 = LRCLK, BIO27/PC11 = BCLK.

use bao1x_api::bio_code;

pub const BIO_DIN: u8 = 24;
pub const BIO_LRCLK: u8 = 26;
pub const BIO_BCLK: u8 = 27;
pub const PIN_MASK: u32 = (1 << BIO_DIN) | (1 << BIO_LRCLK) | (1 << BIO_BCLK);
pub const TARGET_QUANTUM_HZ: u32 = 6_144_000;

/// 2 quantum ticks per BCLK period, 64 BCLK periods per stereo frame.
/// Therefore BCLK ~= 3.072 MHz and LRCLK ~= 48 kHz.
/// FIFO1 reads block while the host refills the queue. The clock pauses only
/// at the frame boundary, before the next sample pair is fetched.
#[rustfmt::skip]
bio_code!(i2s_audio_kernel, I2S_AUDIO_START, I2S_AUDIO_END,
    "li x26, 0x0D000000", // BIO24 + BIO26 + BIO27 output mask
    "mv x24, x26",        // configure the three BIO pins as outputs
    "mv x23, x0",         // start with DIN, LRCLK and BCLK low
    "li x5, 0x08000000",  // BCLK set mask
    "li x6, 0x04000000",  // LRCLK set mask
    "li x7, 0x01000000",  // DIN set mask
    "li x8, 0x0C000000",  // clear DIN only
    "li x9, 0x05000000",  // clear BCLK only
    "li x11, 0x09000000", // clear LRCLK only
    "li x3, 0x40000000",  // sample MSB after 1-bit I2S delay (bit 30)
    "mv x12, x17",        // frame count from FIFO1

"10:",                    // next stereo frame; fetch both samples before clocks
    "mv x4, x17",          // left sample (unsigned 16-bit payload)
    "mv x13, x17",         // right sample (unsigned 16-bit payload)
    "mv x23, x11",         // LRCLK low: MAX98357A left channel
    "mv x23, x8",          // DIN low during the mandatory I2S delay bit
    "mv x22, x5",          // delay-bit BCLK rising edge
    "mv x20, x0",
    "mv x23, x9",          // delay-bit BCLK falling edge
    "mv x20, x0",
    "slli x4, x4, 15",     // next 16 bits are the signed sample MSBs
    "li x14, 31",          // remaining 31 clocks: 16 data + 15 zero padding

"20:",                    // left slot
    "and x15, x4, x3",
    "beq x15, x0, 21f",
    "mv x22, x7",          // DIN high for a one
    "j 22f",
"21:",
    "mv x23, x8",          // DIN low for a zero
"22:",
    "mv x20, x0",           // wait one quantum with data stable
    "mv x22, x5",           // BCLK rising edge: MAX98357A samples here
    "mv x20, x0",           // hold BCLK high for one quantum
    "mv x23, x9",           // BCLK falling edge
    "slli x4, x4, 1",
    "addi x14, x14, -1",
    "bne x14, x0, 20b",

    "mv x22, x6",           // LRCLK high: right channel
    "mv x23, x8",          // DIN low during the mandatory I2S delay bit
    "mv x22, x5",          // delay-bit BCLK rising edge
    "mv x20, x0",
    "mv x23, x9",          // delay-bit BCLK falling edge
    "mv x20, x0",
    "mv x4, x13",
    "slli x4, x4, 15",
    "li x14, 31",

"30:",                    // right slot
    "and x15, x4, x3",
    "beq x15, x0, 31f",
    "mv x22, x7",
    "j 32f",
"31:",
    "mv x23, x8",
"32:",
    "mv x20, x0",
    "mv x22, x5",
    "mv x20, x0",
    "mv x23, x9",
    "slli x4, x4, 1",
    "addi x14, x14, -1",
    "bne x14, x0, 30b",

    "addi x12, x12, -1",
    "bne x12, x0, 10b",

    // Stop at the end of a complete LRCLK frame, with all outputs low.
    "mv x23, x0",
    "mv x18, x31",          // completion token to host through FIFO2
"40:",
    "j 40b"
);
