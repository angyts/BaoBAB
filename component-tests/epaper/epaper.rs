use crate::{CommonEnv, ShellCmdApi};
use String;

use bao1x_api::{
    IoSetup, IoxDir, IoxDriveStrength, IoxEnable, IoxFunction, IoxHal, IoxPort, IoxValue,
    PeriphId, SpimChannel,
};
use bao1x_hal::udma::{Spim, SpimClkPha, SpimClkPol, SpimCs};
use bao1x_hal_service::UdmaGlobal;
use std::time::{Duration, Instant};

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
        while spi.is_tx_busy() {
            if start.elapsed() > TX_TIMEOUT {
                return Err("SPI TX timed out");
            }
            xous::yield_slice();
        }
        Ok(())
    }

    fn tx(spi: &mut Spim, data: &[u8]) -> Result<(), &'static str> {
        spi.tx_data_async(data, true, false);
        Self::wait_tx(spi)
    }

    fn command(spi: &mut Spim, iox: &IoxHal, command: u8) -> Result<(), &'static str> {
        iox.set_gpio_pin_value(IoxPort::PB, DC_PIN, IoxValue::Low);
        Self::tx(spi, &[command])
    }

    fn data(spi: &mut Spim, iox: &IoxHal, data: &[u8]) -> Result<(), &'static str> {
        iox.set_gpio_pin_value(IoxPort::PB, DC_PIN, IoxValue::High);
        Self::tx(spi, data)
    }

    fn wait_idle(iox: &IoxHal) -> Result<(), &'static str> {
        let start = Instant::now();
        while iox.get_gpio_pin_value(IoxPort::PB, BUSY_PIN) == IoxValue::High {
            if start.elapsed() > BUSY_TIMEOUT {
                return Err("e-Paper BUSY remained HIGH for 30 seconds");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    fn reset(iox: &IoxHal) {
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

    fn run(&self, env: &mut CommonEnv, action: &str) -> Result<String, &'static str> {
        let iox = IoxHal::new();
        Self::setup_pins(&iox);
        std::thread::sleep(Duration::from_millis(100));

        // A hardware reset is GPIO-only. Do not allocate a DMA/IFRAM buffer
        // for this command; it must remain usable even when SPI is not ready.
        if action == "reset" {
            Self::reset(&iox);
            return Ok("e-Paper hardware reset completed.".to_string());
        }

        let udma = UdmaGlobal::new();
        udma.udma_clock_config(PeriphId::Spim2, true);
        udma.reset(PeriphId::Spim2);

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
        udma.udma_clock_config(PeriphId::Spim2, false);
        result
    }
}

impl<'a> ShellCmdApi<'a> for EpaperCmd {
    cmd_api!(epaper);

    fn process(&mut self, args: String, env: &mut CommonEnv) -> Result<Option<String>, xous::Error> {
        let mut tokens = args.split_whitespace();
        match tokens.next().unwrap_or("") {
            "info" => {
                Ok(Some(format!(
                    "Waveshare 2.13-inch V4; SPI2 mode 0 at {} Hz; frame={} bytes; HAT pins DIN19/SCK23/CS24/DC22/RST11/BUSY18/PWR12; RTC disconnected.",
                    SPI_HZ, FRAME_BYTES
                )))
            }
            "reset" | "init" | "white" | "black" => {
                let action = args.split_whitespace().next().unwrap_or("");
                if tokens.next().is_some() {
                    return Ok(Some("Usage: epaper info|reset|init|white|black".to_string()));
                }
                match self.run(env, action) {
                    Ok(message) => Ok(Some(message)),
                    Err(error) => Ok(Some(format!("e-Paper error: {}", error))),
                }
            }
            _ => Ok(Some("Usage: epaper info | reset | init | white | black".to_string())),
        }
    }
}
