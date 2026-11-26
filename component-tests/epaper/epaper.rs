use crate::{CommonEnv, ShellCmdApi};
use String;

use bao1x_api::{
    HalOpcode, IoSetup, IoxDir, IoxDriveStrength, IoxEnable, IoxFunction, IoxHal, IoxPort,
    IoxValue, PeriphId, SpimChannel, SERVER_NAME_BAO1X_HAL,
};
use bao1x_hal::udma::{Spim, SpimClkPha, SpimClkPol, SpimCs};
use core::sync::atomic::{AtomicBool, Ordering};
use num_traits::ToPrimitive;
use std::time::{Duration, Instant};

/// Process-wide verbosity switch for the BaoBAB diagnostic command.
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

fn set_debug(enabled: bool) {
    // Do not change the process-wide log filter: that would enable noisy
    // Trace/Debug messages from the shell (including every keypress).
    DEBUG_ENABLED.store(enabled, Ordering::Relaxed);
    log::info!("BaoBAB e-Paper debug mode {}", if enabled { "ENABLED" } else { "DISABLED" });
}

macro_rules! ep_debug {
    ($($arg:tt)*) => {{
        if DEBUG_ENABLED.load(Ordering::Relaxed) {
            log::info!("[EPAPER DEBUG] {}", format_args!($($arg)*));
        }
    }};
}

macro_rules! ep_trace {
    ($($arg:tt)*) => {{
        if DEBUG_ENABLED.load(Ordering::Relaxed) {
            log::info!("[EPAPER TRACE] {}", format_args!($($arg)*));
        }
    }};
}

const WIDTH: usize = 122;
const HEIGHT: usize = 250;
const STRIDE: usize = (WIDTH + 7) / 8;
const FRAME_BYTES: usize = STRIDE * HEIGHT;

const SPI_HZ: u32 = 1_000_000;
// The existing Baochip graphics backend configures SPIM with a 50 MHz
// peripheral clock. Keep this conservative until the actual Dabao clock
// tree for SPIM2 has been measured/confirmed.
const SPIM_CLOCK_HZ: u32 = 50_000_000;

const DC_PIN: u8 = 2;   // PB2 / Dabao header 31 / HAT pin 22
const RST_PIN: u8 = 3;  // PB3 / Dabao header 32 / HAT pin 11
const BUSY_PIN: u8 = 4; // PB4 / Dabao header 34 / HAT pin 18
const PWR_PIN: u8 = 5; // PB5 / Dabao header 35 / HAT pin 12 (BCM GPIO18)

const BUSY_TIMEOUT: Duration = Duration::from_secs(30);
const TX_TIMEOUT: Duration = Duration::from_secs(5);

pub struct EpaperCmd;

impl EpaperCmd {
    pub fn new() -> Self { Self }

    fn setup_pins(iox: &IoxHal) {
        log::info!("EPAPER: configuring SPI2 and panel GPIO pins");
        // SPI2 alternate-function B pins: PC0=CLK, PC1=MOSI, PC3=CS0.
        for pin in [0, 1, 3] {
            iox.setup_pin(
                IoxPort::PC, pin, Some(IoxDir::Output), Some(IoxFunction::AF2),
                None, None, Some(IoxEnable::Enable), Some(IoxDriveStrength::Drive4mA),
            );
        }
        iox.setup_pin(
            IoxPort::PB, DC_PIN, Some(IoxDir::Output), Some(IoxFunction::Gpio),
            None, None, Some(IoxEnable::Enable), Some(IoxDriveStrength::Drive2mA),
        );
        iox.setup_pin(
            IoxPort::PB, RST_PIN, Some(IoxDir::Output), Some(IoxFunction::Gpio),
            None, None, Some(IoxEnable::Enable), Some(IoxDriveStrength::Drive2mA),
        );
        iox.setup_pin(
            IoxPort::PB, BUSY_PIN, Some(IoxDir::Input), Some(IoxFunction::Gpio),
            Some(IoxEnable::Enable), None, None, None,
        );
        iox.setup_pin(
            IoxPort::PB, PWR_PIN, Some(IoxDir::Output), Some(IoxFunction::Gpio),
            None, None, Some(IoxEnable::Enable), Some(IoxDriveStrength::Drive2mA),
        );
        // Waveshare's own Raspberry Pi driver asserts PWR before opening SPI.
        iox.set_gpio_pin_value(IoxPort::PB, PWR_PIN, IoxValue::High);
        iox.set_gpio_pin_value(IoxPort::PB, DC_PIN, IoxValue::High);
        iox.set_gpio_pin_value(IoxPort::PB, RST_PIN, IoxValue::High);
    }

