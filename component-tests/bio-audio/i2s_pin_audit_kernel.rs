//! Read back Dabao BIO pins 24, 26 and 27 through the BIO GPIO input path.
//! FIFO2 receives four snapshots: all-low, DIN-high, LRCLK-high, BCLK-high.
use bao1x_api::bio_code;

#[rustfmt::skip]
bio_code!(i2s_pin_audit_kernel, I2S_PIN_AUDIT_START, I2S_PIN_AUDIT_END,
    "li x26, 0x0D000000", // BIO24, BIO26, BIO27
    "mv x24, x26",        // configure pins as outputs
    "mv x23, x0",         // drive all three low
    "li x5, 0x01000000",  // BIO24 / PC8 / DIN
    "li x6, 0x04000000",  // BIO26 / PC10 / LRCLK
    "li x7, 0x08000000",  // BIO27 / PC11 / BCLK
    "li x8, 0x0C000000",  // clear DIN only
    "li x9, 0x09000000",  // clear LRCLK only
    "li x10, 0x05000000", // clear BCLK only
    "mv x20, x0",         // allow GPIO pad state to settle

    "mv x4, x21",         // snapshot all-low
    "mv x18, x4",         // report through FIFO2

    "mv x22, x5",         // DIN high
    "mv x20, x0",
    "mv x4, x21",
    "mv x18, x4",
    "mv x23, x8",         // DIN low

    "mv x22, x6",         // LRCLK high
    "mv x20, x0",
    "mv x4, x21",
    "mv x18, x4",
    "mv x23, x9",         // LRCLK low

    "mv x22, x7",         // BCLK high
    "mv x20, x0",
    "mv x4, x21",
    "mv x18, x4",
    "mv x23, x10",        // BCLK low

    "mv x23, x0",         // leave all pins low
"10:",
    "j 10b"
);
