# Baochip audio architecture research: UDMA I2S vs PWM + low-pass filter

**Research date:** 2026-11-26  
**Repositories reviewed:** `baochip/baochip-1x`, `betrusted-io/xous-core`, `ArmstrongSubero/dabao-sdk`  
**Target hardware:** Dabao board, Baochip-1x, MAX98357A breakout and miniature speaker  
**Status:** source-level investigation; no firmware or wiring changed.

## Executive summary

There are two real audio-output approaches supported by the Baochip silicon:

1. **Dedicated UDMA I2S TX:** a hardware serial-audio engine with clock/word-select generation, TX/RX datapaths and DMA channels. The RTL is implemented. The current public Xous tree has the peripheral ID and event definitions but does not expose an I2S driver in `libs/bao1x-hal/src/udma`; the directory exports ADC, I2C, SPI master and UART only. A Xous driver would need to be written and its physical pin routing on the Dabao board verified.
2. **PWM + analogue low-pass + analogue-input amplifier:** Armstrong Subero directly told the user that his Dabao audio output is PWM through a low-pass filter (suggested starting point: 1 kΩ and 100 nF), then an amplifier. The public SDK has a working PWM driver and PB1 LED example, but no public PCM audio player/sample-streaming loop was found. This is a likely simpler first sound-output experiment if an analogue-input amplifier is available.

**Crucial compatibility distinction:** MAX98357A is a digital I2S-input Class-D amplifier. It cannot accept the filtered analogue voltage from PWM. For PWM audio, use a separate amplifier module with an analogue input. To keep the MAX98357A, use true I2S (either the custom BIO transmitter or the dedicated UDMA I2S TX).

Do not interpret the PWM LED example as proof that the SDK has a music player. It proves PWM pin setup, carrier generation and duty-cycle changes.

---

## 1. Armstrong Subero's direct clarification

The user reports receiving this direct response from Armstrong Subero on 2026-11-26:

> On the Dabao it’s just PWM through a low pass filter, when in doubt 1k and 100nF… then feed it into an amp and that’s it…audio…well audio out at least..audio in you’d need a mic and op amp to keep the sound within the Dabao’s bang gap range realistically under a volt and sample that…

This is a direct statement relayed by the user, not a public source-code excerpt. It clarifies the architecture of his Dabao audio output:

```text
PCM/sample values in software
          |
          v
Change PWM duty cycle over time
          |
          v
PWM pin: fast digital pulse train
          |
          v
RC low-pass filter (e.g. 1 kOhm + 100 nF)
          |
          v
Analogue voltage waveform
          |
          v
Analogue-input audio amplifier
          |
          v
Speaker
```

The public Dabao SDK contains a PWM driver and LED-breathing example. It does not contain a public audio PCM buffer/player implementation in the examined main branch. Therefore, Armstrong's direct description establishes his hardware topology, but the complete sample timing, carrier frequency, sample rate, buffering and anti-alias/filter design are not established by that public example.

For audio input, his point is that a microphone signal needs analogue gain/conditioning before reaching the chip ADC. Baochip ADC's documented input range is around 0.3–1.0 V, so a microphone cannot simply be connected directly. The microphone preamp must keep the waveform within the ADC's permitted input range and bias requirements; check the ADC and board analogue schematic before wiring.

---

## 2. Armstrong's public PWM driver and example

Repository:
- https://github.com/ArmstrongSubero/dabao-sdk
- PWM implementation: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/src/bao1x/hardware_pwm/pwm.c
- PWM API: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/src/bao1x/hardware_pwm/include/hardware/pwm.h
- PWM example: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/pwm/pwm.c

### 2.1 What the example does

The example is explicitly an LED breathing demonstration:
- Configures PB1 as PWM alternate function AF3.
- Uses PWM slice 1, channel 1.
- Starts PWM at 1 kHz.
- Ramps duty cycle from 0% to 100% and back in 20 ms steps.
- The LED is connected between PB1 and ground through a 330-ohm resistor.

This is not an audio sample player, but the exact mechanism needed for rudimentary PWM audio is present: a fixed PWM carrier with a duty value that software can update.

### 2.2 PWM driver capabilities

