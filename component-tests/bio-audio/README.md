# BIO-powered I2S audio experiment (Xous / Dabao)

**Status:** first clock-only test source committed; not yet installed into Xous, compiled, or tested on hardware. The 440 Hz tone and PCM streaming are not implemented. The MAX98357A may be connected only after the clock test is reviewed and the wiring below is corrected.

## 1. Existing Xous facilities

The current Xous tree includes the Baochip BIO service and Rust interface:
- `libs/bao1x-api/src/bio.rs`: `BioApi`, `BioCore`, `BioPin`, `ClockMode`, `IoConfig`, and `bio_code!`.
- `libs/bao1x-api/src/bio_resources.rs`: runtime BIO core/FIFO/pin reservation types.
- `libs/bao1x-hal/src/bio.rs`: Xous client for BIO and resource management.
- `libs/bao1x-hal/src/bio_hw.rs`: hardware implementation for loading cores, clock dividers and BIO I/O mapping.
- `services/bao1x-hal-service/src/servers/bio.rs`: BIO server/resource tracker.
- `libs/bio-lib/src/ws2812.rs`: working example of BIO program setup, resource claims, pin mapping, core start/stop and CSR handles.

BaoBAB should reuse Xous's BIO service and resource manager. Do not write directly to BIO registers or bypass its ownership tracker.

Source references:
- https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-api/src/bio.rs
- https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-api/src/bio_resources.rs
- https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/bio.rs
- https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/bio_hw.rs
- https://github.com/betrusted-io/xous-core/blob/main/libs/bio-lib/src/ws2812.rs

## 2. Corrected Dabao BIO pin mapping

Xous maps PB pins to BIO bits 0-15 and PC pins to BIO bits 16-31:
- PC8 = BIO24
- PC9 = BIO25
- PC10 = BIO26
- PC11 = BIO27

The Dabao console's existing BIO loader allow-list is:
`[1, 2, 3, 4, 5, 11, 12, 16, 17, 18, 19, 23, 24, 26, 27, 28, 29]`.

Therefore PC9 / BIO25 is excluded by that allow-list. It was incorrectly proposed earlier as the audio data pin. **Do not use PC9 for this experiment.** The clock test uses PC8 / BIO24 for data instead, because BIO24 is included in the allow-list. PC10 / BIO26 and PC11 / BIO27 are also in that list.

| Signal | Dabao pad | BIO bit | MAX98357A |
|---|---|---:|---|
| Serial data (DIN) | PC8 | 24 | DIN |
| Word/frame select | PC10 | 26 | LRC / LRCLK |
| Bit clock | PC11 | 27 | BCLK |
| Power | 3V3 | — | VIN |
| Ground | GND | — | GND |

The standard Dabao pinout identifies PC8 as physical header pin 7, PC10 as physical pin 5, and PC11 as physical pin 4. Confirm the header orientation against the official pinout before moving jumpers: https://github.com/baochip/dabao/blob/main/docs/pinout/index.html

BIO bit numbers are positions in the BIO bitmask, not GPIO port pin numbers.

## 3. MAX98357A facts

Manufacturer datasheet: https://www.analog.com/media/en/technical-documentation/data-sheets/MAX98357A-MAX98357B.pdf

- MAX98357A accepts standard I2S (DIN, BCLK, LRCLK) and does not require MCLK.
- Supports 8 kHz–96 kHz sample rates and 16/24/32-bit I2S resolution.
- OUT+ and OUT- are bridge-tied speaker outputs. Neither speaker lead may be grounded.
- Verify the exact breakout board's GAIN and SD_MODE straps; breakout clones can differ.

Adafruit breakout notes: https://learn.adafruit.com/adafruit-max98357-i2s-class-d-mono-amp/pinouts

## 4. Wiring reference (corrected)

| MAX98357A | Dabao |
|---|---|
| VIN | 3V3 |
| GND | GND |
| DIN | **PC8 / BIO24** |
| LRC | PC10 / BIO26 |
| BCLK | PC11 / BIO27 |
| GAIN | Leave open for initial bring-up (subject to exact breakout variant) |
| SD | Keep the breakout's default resistor network intact; don't wire to GPIO |
| OUT+ / OUT- | Speaker across both outputs; neither side to GND |

**Move the existing DIN jumper from PC9 to PC8 before installing/running this test.** Leave BCLK and LRC on PC11 and PC10 respectively. Do not solder anything.

## 5. Current implementation: clock-only test

Files:
- `i2s_clock_kernel.rs`: BIO assembly kernel. DIN remains low. It toggles BCLK and LRCLK only.
- `i2s_clock.rs`: Xous BIO resource-managed wrapper with explicit start/stop and pin cleanup.
- `i2s_clock_cmd.rs`: Dabao console command `i2sclock start|status|stop`.
- `install.py`: conservative installer for a local Xous checkout. It does not build or flash.

At a nominal BIO quantum of 3.072 MHz, the kernel performs two quantum waits per BCLK period:
- BCLK nominal = 1.536 MHz.
- LRCLK toggles after 16 BCLK periods, giving 48 kHz full-period LRCLK.
- DIN is held low (digital zero); this is a clock smoke test, not a tone.

The frequency printed by the command comes from Xous's actual BIO clock-divider calculation. Hardware timing must still be checked; the printed number is the configured quantum rate, not an oscilloscope measurement.

## 6. Safe test sequence

1. Power down the board before moving the DIN jumper from PC9 to PC8.
2. Confirm VIN and GND connections, speaker connected only across OUT+ and OUT-, and no unintended shorts.
3. Review the installer diff before building.
4. Build and flash using the user's established Xous procedure.
5. Start the BIO clock-only test with the MAX98357A powered from 3V3 and DIN held low. This is digital silence; use low volume conditions and stop immediately for unexpected noise, heat, smell, or excessive current.
6. Observe BCLK/LRCLK with a logic analyser or oscilloscope if available. Verify approximately 1.536 MHz BCLK and 48 kHz LRCLK.
7. Stop the clock test and verify it reports pins released.
8. Only after clocks are verified should we implement the 440 Hz PCM tone, followed by bounded PCM playback.

Command sequence:
```text
i2sclock start
i2sclock status
i2sclock stop
```

## 7. Remaining work

- No BIO I2S 440 Hz tone or PCM streaming has been implemented.
- No hardware clock measurement has been made.
- No audio output has yet been validated.
- Need to review BCLK/LRCLK waveform against the MAX98357A timing diagrams and verify that the chosen 16-bit slot format is accepted by the actual module.
- Need add low-amplitude sample generation, a bounded playback duration, explicit stop-on-error, and audio buffer management before sound tests.

**Do not treat this clock-only implementation as a working audio player.**