    fn wait_tx(spi: &Spim) -> Result<(), &'static str> {
        let start = Instant::now();
        ep_trace!("EPAPER: waiting for UDMA TX completion");
        while spi.is_tx_busy() {
            if start.elapsed() > TX_TIMEOUT {
                return Err("SPI TX timed out");
            }
            xous::yield_slice();
        }
        Ok(())
    }

    fn tx(spi: &mut Spim, data: &[u8]) -> Result<(), &'static str> {
        ep_debug!("EPAPER: SPI TX {} bytes", data.len());
        let started = Instant::now();
        spi.tx_data_async(data, true, false);
        let result = Self::wait_tx(spi);
        ep_debug!("EPAPER: SPI TX result={:?}, elapsed={:?}", result, started.elapsed());
        result
    }

    fn command(spi: &mut Spim, iox: &IoxHal, command: u8) -> Result<(), &'static str> {
        ep_trace!("EPAPER: command 0x{:02x}", command);
        iox.set_gpio_pin_value(IoxPort::PB, DC_PIN, IoxValue::Low);
        Self::tx(spi, &[command])
    }

    fn data(spi: &mut Spim, iox: &IoxHal, data: &[u8]) -> Result<(), &'static str> {
        ep_trace!("EPAPER: data transfer {} bytes", data.len());
        iox.set_gpio_pin_value(IoxPort::PB, DC_PIN, IoxValue::High);
        Self::tx(spi, data)
    }

    fn wait_idle(iox: &IoxHal) -> Result<(), &'static str> {
        let start = Instant::now();
        ep_debug!("EPAPER: waiting for BUSY=LOW (timeout {:?})", BUSY_TIMEOUT);
        while iox.get_gpio_pin_value(IoxPort::PB, BUSY_PIN) == IoxValue::High {
            if start.elapsed() > BUSY_TIMEOUT {
                log::error!("EPAPER: BUSY timeout after {:?}", start.elapsed());
                return Err("e-Paper BUSY remained HIGH for 30 seconds");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    fn reset(iox: &IoxHal) {
        log::info!("EPAPER: GPIO hardware reset sequence");
        iox.set_gpio_pin_value(IoxPort::PB, RST_PIN, IoxValue::High);
        std::thread::sleep(Duration::from_millis(20));
        iox.set_gpio_pin_value(IoxPort::PB, RST_PIN, IoxValue::Low);
        std::thread::sleep(Duration::from_millis(2));
        iox.set_gpio_pin_value(IoxPort::PB, RST_PIN, IoxValue::High);
        std::thread::sleep(Duration::from_millis(20));
    }

    fn initialise(spi: &mut Spim, iox: &IoxHal) -> Result<(), &'static str> {
        Self::reset(iox);
        Self::wait_idle(iox)?;
        Self::command(spi, iox, 0x12)?; // software reset
        Self::wait_idle(iox)?;

        Self::command(spi, iox, 0x01)?; // driver output control
        Self::data(spi, iox, &[0xF9, 0x00, 0x00])?;

        Self::command(spi, iox, 0x11)?; // data entry mode
        Self::data(spi, iox, &[0x03])?;

        Self::command(spi, iox, 0x44)?; // X RAM window
        Self::data(spi, iox, &[0x00, 0x0F])?;

        Self::command(spi, iox, 0x45)?; // Y RAM window
        Self::data(spi, iox, &[0x00, 0x00, 0xF9, 0x00])?;

        Self::command(spi, iox, 0x4E)?; // X cursor
        Self::data(spi, iox, &[0x00])?;
        Self::command(spi, iox, 0x4F)?; // Y cursor
        Self::data(spi, iox, &[0x00, 0x00])?;

        Self::command(spi, iox, 0x3C)?; // border waveform
        Self::data(spi, iox, &[0x05])?;
        Self::command(spi, iox, 0x21)?; // display update control
        Self::data(spi, iox, &[0x00, 0x80])?;
        Self::command(spi, iox, 0x18)?; // internal temperature sensor control
        Self::data(spi, iox, &[0x80])?;
        Self::wait_idle(iox)
    }

    fn refresh(spi: &mut Spim, iox: &IoxHal, colour: u8) -> Result<(), &'static str> {
        let frame = [colour; FRAME_BYTES];
        Self::command(spi, iox, 0x24)?; // write image RAM
        Self::data(spi, iox, &frame)?;
        Self::command(spi, iox, 0x22)?;
        Self::data(spi, iox, &[0xF7])?;
        Self::command(spi, iox, 0x20)?;
        Self::wait_idle(iox)
    }

    fn hal_connect() -> Result<xous::CID, String> {
        let xns = xous_names::XousNames::new()
            .map_err(|e| format!("Xous names service initialization failed: {:?}", e))?;
        xns.request_connection(SERVER_NAME_BAO1X_HAL)
            .map_err(|e| format!("HAL service lookup failed: {:?}", e))
    }

    /// Raw blocking scalar request: report IPC failures instead of using panic-on-error wrappers.
    /// Clock/reset acknowledgements prove IPC delivery only, not physical peripheral health.
    fn hal_scalar(cid: xous::CID, opcode: HalOpcode, arg1: usize, arg2: usize) -> Result<u32, String> {
        let op = opcode.to_usize().ok_or_else(|| format!("Cannot encode HAL opcode {:?}", opcode))?;
        ep_debug!("HAL request {:?}: arg1={} arg2={}", opcode, arg1, arg2);
        let started = Instant::now();
        match xous::send_message(cid, xous::Message::new_blocking_scalar(op, arg1, arg2, 0, 0)) {
            Ok(xous::Result::Scalar5(_, value, _, _, _)) => {
                ep_debug!("HAL reply {:?}: value=0x{:08x}, elapsed={:?}", opcode, value, started.elapsed());
                Ok(value as u32)
            }
            Ok(other) => Err(format!("HAL {:?} returned unexpected reply: {:?}", opcode, other)),
            Err(error) => Err(format!("HAL {:?} IPC failed after {:?}: {:?}", opcode, started.elapsed(), error)),
        }
    }

    fn diagnostics(&self) -> Result<String, String> {
        let mut report = Vec::new();
        log::info!("EPAPER-DIAG: starting non-refresh diagnostics");
        report.push("BaoBAB e-Paper diagnostics (no panel refresh or GPIO writes)".to_string());
        let cid = Self::hal_connect()?;
        report.push("PASS: Baochip HAL service connection".to_string());

        for (name, port) in [("PB", IoxPort::PB), ("PC", IoxPort::PC)] {
            match Self::hal_scalar(cid, HalOpcode::GetGpioBank, port as usize, 0) {
                Ok(bank) => {
                    report.push(format!("PASS: GPIO {} bank read = 0x{:04x}", name, bank));
                    if name == "PB" {
                        report.push(format!("INFO: PB2/DC={} PB3/RST={} PB4/BUSY={} PB5/PWR={}",
                            (bank >> DC_PIN) & 1, (bank >> RST_PIN) & 1,
                            (bank >> BUSY_PIN) & 1, (bank >> PWR_PIN) & 1));
                    }
                }
                Err(e) => report.push(format!("FAIL: GPIO {} bank read: {}", name, e)),
            }
        }

        if let Err(e) = Self::hal_scalar(cid, HalOpcode::ConfigureUdmaClock, PeriphId::Spim2 as usize, 1) {
            report.push(format!("FAIL: SPIM2 clock-enable IPC: {}", e));
            return Ok(report.join("\n"));
        }
        report.push("PASS: SPIM2 clock-enable IPC acknowledgement".to_string());

        match Self::hal_scalar(cid, HalOpcode::PeriphReset, PeriphId::Spim2 as usize, 0) {
            Ok(_) => report.push("PASS: SPIM2 reset IPC acknowledgement".to_string()),
            Err(e) => report.push(format!("FAIL: SPIM2 reset IPC: {}", e)),
        }

        // Allocate and release the IFRAM-backed SPIM resources, but transmit no bytes.
        let allocation = unsafe {
            Spim::new(
                SpimChannel::Channel2, SPI_HZ, SPIM_CLOCK_HZ,
                SpimClkPol::LeadingEdgeRise, SpimClkPha::CaptureOnLeading,
                SpimCs::Cs0, 2, 2, None, FRAME_BYTES, 0, None, None,
            )
        };
        match allocation {
            Some(spi) => {
                report.push("PASS: SPIM2 object / DMA buffer allocation".to_string());
                drop(spi);
                report.push("PASS: SPIM2 object drop requested (IFRAM cleanup path exercised)".to_string());
            }
            None => report.push("FAIL: SPIM2 object / DMA buffer allocation returned None".to_string()),
        }

        xous::yield_slice();
        // IframRange::drop() may disconnect the shared HAL CID when its final
        // reference is released. Never reuse the pre-allocation CID after drop.
        match Self::hal_connect().and_then(|fresh_cid| {
            Self::hal_scalar(fresh_cid, HalOpcode::ConfigureUdmaClock, PeriphId::Spim2 as usize, 0)
        }) {
            Ok(_) => report.push("PASS: SPIM2 clock-disable IPC acknowledgement (fresh HAL connection)".to_string()),
            Err(e) => report.push(format!("FAIL: SPIM2 clock-disable IPC: {}", e)),
        }
        log::info!("EPAPER-DIAG: completed with {} report lines", report.len());
        Ok(report.join("\n"))
    }

    fn run(&self, env: &mut CommonEnv, action: &str) -> Result<String, &'static str> {
        let started = Instant::now();
        log::info!("EPAPER: action '{}' starting", action);
        let iox = IoxHal::new();
        Self::setup_pins(&iox);
        std::thread::sleep(Duration::from_millis(100));

        // A hardware reset is GPIO-only. Do not allocate a DMA/IFRAM buffer
        // for this command; it must remain usable even when SPI is not ready.
        if action == "reset" {
            Self::reset(&iox);
            return Ok("e-Paper hardware reset completed.".to_string());
        }

        let hal = Self::hal_connect().map_err(|_| "Could not connect to Baochip HAL")?;
        Self::hal_scalar(hal, HalOpcode::ConfigureUdmaClock, PeriphId::Spim2 as usize, 1)
            .map_err(|_| "Could not enable SPIM2 clock")?;
        Self::hal_scalar(hal, HalOpcode::PeriphReset, PeriphId::Spim2 as usize, 0)
            .map_err(|_| "Could not reset SPIM2")?;

        // Safety: SPIM2 clock is enabled above. The SPIM object owns a
        // page-backed DMA/command buffer and is dropped before the clock gate
        // is disabled below.
        let result = (|| -> Result<String, &'static str> {
            let mut spi = unsafe {
                Spim::new(
                    SpimChannel::Channel2,
                    SPI_HZ,
                    SPIM_CLOCK_HZ,
                    SpimClkPol::LeadingEdgeRise,
                    SpimClkPha::CaptureOnLeading,
                    SpimCs::Cs0,
                    2,
                    2,
                    None,
                    FRAME_BYTES,
                    0,
                    None,
                    None,
                )
            }.ok_or("Could not allocate SPIM DMA buffer")?;

            let result = match action {
                "init" => {
                    Self::initialise(&mut spi, &iox)?;
                    Ok("Waveshare V4 controller initialised; BUSY is LOW.".to_string())
                }
                "white" | "black" => {
                    Self::initialise(&mut spi, &iox)?;
                    Self::refresh(&mut spi, &iox, if action == "white" { 0xFF } else { 0x00 })?;
                    Ok(format!("Waveshare V4 full-screen {} refresh completed.", action))
                }
                _ => Err("unknown action"),
            };
            drop(spi);
            result
        })();

        let _ = env.ticktimer.sleep_ms(1);
        // Dropping Spim drops IframRange, which can disconnect its shared HAL CID.
        // Re-resolve the service before the cleanup request instead of reusing it.
        let clock_off = Self::hal_connect().and_then(|fresh_hal| {
            Self::hal_scalar(fresh_hal, HalOpcode::ConfigureUdmaClock, PeriphId::Spim2 as usize, 0)
        });
        if let Err(error) = clock_off {
            log::error!("EPAPER: failed to disable SPIM2 clock: {}", error);
        }
        log::info!("EPAPER: action '{}' finished success={} elapsed={:?}", action, result.is_ok(), started.elapsed());
        result
    }
}