The public driver exposes:
- `pwm_init_pin(port, pin)`: selects AF3 and enables output.
- `pwm_init(slice, freq_hz)`: starts a PWM slice at a requested frequency, chooses prescaler and period.
- `pwm_set_duty(slice, channel, duty)`: changes raw duty count.
- `pwm_set_percent(slice, channel, percent)`: percentage wrapper.
- `pwm_stop(slice)`: stops the slice.
- `pwm_get_period(slice)`: returns the cached period.

Header documents 4 slices, each with 4 channels. On Dabao, the SDK maps:
- PWM1: PB0–PB3, channels 0–3, AF3.
- PWM2: PC0–PC3, channels 0–3, AF3.

The example uses PB1 (header pin 29 according to its own wiring comment), PWM slice 1 channel 1.

### 2.3 PWM clock arithmetic in the driver

The driver sets `pwm_clk = ACLK_HZ / 8`, corresponding to PCLK (43.75 MHz under the documented Baochip clock plan). It selects an 8-bit prescaler and 16-bit counter period:

```text
timer_clock = PCLK / (prescaler + 1)
PWM_carrier = timer_clock / period
```

The counter counts from zero through `period - 1`. The driver uses a compare/update register write to change the duty, and explicitly handles constant 0% and 100% levels.

**Audio implication:** choose a sufficiently high PWM carrier to make the output filter remove the switching component, while preserving enough period counts to provide useful amplitude resolution. The SDK's 1 kHz LED carrier is not an appropriate default audio carrier.

### 2.4 What is missing for audio

The public PWM API does not itself provide:
- PCM/WAV file decoding.
- A sample rate timer callback.
- A sample ring buffer.
- DMA-fed duty updates.
- Underrun/overrun management.
- Stereo output or channel mixing.
- A verified audio reconstruction filter design.

A first software experiment could generate a short sine table and update one PWM channel's duty at a fixed sample cadence. Before calling it a player, measure sample timing jitter and ensure that the carrier PWM remains stable while duty updates occur.

---

## 3. The proposed 1 kΩ / 100 nF RC filter

Armstrong's suggested starting values are:
- Series resistor: 1 kΩ.
- Capacitor from filtered node to ground: 100 nF.

For a simple first-order RC low-pass:

```text
fc = 1 / (2 * pi * R * C)
```

With 1 kΩ and 100 nF:

```text
fc = 1 / (2 * pi * 1000 * 100e-9)
   ≈ 1,592 Hz
```

This is the approximate -3 dB corner frequency.

A first-order filter's amplitude response is:

```text
|H(f)| = 1 / sqrt(1 + (f/fc)^2)
```

Approximate implications:
- At 440 Hz: about -0.3 dB.
- At 1 kHz: about -1.4 dB.
- At 2 kHz: about -4.1 dB.
- At 5 kHz: about -10.4 dB.
- At a 31.25 kHz PWM carrier: about -25.9 dB.
- At a 62.5 kHz PWM carrier: about -31.9 dB.

**Interpretation:** 1 kΩ + 100 nF is a convenient low-cost starting filter for simple, low-bandwidth audio. It rolls off higher audio frequencies considerably, so it should not be assumed to provide hi-fi/full-bandwidth playback. Its effectiveness also depends on PWM carrier frequency, GPIO output impedance, amplifier input impedance, wiring and the actual load. A single RC stage has only a 20 dB/decade attenuation slope.

Do not connect a speaker directly to the filtered PWM node. The GPIO and RC network are signal-level circuitry, not a speaker power stage.

---

## 4. Which amplifier can be used?

The existing MAX98357A breakout is designed for **digital I2S input**, not analogue line input. The filtered PWM signal is analogue-like; connecting it to MAX98357A DIN would not turn it into an audio signal because DIN expects serial digital data synchronized to BCLK/LRCLK.

Therefore:

- **PWM route:** select an amplifier board with an analogue input (line-level or suitable single-ended input) and adequate speaker drive. Its supply, gain, input bias and output topology must be checked. The speaker connects to that amplifier's outputs, not to the Baochip pin.
- **I2S route:** keep the MAX98357A and provide actual I2S BCLK, LRCLK and DIN.
- Do not connect MAX98357A OUT+/OUT- to ground; these are bridge-tied speaker outputs.

This means adopting Armstrong's PWM architecture would require a different amplifier module from the one currently on the breadboard.

---

## 5. Dedicated UDMA I2S: RTL audit

