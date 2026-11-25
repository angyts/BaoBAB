use crate::{CommonEnv, ShellCmdApi};
use bio_lib::i2s_clock::I2sClockTest;
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
            _ => out.push_str("i2sclock start | status | stop"),
        }
        Ok(Some(out))
    }
}
