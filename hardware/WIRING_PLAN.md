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

## 5A. Optional PWM audio test route (in parallel with I2S)

The Dabao physical pinout maps **PB1 / physical pin 29 to PWM1.1**. Preserve that pin for a PWM test output. Route PB1 to an accessible, labelled perfboard test pad or small removable two-pin test header:

| Dabao physical pin | Dabao signal | Destination | Colour | Status |
|---:|---|---|---|---|
| 29 | PB1 / PWM1.1 | Labelled `PWM-AUDIO` test pad/header | Green | Pin mapping documented; firmware PWM setup and waveform still to be tested |

**Do not connect this PWM pin directly to the 8.7-ohm speaker or to MAX98357A DIN.** The MAX98357A is an I2S digital-input amplifier; it is not an analogue/PWM-input amplifier. A filtered PWM audio signal needs an appropriate low-pass filter and an analogue-input speaker amplifier. No separate analogue amplifier has yet been positively identified in the photographed salvaged green PCB. Do not assume that PCB's battery/speaker connections expose an analogue input.

### Mode selection

The two paths are electrically different:
- I2S mode: Dabao PC8/PC10/PC11 -> MAX98357A -> speaker.
- PWM mode: Dabao PB1/PWM1.1 -> low-pass filter -> analogue-input amplifier -> speaker.

A single signal jumper cannot safely switch the MAX98357A's digital input to PWM. Once a separate PWM amplifier is identified, switch **both speaker wires** between the two amplifier outputs using a two-pole selector. A practical header implementation is two independent 3-pin rows:

```text
Speaker terminal A ---- middle pin of row A
I2S OUT+ -------------- left pin of row A
PWM AMP OUT+ ---------- right pin of row A

Speaker terminal B ---- middle pin of row B
I2S OUT- -------------- left pin of row B
PWM AMP OUT- ---------- right pin of row B

For I2S: fit one removable jumper from middle-to-left on EACH row.
For PWM: fit one removable jumper from middle-to-right on EACH row.
Never fit both choices on either row. Power both amplifiers OFF before moving links.
```

This is a two-pole selector because MAX98357A OUT+ and OUT- are bridge-tied outputs; neither is ground. Never connect the amplifier outputs together. If no suitable shunts are already available, use two removable insulated jumper-wire links or leave the speaker connected to I2S and move its two leads manually only while power is off.

Until an analogue amplifier and its input/power requirements are identified, solder only the PB1 `PWM-AUDIO` test pad. Keep the speaker connected to the MAX98357A for I2S testing; leave the PWM-amplifier side of the speaker selector unconnected. This preserves the option without risking the speaker or amplifier.

The salvaged green PCB in the user's photo has an unidentified controller IC and battery/speaker connections. It is not yet confirmed to be a reusable analogue audio amplifier. Obtain a clear IC marking and trace/identify its signal inputs before connecting PB1 to it.

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

## 8. Joystick and four push buttons — six-GPIO two-phase matrix

This no-purchase arrangement supports five joystick actions and four push buttons using six GPIO signals. It preserves PB1 as the optional PWM output.

### Matrix GPIO allocation

| Role | Dabao pad | Dabao physical header pin | Function |
|---|---|---:|---|
| Matrix scan row | PB5 | 35 | Two-phase scan output/input; available only after removing the obsolete PB5-to-HAT-pin-12 connection and removing PB5 PWR setup from e-paper firmware |
| Joystick UP / column 0 | PC12 | 2 | GPIO input with pull-up; verify SD/QSPI not active |
| Joystick DOWN / column 1 | PC7 | 9 | GPIO input with pull-up; verify SD/QSPI not active |
| Joystick LEFT / column 2 | PC9 | 6 | GPIO input with pull-up; verify ADC0 not active |
| Joystick RIGHT / column 3 | PC2 | 11 | GPIO input with pull-up; e-paper write-only path does not use MISO |
| Joystick CENTER / column 4 | PC3 | 10 | Shared electrical net with e-paper CS0; must be time-multiplexed in firmware |

Avoid these known reservations:
- PB13 / physical 16 and PB14 / physical 15 are UART2 console RX/TX; do not use.
- PC13 / physical 1 is used by Dabao USB SE0 / boot-sense setup; do not use.
- PB11/PB12 are I2C0; PC0/PC1/PC3 and PB2/PB3/PB4 are e-paper signals; PC8/PC10/PC11 are the current MAX98357A BIO/I2S signals.
- PB1 / physical pin 29 is reserved for PWM1.1 audio output in this plan, not a matrix row.

PC7 and PC12 have SD/QSPI alternate functions, PC9 has ADC0, and PC2 has SPI2 MISO. These alternate functions do not by themselves prove the pin is unavailable as GPIO, but the active Xous image must explicitly mux them to GPIO and must not concurrently enable the corresponding peripheral. PC3 is more sensitive because it is the active e-paper SPI2 CS0 line; its use as a matrix column requires the software coordination described below.

### Joystick wiring

The user's switch has six terminals (three on each side). Treat it as a common plus five independent switch contacts only after verifying with continuity mode.