The silicon RTL contains a real I2S peripheral:
- Top: https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/udma_i2s_top.sv
- Register interface: https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/udma_i2s_reg_if.sv
- TX channel: https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/i2s_tx_channel.sv
- RX channel: https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/i2s_rx_channel.sv
- Clock/word-select generator: https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/i2s_clkws_gen.sv
- Register reference: https://ci.betrusted.io/bao1x/udma_i2s.html

### 5.1 It is more than a register stub

The RTL implements:
- One I2S peripheral instance in the uDMA subsystem.
- A master TX path (serial data out).
- A slave RX path (serial data in).
- Two serial-data outputs on the master side (SD0 and SD1).
- Two serial-data inputs on the slave side (SD0 and SD1).
- Internal or externally supplied clock/word-select selection.
- Programmable master and slave word sizes, number of words, 2-channel mode and bit order.
- TX/RX uDMA channels and clock-domain-crossing FIFOs.
- PDM capture processing in the slave receive path, including CIC decimation.

The PDM block is a receive/input feature. It should not be mistaken for a PWM or I2S audio-output modulator.

### 5.2 How playback is intended to work

Conceptually:

```text
PCM samples in system memory
          |
          v
uDMA TX channel for I2S
          |
          v
I2S TX FIFO / clock-domain crossing
          |
          v
I2S master TX shift registers
          |
          +----> SD0 (serial data)
          |
          +----> SD1 (second serial data)
          |
          v
BCLK + WS/LRCLK generated by I2S clock block
          |
          v
I2S receiver / DAC / MAX98357A
```

The TX engine loads channel data into shift registers, waits for word-select boundary, shifts each configured word, and requests additional FIFO data at defined word boundaries. The clock generator produces the selected master clock and WS timing. This is the hardware work our BIO implementation currently tries to reproduce in software.

The I2S master clock generator is driven by the I2S peripheral clock and uses a programmable divider. The divider is 8 bits. The generated clock is a toggle divider: a counter runs through the programmed value, then toggles the clock. For an integer divider D, the ideal output is approximately:

```text
BCLK = peripheral_clock / (2 * (D + 1))
```

The word-select generator divides the bit clock according to configured word size and word count. Calculate achievable sample rates from the actual peripheral clock and verify against the amplifier's supported rate range. Do not assume that an arbitrary exact 44.1 or 48 kHz clock can be generated from every source/divider combination.

### 5.3 I2S register map

The generated register documentation lists the base at `0x5010e000`:

| Offset | Register | Meaning |
|---:|---|---|
| 0x00 | RX_SADDR | RX DMA start address |
| 0x04 | RX_SIZE | RX transfer size |
| 0x08 | RX_CFG | RX enable, continuous mode, data size, clear |
| 0x10 | TX_SADDR | TX DMA start address |
| 0x14 | TX_SIZE | TX transfer size |
| 0x18 | TX_CFG | TX enable, continuous mode, data size, clear |
| 0x20 | I2S_CLKCFG_SETUP | master/slave divider, clock enables and clock source selection |
| 0x24 | I2S_SLV_SETUP | receive word size/count, bit order, 2-channel mode, enable |
| 0x28 | I2S_MST_SETUP | transmit word size/count, bit order, 2-channel mode, enable |
| 0x2C | I2S_PDM_SETUP | PDM receive mode, decimation and shift |

TX DMA and the I2S serializer are separate parts of the block. A usable driver must configure both correctly, including the uDMA global clock gate, reset, TX DMA channel routing, buffer address/size, transfer size and the I2S master clock/word format.

### 5.4 Current Xous software support

In the public `betrusted-io/xous-core` tree examined on 2026-11-26:
- `libs/bao1x-api/src/udma.rs` defines `PeriphId::I2s` and I2S TX/RX event offsets.
- `libs/bao1x-hal/src/udma/mod.rs` exports `adc`, `i2c`, `spim` and `uart`.
- There is no `i2s.rs` driver in that HAL UDMA directory and no implementation is exported from it.
- `services/bao1x-emu/src/udma.rs` has stubbed global uDMA clock/event/reset operations.
- The old Renode emulator model `BetrustedI2S.cs` provides a reference behavioural model, but it does not constitute a native Baochip Xous driver.
- Existing `services/codec` is built around the TLV320AIC3100 audio codec for Precursor/Renode feature configurations, not a new Baochip UDMA I2S service. Its hostaudio backend is stubbed.

