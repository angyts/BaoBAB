# Waveshare 2.13-inch e-Paper HAT+ V4 — Dabao component test

**Hardware:** Baochip-1x Dabao + Waveshare 2.13-inch e-Paper HAT+ marked V4 / LSK4.  
**Current stage:** wired and powered; display firmware test has not yet been installed or run. RTC SDA/SCL are disconnected while display SPI is tested.

## Goal

Bring up the display independently of the RTC. First milestones are:
1. Confirm the expected wiring and supply.
2. Configure Dabao SPI2 and three GPIO controls.
3. Reset and initialise the V4 controller.
4. Write a full white frame and refresh.
5. Write a full black frame and refresh.
6. Add drawing/text only after the basic refresh works.

## Wiring currently in use

HAT pin numbers are Raspberry Pi 40-pin **physical header numbers**. Dabao pin numbers are Dabao board **physical header numbers**. They are not interchangeable.

### Display signals

| Function | Baochip pad / peripheral signal | Dabao header | HAT physical pin |
|---|---|---:|---:|
| MOSI / EPD DIN | PC1 / SPIM_SD0_B[2] | 12 | 19 |
| SPI clock / EPD SCK | PC0 / SPIM_CLK_B[2] | 14 | 23 |
| SPI CS | PC3 / SPIM_CSN0_B[2] | 10 | 24 |
| DC | PB2 / GPIO | 31 | 22 |
| RESET | PB3 / GPIO | 32 | 11 |
| BUSY | PB4 / GPIO input | 34 | 18 |
| Panel power enable (PWR) | PB5 / GPIO output, HIGH to enable | 35 | 12 (BCM GPIO18) |

The Waveshare Raspberry Pi driver uses BCM GPIO8 for CS, which is physical header pin 24. Physical pin 21 is MISO and is not used by this display. MISO is not needed for this display.

### Power already wired

| Function | Dabao header | HAT physical pin |
|---|---:|---:|
| 3.3 V | 36 | 1 |
| 5 V VBUS | 40 | 2 |
| GND | 3 | 6 |

The user measured HAT pin 1 at 3.3 V. HAT pin 2 / Dabao VBUS was not independently measured because access to the HAT contact was difficult. Do not probe adjacent header pins with a broad meter probe. Confirm the 5 V rail from an accessible Dabao VBUS-to-GND point if needed.

**Do not bridge the 3.3 V and 5 V rails.** The HAT schematic shows separate 3V3 and 5V header nets and an onboard regulator for the panel power rail; verify the exact revision's power path before changing the wiring.

### RTC intentionally disconnected

- HAT pin 3 SDA -> Dabao PB12 / header 26.
- HAT pin 5 SCL -> Dabao PB11 / header 27.
- DS3231 address is normally 0x68.

Keep both RTC wires disconnected until the display has passed its SPI test. The earlier I2C probe caused a console hang, so do not use the RTC to diagnose the display.

## Pin mapping basis

The Dabao pinout and Baochip electrical-specification table identify:
- PC0 -> SPIM_CLK_B[2].
- PC1 -> SPIM_SD0_B[2] (MOSI).
- PC2 -> SPIM_SD1_B[2] (MISO).
- PC3 -> SPIM_CSN0_B[2].

Therefore use **SPIM channel 2**. In the Xous IOX mux configuration, these SPIM_B[2] signals use AF2. Do not confuse the HAT's RPi header pin numbering with Dabao header numbering.

GPIO assignments:
- PB2 = Dabao header 31 = DC, output.
- PB3 = Dabao header 32 = RESET, output.
- PB4 = Dabao header 34 = BUSY, input.