impl<'a> ShellCmdApi<'a> for EpaperCmd {
    cmd_api!(epaper);

    fn process(&mut self, args: String, env: &mut CommonEnv) -> Result<Option<String>, xous::Error> {
        let mut tokens = args.split_whitespace();
        match tokens.next().unwrap_or("") {
            "debug" => {
                match tokens.next().unwrap_or("status") {
                    "on" => { set_debug(true); Ok(Some("Debug enabled globally for this Xous process (Trace log ceiling).".to_string())) }
                    "off" => { set_debug(false); Ok(Some("Debug disabled (Info log ceiling).".to_string())) }
                    "status" => Ok(Some(format!("Debug is {}.", if DEBUG_ENABLED.load(Ordering::Relaxed) { "ON" } else { "OFF" }))),
                    _ => Ok(Some("Usage: epaper debug on|off|status".to_string())),
                }
            }
            "diag" => {
                if tokens.next().is_some() { return Ok(Some("Usage: epaper diag".to_string())); }
                match self.diagnostics() {
                    Ok(report) => Ok(Some(report)),
                    Err(error) => Ok(Some(format!("e-Paper diagnostic aborted: {}", error))),
                }
            }
            "info" => {
                Ok(Some(format!(
                    "Waveshare 2.13-inch V4; SPI2 mode 0 at {} Hz; frame={} bytes; HAT pins DIN19/SCK23/CS24/DC22/RST11/BUSY18/PWR12; RTC disconnected.",
                    SPI_HZ, FRAME_BYTES
                )))
            }
            "reset" | "init" | "white" | "black" => {
                let action = args.split_whitespace().next().unwrap_or("");
                if tokens.next().is_some() {
                    return Ok(Some("Usage: epaper info|debug on|off|status|diag|reset|init|white|black".to_string()));
                }
                match self.run(env, action) {
                    Ok(message) => Ok(Some(message)),
                    Err(error) => Ok(Some(format!("e-Paper error: {}", error))),
                }
            }
            _ => Ok(Some("Usage: epaper info | debug on|off|status | diag | reset | init | white | black".to_string())),
        }
    }
}
