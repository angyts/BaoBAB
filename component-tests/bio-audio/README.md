# BIO-powered I2S audio experiment (Xous / Dabao)

**Status:** design verified against source; transmitter not yet implemented or flashed. Keep the MAX98357A unpowered until test firmware is ready.

## 1. Existing Xous facilities

The current Xous tree already includes the Baochip BIO service and Rust interface:
- libs/bao1x-api/src/bio.rs: BioApi, BioCore, BioPin, ClockMode, IoConfig and the bio_code! macro.
- libs/bao1x-api/src/bio_resources.rs: runtime BIO core/FIFO/pin reservation types.
- libs/bao1x-hal/src/bio.rs: Xous client for BIO and resource management.
- libs/bao1x-hal/src/bio_hw.rs: hardware implementation for loading cores, clock dividers and BIO I/O mapping.
- services/bao1x-hal-service/src/servers/bio.rs: BIO server/resource tracker.
- libs/bio-lib/src/ws2812.rs: working example of a BIO program, runtime resource claim, pin mapping, core start/stop and Rust-side FIFO CSR handles.

BaoBAB should reuse Xous's BIO service and resource manager. Do not write directly to BIO registers or bypass its ownership tracker.

Source references:
- https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-api/src/bio.rs
- https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-api/src/bio_resources.rs
- https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/bio.rs
- https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/bio_hw.rs
- https://github.com/betrusted-io/xous-core/blob/main/libs/bio-lib/src/ws2812.rs
- https://github.com/betrusted-io/xous-core/blob/main/libs/bio-lib/README.md

The BIO library README says applications should declare resources, claim the required resources, and release them on Drop. Concurrent BIO applications may conflict because cores, FIFOs and pins are limited.

## 2. Dabao BIO pin mapping

Xous's port_and_pin_to_bio_bit mapping maps:
- PC9 to BIO bit 25
- PC10 to BIO bit 26
- PC11 to BIO bit 27

The reverse mapping is also explicit. Therefore the proposed Dabao signal assignment corresponds to these BIO bits, provided the selected board header pads are indeed PC9, PC10 and PC11.

| Signal | Dabao pad | BIO bit | MAX98357A |
|---|---:|---:|---|
| Serial data | PC9 | 25 | DIN |
| Word/frame select | PC10 | 26 | LRC / LRCLK |
| Bit clock | PC11 | 27 | BCLK |
| Power | 3V3 | — | VIN |
| Ground | GND | — | GND |

BIO bits 25–27 are positions in the BIO bitmask, not GPIO port numbers. The output mapping mask is bits 25, 26 and 27 set.

## 3. MAX98357A facts and breakout defaults

Manufacturer datasheet: https://www.analog.com/media/en/technical-documentation/data-sheets/MAX98357A-MAX98357B.pdf

- MAX98357A accepts standard I2S (DIN, BCLK, LRCLK) and does not require MCLK.
- Supports 8 kHz–96 kHz sample rates and 16/24/32-bit I2S resolution.
- MAX98357A samples data on BCLK rising edge.
- Standard I2S timing includes a one-BCLK delay from LRCLK transition to the next word's MSB.
- Do not stop LRCLK while BCLK remains active; the datasheet warns this can cause unexpected output including a large DC speaker voltage.
- OUT+ and OUT- are bridge-tied outputs. Neither speaker lead may be grounded.

Adafruit breakout documentation: https://learn.adafruit.com/adafruit-max98357-i2s-class-d-mono-amp/pinouts

For an Adafruit-style breakout:
- GAIN left open gives default 9 dB gain.
- SD left open uses the board resistor network for the mono L+R mix. Its exact voltage depends on VIN.
- GAIN is not a GPIO signal. Leave it open for the first test.
- SD is not a GPIO signal. Leave the breakout's default resistor network intact; do not connect SD to a Baochip GPIO or hard-tie it to ground for normal playback.
- Confirm the actual breakout uses the Adafruit-style resistor network before relying on these defaults; clones can differ.

## 4. Wiring reference

