use crate::{CommonEnv, ShellCmdApi};
use bio_lib::i2s_clock::I2sClockTest;
use bio_lib::i2s_audio::I2sTone;
use bio_lib::i2s_pin_audit::I2sPinAudit;
use core::fmt::Write;
use String;

pub struct I2sClockCmd {
    test: Option<I2sClockTest>,
}

impl I2sClockCmd {
    pub fn new() -> Self {
        Self { test: None }
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
        match args.trim() {
            "start" => {
                if self.test.is_some() {
                    out.push_str("I2S clock test is already running.");
                } else {
                    match I2sClockTest::start() {
                        Ok(test) => {
                            writeln!(
                                out,
                                "BIO clock test running: quantum={} Hz, BCLK={} Hz, LRCLK={} Hz; DIN held low (digital silence).",
                                test.actual_quantum_hz(),
                                test.actual_bclk_hz(),
                                test.actual_lrclk_hz()
                            )
                            .unwrap();
                            self.test = Some(test);
                        }
                        Err(error) => {
                            writeln!(out, "Could not start BIO clock test: {:?}", error).unwrap();
                        }
                    }
                }
            }
            "stop" => {
                if let Some(mut test) = self.test.take() {
                    match test.stop() {
                        Ok(()) => out.push_str("BIO clock test stopped; pins released."),
                        Err(error) => writeln!(out, "Stop/cleanup error: {:?}", error).unwrap(),
                    }
                } else {
                    out.push_str("I2S clock test is not running.");
                }
            }
            "status" => {
                if let Some(test) = self.test.as_ref() {
                    writeln!(
                        out,
                        "RUNNING: quantum={} Hz, BCLK={} Hz, LRCLK={} Hz; DIN low.",
                        test.actual_quantum_hz(),
                        test.actual_bclk_hz(),
                        test.actual_lrclk_hz()
                    )
                    .unwrap();
                } else {
                    out.push_str("STOPPED");
                }
            }
            "tone440" => {
                if self.test.is_some() {
                    out.push_str("Stop the clock-only test first: i2sclock stop");
                } else {
                    match I2sTone::play_440() {
                        Ok(report) => {
                            writeln!(
                                out,
                                "Played 440 Hz test tone: quantum={} Hz, BCLK={} Hz, LRCLK={} Hz, frames={}, peak_sample={}.",
                                report.quantum_hz,
                                report.bclk_hz,
                                report.lrclk_hz,
                                report.frames,
                                report.peak_sample
                            )
                            .unwrap();
                            out.push_str("BIO resources released.");
                        }
                        Err(error) => {
                            writeln!(out, "Audio test failed: {:?}", error).unwrap();
                        }
                    }
                }
            },
            "pintest" => {
                if self.test.is_some() {
                    out.push_str("Stop the clock-only test first: i2sclock stop");
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
                                    labels[i],
                                    samples[i],
                                    expected[i],
                                    if ok { "PASS" } else { "FAIL" }
                                ).unwrap();
                            }
                            writeln!(out, "Pin readback audit: {}", if passed { "4/4 PASS" } else { "FAILED" }).unwrap();
                        }
                        Err(error) => {
                            writeln!(out, "Pin audit failed: {:?}", error).unwrap();
                        }
                    }
                }
            }
            _ => out.push_str("i2sclock start | status | stop | pintest | tone440"),
        }
        Ok(Some(out))
    }
}
