use crate::{CommonEnv, ShellCmdApi};
use bio_lib::i2s_clock::I2sClockTest;
use bio_lib::i2s_audio::I2sTone;
use bio_lib::i2s_pin_audit::I2sPinAudit;
use bio_lib::i2s_line_hold::I2sLineHold;
use core::fmt::Write;
use String;

pub struct I2sClockCmd {
    test: Option<I2sClockTest>,
    hold: Option<I2sLineHold>,
}

impl I2sClockCmd {
    pub fn new() -> Self {
        Self { test: None, hold: None }
    }
}

impl<'a> ShellCmdApi<'a> for I2sClockCmd {
    cmd_api!(i2sclock);

    fn process(
        &mut self,
        args: String,
        _env: &mut CommonEnv,
    ) -> Result<Option<String>, xous::Error> {
        let mut out = String::new();
        let mut args = args.split_whitespace();
        match args.next().unwrap_or("") {
            "start" => {
                if self.hold.is_some() {
                    out.push_str("Stop line hold first: i2sclock linehold stop");
                } else if self.test.is_some() {
                    out.push_str("I2S clock test is already running.");
                } else {
                    match I2sClockTest::start() {
                        Ok(test) => {
                            writeln!(
                                out,
                                "BIO clock test running: quantum={} Hz, BCLK={} Hz, LRCLK={} Hz; DIN held low.",
                                test.actual_quantum_hz(),
                                test.actual_bclk_hz(),
                                test.actual_lrclk_hz()
                            ).unwrap();
                            self.test = Some(test);
                        }
                        Err(error) => writeln!(out, "Could not start BIO clock test: {:?}", error).unwrap(),
                    }
                }
            }
            "stop" => {
                if args.next().is_some() {
                    out.push_str("Usage: i2sclock stop");
                } else if let Some(mut test) = self.test.take() {
                    match test.stop() {
                        Ok(()) => out.push_str("BIO clock test stopped; pins released."),
                        Err(error) => writeln!(out, "Stop/cleanup error: {:?}", error).unwrap(),
                    }
                } else {
                    out.push_str("I2S clock test is not running.");
                }
            }
            "status" => {
                if args.next().is_some() {
                    out.push_str("Usage: i2sclock status");
                } else if let Some(test) = self.test.as_ref() {
                    writeln!(
                        out,
                        "RUNNING: quantum={} Hz, BCLK={} Hz, LRCLK={} Hz; DIN low.",
                        test.actual_quantum_hz(),
                        test.actual_bclk_hz(),
                        test.actual_lrclk_hz()
                    ).unwrap();
                } else {
                    out.push_str("STOPPED");
                }
            }
            "tone440" => {
                if self.hold.is_some() {
                    out.push_str("Stop line hold first: i2sclock linehold stop");
                } else if self.test.is_some() {
                    out.push_str("Stop the clock-only test first: i2sclock stop");
                } else {
                    let requested_peak = match args.next().map(str::parse::<u16>) {
                        None => bio_lib::i2s_audio::DEFAULT_PEAK,
                        Some(Ok(value)) if [512, 1024, 2048, 4096].contains(&value) => value,
                        Some(Ok(_)) => {
                            out.push_str("Allowed amplitudes: 512, 1024, 2048, 4096 (maximum).");
                            return Ok(Some(out));
                        }
                        Some(Err(_)) => {
                            out.push_str("Usage: i2sclock tone440 [512|1024|2048|4096]");
                            return Ok(Some(out));
                        }
                    };
                    if args.next().is_some() {
                        out.push_str("Usage: i2sclock tone440 [512|1024|2048|4096]");
                        return Ok(Some(out));
                    }
                    match I2sTone::play_440(requested_peak) {
                        Ok(report) => {
                            writeln!(
                                out,
                                "Played 440 Hz tone: peak={} / 32767 ({}.{:01}%), quantum={} Hz, BCLK={} Hz, LRCLK={} Hz, frames={}.",
                                report.requested_peak,
                                (report.requested_peak as u32 * 1000 / 32767) / 10,
                                (report.requested_peak as u32 * 1000 / 32767) % 10,
                                report.quantum_hz,
                                report.bclk_hz,
                                report.lrclk_hz,
                                report.frames
                            ).unwrap();
                            out.push_str("BIO resources released.");
                        }
                        Err(error) => writeln!(out, "Audio test failed: {:?}", error).unwrap(),
                    }
                }
            }
            "pintest" => {
                if self.hold.is_some() {
                    out.push_str("Stop line hold first: i2sclock linehold stop");
                } else if self.test.is_some() {
                    out.push_str("Stop the clock-only test first: i2sclock stop");
                } else if args.next().is_some() {
                    out.push_str("Usage: i2sclock pintest");
                } else {
                    match I2sPinAudit::run() {
                        Ok(samples) => {
                            let expected = [0u32, 1 << 24, 1 << 26, 1 << 27];
                            let labels = ["all-low", "DIN-high", "LRCLK-high", "BCLK-high"];
                            let mut passed = true;
                            for i in 0..4 {
                                let ok = samples[i] == expected[i];
                                passed &= ok;
                                writeln!(
                                    out,
                                    "{}: readback=0x{:08X}, expected=0x{:08X} {}",
                                    labels[i], samples[i], expected[i],
                                    if ok { "PASS" } else { "FAIL" }
                                ).unwrap();
                            }
                            writeln!(out, "Pin readback audit: {}", if passed { "4/4 PASS" } else { "FAILED" }).unwrap();
                        }
                        Err(error) => writeln!(out, "Pin audit failed: {:?}", error).unwrap(),
                    }
                }
            }
            "linehold" => {
                let target = args.next().unwrap_or("");
                match target {
                    "din" | "lrclk" | "bclk" => {
                        if self.test.is_some() || self.hold.is_some() {
                            out.push_str("Stop any running I2S test first.");
                        } else if args.next().is_some() {
                            out.push_str("Usage: i2sclock linehold din|lrclk|bclk|stop");
                        } else {
                            let (mask, label) = match target {
                                "din" => (1u32 << 24, "DIN / PC8 / BIO24"),
                                "lrclk" => (1u32 << 26, "LRCLK / PC10 / BIO26"),
                                _ => (1u32 << 27, "BCLK / PC11 / BIO27"),
                            };
                            match I2sLineHold::start(mask) {
                                Ok(hold) => {
                                    writeln!(
                                        out,
                                        "Holding {} HIGH. Measure the loose far end of that jumper against Dabao GND. Then run i2sclock linehold stop.",
                                        label
                                    ).unwrap();
                                    self.hold = Some(hold);
                                }
                                Err(error) => writeln!(out, "Could not start line hold: {:?}", error).unwrap(),
                            }
                        }
                    }
                    "stop" => {
                        if args.next().is_some() {
                            out.push_str("Usage: i2sclock linehold din|lrclk|bclk|stop");
                        } else if let Some(mut hold) = self.hold.take() {
                            match hold.stop() {
                                Ok(()) => out.push_str("Line hold stopped; pins released."),
                                Err(error) => writeln!(out, "Line hold cleanup error: {:?}", error).unwrap(),
                            }
                        } else {
                            out.push_str("No line hold is active.");
                        }
                    }
                    _ => out.push_str("Usage: i2sclock linehold din|lrclk|bclk|stop"),
                }
            }
            _ => out.push_str("i2sclock start | status | stop | pintest | linehold din|lrclk|bclk|stop | tone440 [512|1024|2048|4096]"),
        }
        Ok(Some(out))
    }
}