Therefore the silicon block is real, but there is no ready-made Xous I2S TX driver in the reviewed HAL. A driver/service needs to be implemented and tested.

### 5.5 Important physical pin-routing caveat

The RTL exposes dedicated I2S master and slave pads through the interface subsystem. The signal names include:
- `i2sm_sd`, `i2sm_ws`, `i2sm_sck` for master output.
- `i2ss_sd`, `i2ss_ws`, `i2ss_sck` for slave interface.

The SoC RTL connects these to dedicated interface-subsystem pad wires and the pad mux has alternate-function routes for them. However, the inspected Dabao software pin setup does not configure an I2S interface, and the exact routing/exposure to Dabao board header pins has not yet been proven from the board schematic and package bonding.

Do not assume the master I2S pins are available on the current breadboard headers just because the RTL names exist. Verify:
1. Pin mux route from the SoC pad to I2S master.
2. Package bond-out and board schematic.
3. Dabao board header access.
4. Conflict with existing functions on those pins.

### 5.6 A possible clock-generation issue to investigate

The master divider is only 8 bits and derives from the peripheral clock. The Xous clock plan lists PCLK at 43.75 MHz, but the precise I2S peripheral clock source/gating and divider interpretation should be confirmed in the built silicon configuration. If it is 43.75 MHz and the output divider formula above applies, an integer divider gives discrete frequencies. For example D=6 gives about 3.125 MHz BCLK; with a 64-bit stereo frame that corresponds to about 48.83 kHz sample rate.

That is within the broad MAX98357A sample-rate range, but confirm the actual supported-clock interpretation and measure the output waveform. Do not hard-code 48 kHz based on a nominal calculation.

---

## 6. Comparing the two Baochip playback paths

| Consideration | PWM + RC + analogue amp | UDMA I2S + MAX98357A |
|---|---|---|
| Core audio idea | Variable PWM duty approximates an analogue voltage | Hardware serialises PCM samples |
| Timing-sensitive signal generation | PWM hardware generates carrier; software updates duty | Dedicated I2S hardware generates BCLK/WS/data |
| External filter | Yes, RC low-pass | No analogue RC needed on I2S data |
| Amplifier type | Analogue-input amplifier | Digital I2S-input amplifier |
| Existing MAX98357A usable? | No | Yes |
| Public Armstrong driver | PWM driver exists | No I2S driver found |
| Public sample player | Not found | Not found in the reviewed Baochip Xous HAL |
| Xous driver | Need PWM driver/service | Need UDMA I2S driver/service |
| Main risk | Smooth/regular duty updates and filter response | Pin exposure, DMA configuration and I2S format/clock setup |
| Expected complexity | Lower for a simple mono tone | Higher initial driver work, cleaner scalable audio path |

The existing BIO approach is a third route: custom software-driven I2S. It can remain a fallback or learning experiment, but it should no longer be the assumed best way to get audio from the Baochip.

---

## 7. Recommended engineering decision

### If the objective is simply to hear sound quickly