The user's breadboard wiring is in place; keep power disconnected until the BIO transmitter is ready and reviewed.

| MAX98357A | Dabao |
|---|---|
| VIN | 3V3 |
| GND | GND |
| DIN | PC9 / BIO25 |
| LRC | PC10 / BIO26 |
| BCLK | PC11 / BIO27 |
| GAIN | Open |
| SD | Open only if breakout default pull-up network is present |
| OUT+ / OUT- | 8 ohm speaker across the two outputs; neither side to ground |

The speaker is believed to be 8 ohm, 0.5 W. Start at very low digital amplitude (target no more than 5% full scale for the first tone) and a short bounded duration.

## 5. BIO transmitter timing proposal

A sensible first target is 48 kHz stereo, 16-bit PCM. Nominal BCLK for 16-bit stereo slots is sample-rate × 2 channels × 16 bits = 1.536 MHz. A 32-bit slot format instead uses 3.072 MHz BCLK. Choose one framing explicitly and validate the exact standard I2S one-bit delay against the MAX98357A timing diagram before connecting power.

The Xous BIO core runs its program with a host-configured quantum divider. BIO ClockMode::TargetFreqInt can request an integer-divider quantum rate. BIO assembly can use x20 to wait on the next quantum and x21–x26 for GPIO access. The Xous ws2812 BIO example demonstrates these patterns, but is not an I2S driver.

Safe development sequence:
1. Create a Xous BIO library component using the existing Bio HAL and BioResources resource manager.
2. Claim one available BIO core and BIO pins 25, 26, 27. Avoid hardcoding Core0; ask the allocator for an available core.
3. Configure BIO I/O mapping only for the three pins, and set them as outputs.
4. First run a clock-only test with MAX98357A unpowered. Give BIO code an explicit bounded stop path.
5. Implement standard I2S framing with explicit bit order, BCLK rising-edge data validity, LRCLK polarity, slot width and one-BCLK MSB delay.
6. Generate a short, low-amplitude 440 Hz tone; verify LRCLK and BCLK rates and speaker waveform. Do not allow BCLK to continue if LRCLK stops.
7. Only after that passes, add PCM buffer playback and a producer/consumer strategy. Do not assume Xous task scheduling alone can refill a tiny FIFO without underruns.
8. Implement Drop/stop so it stops the BIO core, returns pins from BIO mux ownership, and releases all resources.

Suggested acceptance criteria for 48 kHz / 16-bit stereo:
- LRCLK = 48 kHz.
- BCLK = 1.536 MHz.
- BCLK has stable duty cycle.
- DIN follows MAX98357A I2S timing, including one-bit delay after LRCLK transitions.
- Both channels are populated consistently for mono output.
- No clock continues after stop.
- Tone duration is bounded (initially 250–500 ms).
- First test amplitude <= 5% full scale.

## 6. Important caveat

No custom BIO I2S transmitter has been committed or tested yet. Existing Xous BIO service, resource APIs, BIO pin mapping and WS2812 reference implementation are confirmed in source. I2S bit timing, PCM buffering and actual sound output remain to be implemented and hardware-verified. Do not interpret the proposed timings as proof that the transmitter is ready.

## 7. References

1. MAX98357A/MAX98357B datasheet: https://www.analog.com/media/en/technical-documentation/data-sheets/MAX98357A-MAX98357B.pdf
2. Adafruit MAX98357A breakout pinouts/default GAIN and SD behaviour: https://learn.adafruit.com/adafruit-max98357-i2s-class-d-mono-amp/pinouts
3. Xous BIO API: https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-api/src/bio.rs
4. Xous BIO resource manager API: https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-api/src/bio_resources.rs
5. Xous BIO Rust HAL: https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/bio.rs
6. Xous BIO hardware implementation: https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/bio_hw.rs
7. Xous BIO WS2812 example: https://github.com/betrusted-io/xous-core/blob/main/libs/bio-lib/src/ws2812.rs
8. Xous BIO library conventions: https://github.com/betrusted-io/xous-core/blob/main/libs/bio-lib/README.md
