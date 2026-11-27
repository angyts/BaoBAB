# BaoBAB prototype — consolidated hardware wiring plan

**Updated:** 2026-11-27  
**Target:** Baochip-1x Dabao, Waveshare 2.13-inch e-Paper HAT V4, MAX98357A audio breakout, DRV2605L haptic breakout, GY-521 MPU6050, salvaged 8-ohm-class speaker, five-way joystick and four push buttons.

This is the current source of truth for prototype wiring. It deliberately separates manufacturer-/board-documented connections from items that still need a physical check. A documented pin mapping is not a claim that the completed assembly has passed electrical or functional testing.

## 1. Wire-colour convention

Only the five wire colours currently available are assumed:

| Colour | Use |
|---|---|
| Red | 3.3 V |
| Black | GND |
| Yellow | 5 V VBUS |
| Blue | Clock lines (SPI SCK, I2C SCL, I2S BCLK/LRC) |
| Green | Data and GPIO lines (SPI DIN/CS, I2C SDA, I2S DIN, control GPIO) |

Blue and green are reused, so label both ends of every signal wire (for example `EPD-DIN`, `EPD-CS`, `I2C-SDA`, `AUDIO-DIN`). Wire colours are a project convention, not a manufacturer standard.

## 2. Waveshare display — exact V4 manual supplied by the builder

