//! Hold one selected I2S BIO output high for external wiring verification.
//! FIFO1 receives exactly one mask: BIO24 (DIN), BIO26 (LRCLK), or BIO27 (BCLK).
use bao1x_api::bio_code;

#[rustfmt::skip]
bio_code!(i2s_line_hold_kernel, I2S_LINE_HOLD_START, I2S_LINE_HOLD_END,
    "li x26, 0x0D000000", // BIO24 + BIO26 + BIO27 are the permitted outputs
    "mv x24, x26",        // configure all three pins as outputs
    "mv x23, x0",         // begin with all three low
    "mv x4, x17",         // receive selected BIO output mask from FIFO1
    "mv x22, x4",         // drive only the selected line high
"10:",
    "j 10b"
);