| Joystick contact (identify by continuity) | Connect to | Colour |
|---|---|---|
| COM | GND bus | Black |
| UP throw | PC12 / physical pin 2 | Green |
| DOWN throw | PC7 / physical pin 9 | Green |
| LEFT throw | PC9 / physical pin 6 | Green |
| RIGHT throw | PC2 / physical pin 11 | Green |
| CENTER throw | PC3 / physical pin 10 | Green |

Map the actual COM and throws with the switch unpowered. Do not infer the physical terminal order from package orientation.

### Four push buttons

Each push button bridges the PB5 scan row to one joystick column. They do **not** connect directly to GND in this matrix.

| Pushbutton | Terminal 1 | Terminal 2 | Event |
|---|---|---|---|
| Button 1 | PB5 / physical pin 35 (scan row) | PC12 / physical pin 2 | BACK |
| Button 2 | PB5 / physical pin 35 (scan row) | PC7 / physical pin 9 | CONFIRM |
| Button 3 | PB5 / physical pin 35 (scan row) | PC9 / physical pin 6 | MENU |
| Button 4 | PB5 / physical pin 35 (scan row) | PC2 / physical pin 11 | CANCEL |

Use green for all signal wires and label both ends. Joystick COM alone connects to black GND.

### Button physical placement and underside wiring sketch

The joystick and four tactile switches belong on the **outward/user-facing side** of the perfboard or on the enclosure's front control panel. They must not be trapped between the perfboard and the e-paper HAT. Use insulated short leads from SMD switch terminals to the perfboard's plated pads; strain-relieve the wires so the SMD terminals do not take mechanical force.

Top view, user-facing side (suggested arrangement only):

```text
          FRONT / USER SIDE
   +--------------------------------+
   |                                |
   |          [  JOYSTICK  ]        |
   |             UP                 |
   |          LEFT  o RIGHT         |
   |            DOWN                |
   |          (CENTER)              |
   |                                |
   |    [BACK] [CONFIRM]            |
   |    [MENU] [CANCEL]             |
   |                                |
   +--------------------------------+
           perfboard edge
```

Electrical view, underside of perfboard:

```text
                         FIVE JOYSTICK CONTACTS
PC12 / pin 2  <----------- UP       +----[joystick]---- GND (black)
PC7  / pin 9  <----------- DOWN     |
PC9  / pin 6  <----------- LEFT     |
PC2  / pin 11 <----------- RIGHT    |
PC3  / pin 10 <----------- CENTER   |

PB5 / pin 35  ----+----[BACK button]----- PC12 / pin 2
(scan row)        +----[CONFIRM button]-- PC7  / pin 9
                  +----[MENU button]----- PC9  / pin 6
                  +----[CANCEL button]--- PC2  / pin 11

PB1 / pin 29  -------------------------- PWM-AUDIO test pad only
```

Each two-terminal button is soldered between PB5 scan-row and its assigned column. The joystick's separate COM terminal connects to GND. Do not join the four pushbutton terminals to GND in this matrix arrangement. The exact six joystick terminals must be identified by continuity before soldering.

### Firmware scan behaviour and display-CS sharing

- Configure PC12, PC7, PC9, PC2 and (when not doing SPI) PC3 as GPIO inputs with pull-ups.
- Joystick phase: PB5 is an input with pull-up. A pressed joystick direction/centre grounds its column and reads LOW. A pushbutton by itself cannot pull a column LOW in this phase because PB5 is not driving LOW.
- Pushbutton phase: first ensure SPI2 is idle and its CS signal is deasserted. Set PB5's output latch LOW before changing PB5 to output-low. Read the five columns. A newly LOW column indicates its corresponding pushbutton.
- End the scan by returning PB5 to input/high-impedance with pull-up.
- PC3 is both HAT CS0 and the joystick CENTER column. During the matrix scan, configure PC3 as GPIO input with pull-up; its HAT CS net idles HIGH. A pressed CENTER or CANCEL switch may pull HAT CS LOW, but no SPI clocks are issued during scanning. Before any SPI transfer, stop scanning, put PB5 in high-impedance input mode, restore PC3 to SPIM2 CS0 alternate function, then perform the transfer. After the transfer, return PC3 to GPIO input mode before scanning resumes. Do not let the scan and SPI transfer run concurrently.
- Add an IOX/SPI mutex or equivalent single-owner arbitration. A held button must never connect a PB5 output-low to a PC3 CS output. PB5 must be high impedance before PC3 returns to SPI alternate function.
- Apply 15–20 ms software debounce initially. First acceptance target is one control pressed at a time; simultaneous joystick/button actions sharing a column are ambiguous.

### Physical placement

Mount the joystick and four tactile switches on the **user-facing/outward side** of the perfboard or on the enclosure's front control panel, not between the perfboard and e-paper HAT. Place the joystick where the user's thumb can move it; arrange BACK / CONFIRM / MENU / CANCEL around or below it. If the SMD footprint does not line up with 2.54 mm perfboard pads, use short insulated two-wire leads from each switch to the designated matrix nets—do not force the switch legs into misaligned holes.

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
5. Map the joystick COM/throws with continuity mode. The proposed six-GPIO two-phase matrix is documented in Section 8; confirm PC7/PC9/PC12 are not assigned to active SD/QSPI/ADC functions in the actual Xous image and implement the two-phase scanner before expecting button events.
6. Test display, haptic, motion and audio individually before combined-load testing.