Reference: [Waveshare 2.13-inch e-Paper HAT Manual](https://www.waveshare.com/wiki/2.13inch_e-Paper_HAT_Manual) and its [HAT schematic](https://files.waveshare.com/upload/4/44/2.13inch_e-Paper_HAT_Schematic_diagram.pdf).

The linked manual describes V4 as compatible with V3 and gives the 40-pin Raspberry Pi mapping for the display signals. It lists VCC, GND, DIN, CLK, CS, DC, RST and BUSY. The linked standard HAT schematic does **not** show an RTC chip or a separate PWR control input.

### Dabao to HAT 40-pin P1 header

| Dabao physical pin | Dabao signal | HAT physical pin (Pi numbering) | HAT signal | Colour | Status |
|---:|---|---:|---|---|---|
| 36 | 3V3 | 1 | 3V3 rail | Red | Documented power rail |
| 40 | VBUS (5 V) | 2 | RPI_5V rail | Yellow | Documented on the 40-pin HAT schematic |
| 3 | GND | 6 | GND | Black | Documented |
| 12 | PC1 / SPIM2 MOSI | 19 | DIN / MOSI | Green | Manual + Dabao pinout |
| 14 | PC0 / SPIM2 CLK | 23 | CLK / SCK | Blue | Manual + Dabao pinout |
| 10 | PC3 / SPIM2 CS0 | 24 | CS / CE0 | Green | Manual + Dabao pinout |
| 31 | PB2 | 22 | DC | Green | Manual + Dabao pinout |
| 32 | PB3 | 11 | RST | Green | Manual + Dabao pinout |
| 34 | PB4 | 18 | BUSY | Blue | Manual + Dabao pinout |

Notes:

- This build uses the HAT's **40-pin Raspberry Pi-compatible P1 header**. The HAT schematic shows 3V3 and RPI_5V rails on that header; pin 2 and pin 4 are the RPI_5V net on the HAT. The table uses Dabao VBUS on HAT pin 2. Do not connect 5 V to any of the 3.3 V-only sensor or audio breakouts.
- The manual's separate 8-pin panel cable/interface is a different connector: its VCC pin is the HAT's internal VCC rail. Do not confuse that connector with Raspberry Pi 40-pin P1 pin 2.
- **There is no separate HAT V4 PWR wire in the supplied manual's eight-signal interface. HAT physical pin 12 is a normal Raspberry Pi GPIO18 position, not a display PWR input. Do not connect Dabao PB5 to HAT pin 12.**
- HAT physical pin 18 is BUSY. Do not confuse it with the separate LSK4 HAT+ schematic/product.
- HAT physical pins 3 and 5 are the Raspberry Pi SDA/SCL positions on P1. The supplied standard V4 HAT schematic does not contain a DS3231 RTC. Do not describe pins 3/5 as a built-in RTC connection. If a separate RTC board is physically fitted, identify that board and its chip before wiring it.

## 3. Dabao signal reservations confirmed from the official pinout and Xous board source

Reference: [Dabao physical pinout CSV](https://github.com/baochip/dabao/blob/main/docs/pinout/pins.csv), [Xous Dabao board pin setup](https://github.com/betrusted-io/xous-core/blob/dev/libs/bao1x-hal/src/board/dabao.rs).

| Dabao physical pin | Pad | Function used in this plan |
|---:|---|---|
| 3 | GND | Common ground |
| 4 | PC11 / BIO27 | MAX98357A BCLK |
| 5 | PC10 / BIO26 | MAX98357A LRC |
| 7 | PC8 / BIO24 | MAX98357A DIN |
| 10 | PC3 | E-paper SPI2 CS0 |
| 11 | PC2 | SPI2D1/MISO pad; not used by this e-paper SPI transfer; only a *candidate* GPIO after pinmux audit |
| 12 | PC1 | E-paper SPI2 MOSI |
| 14 | PC0 | E-paper SPI2 CLK |
| 26 | PB12 | I2C0 SDA |
| 27 | PB11 | I2C0 SCL |
| 29 | PB1 | PWM1.1; keep reserved if retaining the proposed PWM-audio fallback |
| 31 | PB2 | E-paper DC |
| 32 | PB3 | E-paper RESET |
| 34 | PB4 | E-paper BUSY |
| 35 | PB5 | Available after removing the mistaken HAT-PWR setup from the e-paper command; candidate GPIO only after firmware/pin audit |
| 36 | 3V3 | 3.3 V distribution |
| 40 | VBUS | 5 V source for HAT P1 RPI_5V, subject to current-budget checks |

## 4. Shared I2C bus

Dabao I2C0 is PB12/SDA on physical pin 26 and PB11/SCL on physical pin 27. The same two signal wires are shared by I2C peripherals; each device is connected in parallel to SDA, SCL, 3.3 V and GND.

| Dabao connection | Device pin | Colour | Notes |
|---|---|---|---|
| Pin 26 / PB12 / SDA | DRV2605L SDA | Green | Shared bus |
| Pin 27 / PB11 / SCL | DRV2605L SCL | Blue | Shared bus |
| 3V3 bus | DRV2605L VIN | Red | 3.3 V module supply |
| GND bus | DRV2605L GND | Black | Common ground |
| Pin 26 / PB12 / SDA | GY-521 SDA | Green | Shared bus |
| Pin 27 / PB11 / SCL | GY-521 SCL | Blue | Shared bus |
| 3V3 bus | GY-521 VCC | Red | Use 3.3 V for this prototype |
| GND bus | GY-521 GND | Black | Common ground |
| 3V3 bus | GY-521 AD0 | Red | Tie HIGH to select 0x69; verify the module pin before soldering |
| Pin 26 / PB12 / SDA | External DS3231 SDA, **only if a separate RTC module is actually present** | Green | Not part of the standard HAT in the linked manual |
| Pin 27 / PB11 / SCL | External DS3231 SCL, **only if a separate RTC module is actually present** | Blue | Not part of the standard HAT in the linked manual |
| 3V3/GND bus | External DS3231 VCC/GND, if fitted | Red/Black | Confirm exact RTC breakout supply requirements first |

### I2C addresses — manufacturer-backed

| Device | 7-bit address | How selected | Evidence |
|---|---:|---|---|
| DRV2605L | **0x5A** | Fixed device address | TI DRV2605L datasheet |
| MPU-6050 / GY-521 | **0x68** when AD0 LOW; **0x69** when AD0 HIGH | AD0 is the address LSB | TDK/InvenSense register map |
| DS3231 RTC, if a separate DS3231 is present | **0x68** | Fixed address | Analog Devices DS3231 datasheet |

Therefore the proposed shared bus is address-distinct **only when MPU6050 AD0 is HIGH** if a DS3231 is also present: DRV2605L 0x5A, DS3231 0x68, MPU6050 0x69. The supplied standard e-Paper HAT manual/schematic does not show an onboard RTC, so an RTC must be treated as a separate, as-yet-unidentified module unless the physical board has a clearly identifiable RTC chip and backup battery.

I2C electrical checks before connecting all modules:

- Measure idle SDA and SCL relative to GND; the bus must idle HIGH at approximately 3.3 V, not 5 V.
- Check the pull-ups on the GY-521, DRV2605L and any separate RTC. Their parallel resistance is the combined bus pull-up; do not assume every clone board has the same resistor values.
- Do not join a 5 V pull-up to the Dabao 3.3 V I/O bus.
- Start at 100 kHz for first device detection.

Manufacturer references: [TI DRV2605L datasheet](https://www.ti.com/lit/ds/symlink/drv2605l.pdf), [TDK/InvenSense MPU-6000/6050 register map](https://invensense.tdk.com/wp-content/uploads/2015/02/MPU-6000-Register-Map.pdf), [Analog Devices DS3231 datasheet](https://www.analog.com/media/en/technical-documentation/data-sheets/DS3231.pdf).

## 5. MAX98357A audio breakout and salvaged speaker

| Dabao connection | MAX98357A pin | Colour |
|---|---|---|
| Physical pin 36 / 3V3 | VIN | Red |
| Physical pin 3 / GND | GND | Black |
| Physical pin 7 / PC8 / BIO24 | DIN | Green |
| Physical pin 5 / PC10 / BIO26 | LRC | Blue |
| Physical pin 4 / PC11 / BIO27 | BCLK | Blue |
| MAX98357A OUT+ | Speaker terminal A | Separate speaker lead |
| MAX98357A OUT- | Speaker terminal B | Separate speaker lead |

- Keep GAIN open initially and preserve the breakout's existing SD_MODE resistor network.
- Connect the speaker only between OUT+ and OUT-. Neither output goes to GND.
- The salvaged speaker measured **8.7 ohms DC resistance** with its leads disconnected. That is consistent with an 8-ohm-class speaker, but it does not establish its power rating. Start with the existing low-amplitude, short tone test only.
- The BIO FIFO completion reported by the firmware is not proof that valid I2S waveforms reach the amplifier. Dynamic BCLK/LRC/DIN timing still needs measurement if output remains silent.

## 6. DRV2605L haptic breakout and motor

| Dabao connection | DRV2605L breakout pin | Colour |
|---|---|---|
| Physical pin 36 / 3V3 | VIN | Red |
| Physical pin 3 / GND | GND | Black |
| Physical pin 26 / PB12 / SDA | SDA | Green |
| Physical pin 27 / PB11 / SCL | SCL | Blue |
| DRV2605L OUT+ | Motor terminal A | Separate motor lead |
| DRV2605L OUT- | Motor terminal B | Separate motor lead |

Leave the breakout's IN pin unconnected for the initial I2C-controlled use. Do not connect either motor output directly to a Dabao GPIO. Motor voltage/current rating and 3.3 V regulator headroom remain unverified; test short pulses and watch for supply droop or heating.

## 7. GY-521 MPU6050 motion module

| Dabao connection | GY-521 pin | Colour |
|---|---|---|
| Physical pin 36 / 3V3 | VCC | Red |
| Physical pin 3 / GND | GND | Black |
| Physical pin 26 / PB12 / SDA | SDA | Green |
| Physical pin 27 / PB11 / SCL | SCL | Blue |
| 3V3 bus | AD0 | Red |

Leave XDA, XCL and INT disconnected for the first I2C identity test. The module is marked GY-521 and MPU6050 in the supplied photos. Its exact clone circuitry and onboard pull-up arrangement still need inspection; use 3.3 V and verify the I2C idle level before connecting the shared bus.

## 8. Joystick and four push buttons — GPIO mapping is NOT approved yet

There are nine physical switch actions: five-way joystick (up/down/left/right/centre) plus four independent two-terminal push buttons. The previous proposed direct map must **not** be soldered:

- Dabao PB13 (physical pin 16) and PB14 (physical pin 15) are used by Xous `setup_console_pins()` for UART2 RX/TX. Taking them for buttons would break the console path.
- Dabao PC13 (physical pin 1) is used by Dabao USB SE0 / boot-pin setup. Do not allocate it to a button.
- Dabao PB1 (physical pin 29) is PWM1.1 and was being reserved for the optional PWM-audio fallback.
- PC7, PC9 and PC12 (physical pins 9, 6 and 2) have SD/QSPI alternate functions in the Dabao pinout; their use as GPIO depends on whether the board's SD/QSPI path is enabled.
- PC2 (physical pin 11) is SPI2D1/MISO; the e-paper transaction does not use MISO, but GPIO mode must be verified.
- PB5 (physical pin 35) is no longer needed by the standard HAT V4 e-paper interface after the PWR correction; it is only a candidate after the updated firmware is built and its pinmux is audited.

Consequently, there is **not yet a verified nine-input direct-GPIO allocation**. The joystick's six physical terminals must first be mapped with a multimeter (COM plus five contacts cannot be inferred from package orientation). We will evaluate a matrix/other no-purchase scan arrangement only after measuring the switch topology and checking which SD/QSPI pins are unused. Do not solder the button signal wires based on the earlier PC13/PB13/PB14 list.

## 9. Power and assembly safety

- Dabao spare 3.3 V regulator output capacity is not established. Keep each module's power branch removable and bring devices up individually.
- The HAT uses the Pi-compatible P1 rails in the manner shown by its schematic; the 5 V rail is **not** for the MAX98357A, DRV2605L or GY-521 in this plan.
- Power off and disconnect USB before continuity tests, soldering or moving wires.
- Before power-on check continuity of every connection and check for shorts between 3V3/GND, 5V/GND, and 5V/3V3.
- Do not connect either MAX98357A speaker output to GND.
- Do not power all modules simultaneously until current draw and rail stability have been checked.

## 10. Bench status and what has actually been verified

- The e-paper HAL/UDMA diagnostic ran successfully: HAL lookup, GPIO-bank reads, SPIM2 clock/reset IPC, IFRAM-backed SPIM allocation/drop and clock-disable acknowledgement all passed.
- A physical jumper on the display RESET line was found open and continuity was restored.
- The display still timed out waiting for BUSY after reset in the latest supplied log. The wiring plan is not a claim that the panel is now working.
- The HAT's physical BUSY pin is Raspberry Pi pin 18 according to the supplied V4 manual. Earlier measurement of HAT pin 18 as GND was based on a different HAT+ LSK4 schematic and was incorrect for the V4 manual the builder identified.
- The PB5-to-HAT-pin-12 wire/configuration is obsolete for this HAT V4 plan and must be removed.
- Audio data transfer completed at the BIO/FIFO layer, but the user reported no audible sound; dynamic I2S output has not yet been confirmed with an oscilloscope/logic analyser.
- DRV2605L tests previously returned the expected device identity and the user reported vibration, but combined power-budget testing remains outstanding.
- The MPU6050 board has been visually identified as GY-521; it has not yet been tested on the shared I2C bus.

## 11. No-flash next actions

1. Remove the obsolete PB5-to-HAT-pin-12 connection.
2. Update the e-paper source so PB5 is not configured/driven as a display PWR output (this document's companion commit makes that source correction).
3. Verify the HAT's 3V3, 5V and GND rails at the perfboard.
4. Check I2C pull-ups and idle levels before joining all I2C modules; then perform bounded identity reads for 0x5A and 0x69 (and 0x68 only if a separate DS3231 is physically present).
5. Finish button pin allocation only after the Xous/SD/QSPI pin audit and joystick continuity map. Do not solder button wiring from an unverified pin table.
6. Test display, haptic, motion and audio individually before combined-load testing.