Sources:
- [Dabao pinout CSV](https://github.com/baochip/dabao/blob/main/docs/pinout/pins.csv)
- [Baochip electrical specifications / pin mux](https://github.com/baochip/baochip-1x/blob/main/docs/src/ch01-00-electrical-specifications.md)
- [Xous SPIM HAL](https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/udma/spim.rs)

## Waveshare V4 protocol

Upstream V4 source:
- [Python driver](https://github.com/waveshareteam/e-Paper/blob/master/RaspberryPi_JetsonNano/python/lib/waveshare_epd/epd2in13_V4.py)
- [Arduino driver](https://github.com/waveshareteam/e-Paper/blob/master/Arduino/epd2in13_V4/epd2in13_V4.cpp)
- [HAT+ schematic](https://files.waveshare.com/wiki/2.13inch-%20e-Paper-HAT%2B/2.13inch_e-Paper_HAT%2B-SCH.pdf)
- [Waveshare wiki](https://www.waveshare.com/wiki/2.13inch_e-Paper_HAT%2B)

Important protocol facts in the upstream driver:
- SPI mode 0 (CPOL=0, CPHA=0).
- MSB first.
- Raspberry Pi example selects 4 MHz; start conservatively at 1–2 MHz on the first Xous bring-up.
- DC low for command bytes, high for data bytes.
- BUSY high means the panel is busy; idle is low.
- Reset: HIGH 20 ms, LOW 2 ms, HIGH 20 ms.
- Resolution constants: 122 x 250 in the controller driver's orientation.
- Framebuffer stride: ceil(122/8) = 16 bytes.
- Full frame: 16 x 250 = 4,000 bytes.
- Full update: command 0x22, data 0xF7, command 0x20, then wait for BUSY to become low.

The Python driver's busy loop is unbounded. **Do not copy that into Xous.** Every BUSY wait needs a finite timeout and must return an error instead of freezing the console.

## Xous driver integration

The Xous codebase has an existing UDMA SPIM implementation at `libs/bao1x-hal/src/udma/spim.rs`. It provides:
- `SpimChannel::Channel2`.
- `SpimCs::Cs0`.
- `SpimClkPol::LeadingEdgeRise`.
- `SpimClkPha::CaptureOnLeading` for mode 0.
- MSB-first as the default.
- TX buffers backed by IFRAM and a DMA command queue.

Xous display code already demonstrates how to enable a UDMA peripheral clock and instantiate `Spim`. For a small e-Paper driver, command/data transactions can be sent as 8-bit SPI bytes. DC must be changed only after the previous DMA transfer has completed.

GPIO control needs to use the supported Xous IOX/HAL service/API rather than assuming direct register access is safe from a console process. The exact service wrapper and ownership pattern should be implemented before flashing.

Recommended initial SPI settings:
- channel: Channel2
- CS: CS0 (verify hardware route and transaction boundaries)
- mode: Standard, SPI mode 0
- bit order: MSB first
- word size: 8-bit
- requested speed: 1–2 MHz initial

Hardware CS may not naturally match every Waveshare command/data boundary. Verify the Xous SPIM CS behaviour and consider GPIO-managed CS if needed. Do not simultaneously configure PC3 as GPIO and SPIM CS.

## Test command design

A future `epaper` console command should have:
- `epaper info`: print selected pins/controller settings without touching the panel.
- `epaper reset`: hardware reset only.
- `epaper init`: full V4 init, bounded BUSY waits.
- `epaper white`: write 4,000-byte 0xFF buffer, full refresh.
- `epaper black`: write 4,000-byte 0x00 buffer, full refresh.
- `epaper sleep`: enter deep sleep.

The first test must not access the RTC, use partial refresh, or add fonts/graphics.

## Bench status (2026-11-26)

- User identified the board as Waveshare 2.13-inch e-Paper HAT+ with V4 marking.
- User reported all display and power wires connected and powered.
- HAT pin 1 measured 3.3 V.
- HAT pin 2 could not be reached at the HAT for a meter measurement.
- RTC wires disconnected after the I2C test froze the console.
- No SPI initialisation or display refresh result yet reported.

This folder documents the implementation plan; it is not a claim that the display driver is already implemented or working.


## Initial Xous console test source

The first command implementation is in `epaper.rs`. Install it into a Xous checkout with:

```bash
python3 /path/to/BaoBAB/component-tests/epaper/install.py "$PWD"
git diff --check
git status --short
git diff
cargo xtask dabao dabao-console
```

After reviewing the diff and a successful build, flash the image. Then use:

```text
epaper info
epaper reset
epaper init
epaper white
epaper black
```

The command uses Xous SPIM2, mode 0, MSB-first, initially 1 MHz, and a bounded BUSY wait. It drives DC/RST/BUSY through the Xous IOX HAL and hardware CS through SPIM2 CS0. It does not use the RTC.

**Not yet compiled or hardware-tested.** The configured SPIM source clock is currently set to 50 MHz based on the existing Xous Baochip display backend's configuration constant. Confirm against the active Dabao clock setup and check the compiler output before flashing. If SPI DMA or BUSY polling fails, stop there; do not repeatedly issue refreshes.


## Wiring correction — 2026-11-26

An earlier wiring table incorrectly identified the HAT physical header pins for MOSI, SCK, CS and BUSY. The corrected mapping above was derived from Waveshare's own `epdconfig.py` BCM pin definitions and standard Raspberry Pi physical-header numbering.

**Do not use the earlier assignments DIN17 / SCK19 / CS21 / BUSY16.** In particular, physical HAT pin 17 is a 3.3 V supply pin, not MOSI. Verify the corrected wiring with power disconnected before reconnecting USB.


## Required panel power-enable line — 2026-11-26

A review of Waveshare's upstream `epdconfig.py` found a separate power-enable GPIO:
- Raspberry Pi BCM GPIO18, physical HAT header pin 12.
- The official driver calls `GPIO_PWR_PIN.on()` in `module_init()` before opening SPI.
- Therefore this is not optional just because 3.3 V and 5 V rails are present.

Connect Dabao PB5 / physical header pin 35 to HAT physical pin 12. PB5 is exposed on the Dabao header and has no alternate peripheral function listed in the published pin CSV. The e-Paper console test configures PB5 as GPIO output and drives it HIGH before reset/initialisation.

Source: https://github.com/waveshareteam/e-Paper/blob/master/RaspberryPi_JetsonNano/python/lib/waveshare_epd/epdconfig.py

Do not flash the earlier command revision that omitted PWR. Update the local `epaper.rs` from the corrected repository version first.