A PWM prototype with an analogue-input amplifier is probably the shortest path:
1. Confirm the PWM pin is accessible on the Dabao header (Armstrong's public example says PB1/header pin 29).
2. Use a modest high PWM carrier and a small sine table.
3. Add the suggested 1 kΩ series resistor and 100 nF shunt capacitor as a first-order low-pass.
4. Feed the filtered node into an analogue-input amplifier.
5. Start at a low amplitude and verify the waveform and amplifier input before connecting the speaker.

Do not reuse the MAX98357A for this topology; it expects digital I2S.

### If the objective is to keep MAX98357A and build a proper audio path

Investigate the dedicated UDMA I2S TX driver:
1. Confirm physical master I2S pin availability on Dabao.
2. Determine the exact I2S peripheral clock rate and valid divider values.
3. Read the generated register map and RTL for register update/enable ordering.
4. Inspect the generic uDMA channel implementation, event mapping and IFRAM/buffer requirements.
5. Implement a small, resource-managed Xous I2S TX service with bounded DMA completion and stop/cleanup.
6. Start with a tiny fixed PCM buffer and a low-level sine sample, then measure BCLK/LRCLK/DIN at the MAX98357A.

### Recommendation

Given the user already owns and wired the MAX98357A, the most useful next step is to establish whether the dedicated UDMA I2S master pins are accessible on Dabao and whether the Xous uDMA infrastructure can drive the I2S TX channel. If the physical pin route is not available on the headers, the PWM/analogue-amplifier topology is the more practical hardware path.

No change to existing BaoBAB firmware or user wiring is implied by this note.

---

## 8. Primary source references

### Armstrong's direct/public work
- Public SDK: https://github.com/ArmstrongSubero/dabao-sdk
- PWM API header: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/src/bao1x/hardware_pwm/include/hardware/pwm.h
- PWM driver: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/src/bao1x/hardware_pwm/pwm.c
- PWM LED example: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/pwm/pwm.c

### Baochip RTL and registers
- I2S register reference: https://ci.betrusted.io/bao1x/udma_i2s.html
- I2S RTL top: https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/udma_i2s_top.sv
- I2S register interface: https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/udma_i2s_reg_if.sv
- I2S TX channel: https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/i2s_tx_channel.sv
- I2S clock/WS generator: https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/i2s_clkws_gen.sv
- I2S uDMA integration: https://github.com/baochip/baochip-1x/blob/main/rtl/modules/ifsub/rtl/udma_sub.sv
- Baochip pin functions: https://github.com/baochip/baochip-1x/blob/main/docs/src/ch01-00-electrical-specifications.md
- PWM register reference: https://ci.betrusted.io/bao1x/pwm.html

### Xous software
- uDMA API: https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-api/src/udma.rs
- uDMA HAL module: https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/udma/mod.rs
- uDMA I2C driver (useful style/reference for a new driver): https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/udma/i2c.rs



## 9. Dabao board-level pinout cross-check (important practical constraint)

Source inspected:
- Dabao pin list: https://github.com/baochip/dabao/blob/main/docs/pinout/pins.csv
- Pinout documentation: https://github.com/baochip/dabao/tree/main/docs/pinout
- SoC pad mux: https://github.com/baochip/baochip-1x/blob/main/rtl/modules/ifsub/rtl/soc_ifsub.sv

### PWM is physically exposed

The Dabao pin CSV confirms:
- PB1 is physical header pin **29**.
- Its alternate function is **PWM1 channel 1**.
- PB1 is therefore a direct match for Armstrong's public `examples/pwm/pwm.c` wiring comment, which says PB1/header pin 29 and uses PWM slice 1/channel 1.

This makes Armstrong's PWM output topology demonstrably practical on the Dabao headers without relying on BIO pins.

### I2S master output is not listed on the current Dabao header

The SoC interface RTL contains dedicated I2S master pads (`i2sm_sd`, `i2sm_ws`, `i2sm_sck`). In `soc_ifsub.sv`, the pad mux connects the I2S master signals to the D-port pad mux routes:
- D10 AF1 -> `i2sm_sd` (master serial data).
- D11 AF1 -> `i2sm_ws` (word-select / LRCLK).
- D12 AF1 -> `i2sm_sck` (serial bit clock).

However, the Dabao board's published `pins.csv` lists external GPIO/header signals from PC13 down through PB pins and does not list any PD/D10-D12 header pads. The current public physical header pinout therefore does **not** provide an identified header connection for the I2S master trio.

This does not prove the pads are unbonded or impossible to reach through some alternate test point/revision; it does mean we cannot responsibly assume the current Dabao board exposes them for breadboard wiring. Check the package pin map, board schematic/PCB and any unpopulated test points before attempting a UDMA I2S hardware driver for the external MAX98357A.

### Updated recommendation

Given the user's current Dabao board and the MAX98357A breakout:
- The PWM option has a known exposed pin (PB1 / header pin 29), but requires an **analogue-input amplifier**, not the MAX98357A.
- The UDMA I2S TX block exists in silicon RTL, but its master pins are not shown on the current Dabao external header pin list. A Rust driver alone would not solve the physical access problem.
- The existing BIO I2S path still uses the current accessible pins PC8/PC10/PC11 and remains compatible with the MAX98357A electrically, subject to fixing/validating the software waveform.

Therefore, the best immediate route depends on whether the goal is to use the hardware already on the bench (keep MAX98357A -> validate/rework BIO I2S) or to try Armstrong's simpler analogue-output approach (PWM on PB1 -> RC filter -> obtain an analogue-input amplifier). Do not change the current soldering or wiring based only on the UDMA register map.
