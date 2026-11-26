# BaoBAB audio research — consolidated findings and bench record

**Last consolidated:** 2026-11-26  
**Target:** Baochip-1x on Dabao / Xous  
**Current audio parts:** MAX98357A I2S amplifier breakout and small four-terminal speaker  
**Purpose:** one entry point for all research, implementation history, actual bench results, architecture alternatives and next decisions.

> **Current outcome:** the BIO-based 440 Hz PCM command completes and releases resources, but the user hears no sound. GPIO readback and static external wire-level tests passed. No oscilloscope/logic-analyser capture has verified the live I2S waveforms. Armstrong Subero has clarified that his own Dabao audio-output design uses PWM + low-pass + an analogue-input amplifier, not the MAX98357A I2S path.

---

## 1. Plain-English summary

We investigated how to get audio out of the Baochip.

There are three approaches:

1. **BIO-generated I2S:** a small programmable processor generates BCLK, LRCLK and serial audio data. This can use the MAX98357A already on the bench. Our prototype sends PCM data and completes its FIFO transfer, but it produces no audible sound yet. Its exact live waveform has not been measured.
2. **Dedicated UDMA I2S:** the chip RTL contains a real hardware I2S transmitter and DMA channel. However, the public Xous HAL currently does not expose an I2S driver, and the I2S master pins have not been identified on the Dabao header. A driver alone may not make this usable on the board.
3. **PWM + RC low-pass + analogue amplifier:** Armstrong Subero directly told the user that this is how he gets audio output on Dabao. The public SDK has a PWM driver and an LED example. Dabao exposes PWM1 channel 1 on PB1 / physical header pin 29. This approach requires an **analogue-input amplifier**; the current MAX98357A is a digital I2S-input amplifier and cannot accept the filtered PWM audio.

**Decision framing:** if keeping the current MAX98357A is important, keep investigating/fixing I2S, starting with measured waveforms and pin/clock correctness. If the immediate goal is simply to hear a tone using Armstrong's simpler design, PWM on PB1 plus an appropriate analogue-input amplifier is the shorter hardware route.

No claim is made that a finished music player exists in Armstrong's public SDK. Its PWM example is an LED breathing demo. The user's direct message from Armstrong provides the PWM audio topology, but not a complete public implementation of PCM playback.

---

## 2. What has actually been tested on the user's hardware

| Test | Result | What it proves | What it does not prove |
|---|---|---|---|
| I2C haptic component discovery | DRV2605L at 0x5A acknowledged | I2C communication with the haptic IC | Audio path |
| BIO clock test `i2sclock start/status/stop` | Completed; stopped and released pins | BIO resource lifecycle and configured clock setup | Measured BCLK/LRCLK pin frequency or audio |
| BIO GPIO pad audit `i2sclock pintest` | 4/4 PASS | BIO output mask and on-chip pad readback for PC8/PC10/PC11 | Jumper path, MHz timing or I2S framing |
| Static external line hold `i2sclock linehold din/lrclk/bclk` | User reports all three far-end voltages correct | Static DC continuity/level from selected Dabao pins to amplifier-side header | High-speed waveform integrity |
| BIO PCM tone command | FIFO transfer/completion succeeded; resources released | BIO consumed the programmed data and reached completion path | Valid I2S at the amplifier, amplifier output or audible sound |
| Tone at amplitude 512 | No sound | No audible output at this test condition | Exact root cause |
| Tone at 1024, 2048 and 4096 | User reports no sound | Increasing the conservative PCM levels did not solve silence | Speaker's safe maximum or amplifier power |
| Amplifier supply | VIN-to-GND = 3.3 V | Supply present at measurement point | Ripple under playback or IC condition |
| SD_MODE | 1.5 V measured | Static voltage at the breakout pad | Guaranteed channel selection margin or breakout circuit details |
| Speaker DC resistance | 8.6 ohms across the used outer terminals | A coil path consistent with a nominal 8-ohm speaker | Power rating, exact model or what the other two tabs do |
| Four-terminal speaker inspection | Four physical contacts; only outer two are soldered | Mechanical/terminal layout observation | Single vs dual coil; all terminal pair relationships |

### Interpretation

The following hardware/software checks passed:
- BIO can take ownership of and drive the three selected pads.
- The pad readback returned expected masks.
- All three external wires carried a static HIGH level to the amplifier-side header.
- The host/BIO FIFO exchange completed without the original timeout.

The following remain unverified:
- Actual BCLK and LRCLK frequencies at the amplifier pins.
- Clock duty cycle and pulse widths.
- DIN bit order, I2S delay, word/channel framing at the amplifier.
- Whether the amplifier IC is exactly MAX98357A (rather than another family variant).
- The exact breakout SD_MODE/GAIN network and its threshold margin.
- The actual speaker rating and four-terminal internal connectivity.
- Analog output from the amplifier.

A static multimeter test is not a dynamic serial-protocol test. The next decisive measurement for the current MAX98357A setup is a logic-analyser/oscilloscope capture of DIN, BCLK and LRCLK at the amplifier input while a tone is running.

---

## 3. Current BIO I2S implementation and findings

### 3.1 Signal assignments

| MAX98357A input | Dabao chip pin | BIO bit |
|---|---|---:|
| DIN | PC8 | BIO24 |
| LRC / LRCLK | PC10 | BIO26 |
| BCLK | PC11 | BIO27 |
| VIN | 3V3 | — |
| GND | GND | — |
| Speaker | OUT+ and OUT− | — |

PC9/BIO25 was initially considered for DIN, but the console BIO loader's available-pin allow-list did not include it. The implementation moved DIN to PC8/BIO24.

The speaker must remain connected only across amplifier OUT+ and OUT−. Neither output should be connected to GND.

### 3.2 Intended data format

The prototype attempts:
- Standard I2S, with a one-BCLK delay after LRCLK transition.
- 32 BCLKs per channel slot.
- 64 BCLKs per stereo frame.
- 16-bit signed PCM placed in the upper part of each 32-bit slot.
- Same mono sample sent to both left and right channels.
- 440 Hz, 250 ms test tone with short fades.
- BIO FIFO1 for host-to-core data and FIFO2 for completion.
- Resource-manager ownership of one BIO core, two FIFOs and pins 24/26/27.

### 3.3 First failure and corrections

The first tone attempt timed out with `BIO TX FIFO did not drain`. The observed FIFO level was 7, indicating that the core had not consumed the preloaded words.

The development iterations then corrected:
- The standard I2S one-bit delay after the LRCLK boundary.
- A sample-bit mask alignment issue.
- Use of reserved BIO FIFO register x16 as scratch storage.
- Startup ordering (BIO consumer starts before the host streams samples).
- FIFO timeout diagnostics.

After these changes the command completed and reported:
```text
Played 440 Hz test tone:
quantum=6140351 Hz
BCLK=3070175 Hz
LRCLK=47971 Hz
frames=11992
peak_sample=512
BIO resources released.
```

These displayed rates were divider-derived estimates. They were not captured from the physical pins.

### 3.4 Why silence is still unresolved

A source review found that the current BIO serialiser uses data-dependent control flow around bit output and performs ordinary BIO instructions between its timing waits. The host's reported BCLK/LRCLK calculations treat the waits as if they alone determined the periods. They do not account for all BIO instruction cycles.

This means the printed clock values are not actual measured pin frequencies. The code may produce timing variation or duty-cycle irregularity. However, **this source finding does not prove that the timing defect alone causes silence**. A roughly 3 MHz BCLK has a nominal half-period much longer than the MAX98357A's minimum pulse width, so only a real waveform capture can show whether the signal violates requirements.

Do not increase the PCM amplitude further as a substitute for waveform diagnosis.

---

## 4. BIO engineering references: other engineers' implementations

### 4.1 Armstrong Subero's public Dabao SDK

Repository: https://github.com/ArmstrongSubero/dabao-sdk

Relevant public code:
- [BIO driver source](https://github.com/ArmstrongSubero/dabao-sdk/tree/main/src/bao1x/hardware_bio)
- [BIO API header](https://github.com/ArmstrongSubero/dabao-sdk/tree/main/src/bao1x/hardware_bio/include/hardware)
- [BIO blink example](https://github.com/ArmstrongSubero/dabao-sdk/tree/main/examples/bio_blink)
- [BIO divider example](https://github.com/ArmstrongSubero/dabao-sdk/tree/main/examples/bio_div)
- [BIO FIFO example](https://github.com/ArmstrongSubero/dabao-sdk/tree/main/examples/bio_fifo)
- [BIO GPIO example](https://github.com/ArmstrongSubero/dabao-sdk/tree/main/examples/bio_gpio)
- [BIO snapshot example](https://github.com/ArmstrongSubero/dabao-sdk/tree/main/examples/bio_snap)
- [PWM driver](https://github.com/ArmstrongSubero/dabao-sdk/blob/main/src/bao1x/hardware_pwm/pwm.c)
- [PWM API](https://github.com/ArmstrongSubero/dabao-sdk/blob/main/src/bao1x/hardware_pwm/include/hardware/pwm.h)
- [PWM LED demo](https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/pwm/pwm.c)

The public repository inspection found useful BIO/GPIO/divider/FIFO/PWM examples, but no public I2S transmitter, PCM music player or audio sample-streaming example in the examined main branch.

Useful lessons:
- `bio_blink` makes a 100 kHz square wave and expects oscilloscope verification.
- `bio_div` measures timing/divider behaviour rather than relying only on nominal calculations.
- `bio_gpio` documents pad-input readback latency; a just-written output may not be visible immediately.
- `bio_fifo` demonstrates finite FIFO depth and host/core producer-consumer behaviour.
- The BIO DMA example reserves a BIO core for memory transfers; core/resource ownership must be considered.
- The PWM demo uses PB1 / PWM1 channel 1 but is an LED breathing program, not audio playback.

### 4.2 Armstrong's direct answer about audio

The user reports receiving this direct clarification from Armstrong:

> On the Dabao it’s just PWM through a low pass filter, when in doubt 1k and 100nF… then feed it into an amp and that’s it…audio…well audio out at least..audio in you’d need a mic and op amp to keep the sound within the Dabao’s bang gap range realistically under a volt and sample that…

This describes:
- Audio output: PWM duty changes over time → low-pass filter → analogue-input amplifier → speaker.
- Audio input: microphone → analogue gain/conditioning stage → ADC within the permitted input range.

The direct response gives a topology and suggested starting RC values, but does not publish the complete PCM sample generator, sample timing, audio carrier selection, buffer design or filter optimisation.

---

## 5. PWM audio route

### 5.1 Public driver and pin

Armstrong's PWM demo:
- Selects PB1 alternate function AF3.
- Uses PWM slice 1, channel 1.
- Starts a 1 kHz PWM carrier for an LED.
- Changes duty cycle gradually.

The Dabao board pinout confirms PB1 is physical header pin **29**, PWM1 channel 1.

Sources:
- https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/pwm/pwm.c
- https://github.com/ArmstrongSubero/dabao-sdk/blob/main/src/bao1x/hardware_pwm/pwm.c
- https://github.com/baochip/dabao/blob/main/docs/pinout/pins.csv

### 5.2 PWM + RC concept

```text
Sample value / duty update
          |
          v
Baochip PWM pin (PB1 / header 29)
          |
          v
1 kΩ series resistor
          |
          +------ filtered analogue-like output ------> analogue amplifier input
          |
        100 nF
          |
         GND

Analogue amplifier output -> speaker
```

For 1 kΩ and 100 nF, the simple first-order RC corner is approximately:
```text
fc = 1 / (2πRC) ≈ 1.59 kHz
```

This can be a simple, low-bandwidth starting point. It is not a hi-fi filter: it attenuates higher audio frequencies and its actual response depends on source impedance and amplifier input impedance. The RC node is a signal input; never connect the speaker directly to the GPIO/filter.

### 5.3 Amplifier compatibility warning

The present MAX98357A is a **digital I2S-input** amplifier. It expects DIN synchronised to BCLK/LRCLK. It does not accept the analogue-like filtered PWM waveform.

To try Armstrong's PWM approach, obtain an analogue-input amplifier module suitable for the speaker. To keep the MAX98357A, deliver valid I2S data.

---

## 6. Dedicated UDMA I2S peripheral

### 6.1 Silicon implementation

The Baochip RTL contains a real UDMA I2S peripheral with:
- Master TX channel.
- Slave RX channel.
- Master SD0 and SD1 outputs.
- Slave SD0 and SD1 inputs.
- Configurable master/slave word lengths, word counts, channel mode and bit order.
- Programmable master/slave/common clock division.
- TX/RX DMA channel integration and clock-domain crossing.
- PDM/CIC receive path.

Primary source files:
- [I2S top](https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/udma_i2s_top.sv)
- [Register interface](https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/udma_i2s_reg_if.sv)
- [TX serializer](https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/i2s_tx_channel.sv)
- [RX channel](https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/i2s_rx_channel.sv)
- [Clock/word-select generator](https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/i2s_clkws_gen.sv)
- [Register reference](https://ci.betrusted.io/bao1x/udma_i2s.html)
- [uDMA subsystem integration](https://github.com/baochip/baochip-1x/blob/main/rtl/modules/ifsub/rtl/udma_sub.sv)

### 6.2 Register map and semantics

The generated register reference gives base address `0x5010E000`:

| Offset | Register | Key purpose |
|---:|---|---|
| 0x00 | RX_SADDR | RX DMA source/destination address register |
| 0x04 | RX_SIZE | RX transfer byte count |
| 0x08 | RX_CFG | RX clear, enable, data-size, continuous bits |
| 0x10 | TX_SADDR | TX buffer address |
| 0x14 | TX_SIZE | TX byte count |
| 0x18 | TX_CFG | TX clear, enable, data-size, continuous bits |
| 0x20 | I2S_CLKCFG_SETUP | Clock source selection, clock enables and dividers |
| 0x24 | I2S_SLV_SETUP | Slave I2S word format |
| 0x28 | I2S_MST_SETUP | Master I2S word format |
| 0x2C | I2S_PDM_SETUP | PDM receive configuration |

Source-level details from `udma_i2s_reg_if.sv`:
- TX_CFG: bit 5 is clear, bit 4 is enable, bits 2:1 are data size, bit 0 is continuous mode.
- I2S_CLKCFG_SETUP: bits 31:28 select master/slave clock sources; bit 26 PDM clock enable; bit 25 master clock enable; bit 24 slave clock enable; bits 23:16 common divider; bits 15:8 slave divider; bits 7:0 master divider.
- I2S_MST_SETUP: bit 31 master-I2S enable; bit 17 two-channel mode; bit 16 LSB-first; bits 12:8 word length; bits 2:0 number of words. The RTL only accepts master-format writes while master clock enable is low.
- Clock source/divider configuration is copied into the peripheral clock domain through an update mechanism triggered by writing the clock-setup register. A driver must respect this clock-domain handoff; writing the fields alone is not enough to assume they are active.

The register semantics must be checked against the exact RTL revision and peripheral clock configuration before implementation.

### 6.3 Current Xous support as inspected

In public Xous source examined on 2026-11-26:
- `libs/bao1x-api/src/udma.rs` defines an I2S peripheral ID and TX/RX events.
- `libs/bao1x-hal/src/udma/mod.rs` exports ADC, I2C, SPI master and UART drivers.
- No `i2s.rs` driver is present/exported from that UDMA HAL module.
- Baochip emulator uDMA clock/event/reset functions are stubs in the inspected source.
- Therefore the RTL exists, but a ready-to-use Xous I2S TX driver was not found.

A future driver would need to handle the peripheral clock/reset, register setup, TX uDMA channel, buffer lifetime/accessibility, completion events/timeouts, stop and cleanup. It should use Xous resource ownership patterns.

### 6.4 Critical physical pin caveat

The SoC RTL contains I2S master pad signals:
- `i2sm_sd`
- `i2sm_ws`
- `i2sm_sck`

The pad mux source routes these through D-port pad functions (the inspected source shows D10/D11/D12 routes). However, the public Dabao `pins.csv` does not list these D-port pads among the external header pins. This means a convenient breadboard connection for the UDMA I2S master trio has **not been established** for the current Dabao board. It does not prove that no test point or alternate board revision exposes them; the schematic, package and PCB/test-point access need inspection.

By contrast, PB1/PWM1 channel 1 is positively identified on physical header pin 29.

---

## 7. Speaker and amplifier notes

User-reported measurements:
- MAX98357A VIN-to-GND: 3.3 V.
- SD_MODE-to-GND: 1.5 V.
- Speaker resistance across the connected outer terminals with speaker disconnected: 8.6 ohms.
- Four physical speaker terminals visible; only the two outer terminals are soldered to the wiring.

Interpretation:
- 8.6 ohms DC is compatible with an 8-ohm nominal single speaker coil, but does not prove the speaker's power rating.
- Four tabs do not by themselves prove a dual-voice-coil speaker. A resistance matrix across all six terminal pairs is needed to identify the internal connections.
- Do not assume the speaker is 0.5 W. Find a part marking/datasheet or identify it before selecting a power level.
- MAX98357A SD_MODE at 1.5 V is not in the shutdown region, but is near the upper channel-selection threshold described in the datasheet. The exact breakout resistor network and threshold margin should be verified. It should not be treated as a measured confirmation of a robust left/right selection state.
- Do not connect either MAX98357A OUT+ or OUT− to GND; the speaker is connected across the differential outputs.

MAX98357A datasheet:
https://www.analog.com/media/en/technical-documentation/data-sheets/MAX98357A-MAX98357B.pdf

---

## 8. Decisions and next engineering steps

### If keeping the current MAX98357A

1. Do not increase volume further.
2. Verify the exact amplifier IC marking and breakout schematic.
3. Capture DIN, BCLK and LRCLK at the MAX98357A header during a low-amplitude test with a logic analyser/oscilloscope.
4. Check measured BCLK/LRCLK ratio, pulse widths, sample timing, one-bit I2S delay and data setup/hold.
5. If the waveform is wrong, fix the BIO serializer and correct the frequency reporting from actual instruction timing.
6. If the waveform is valid at the IC pins, investigate SD_MODE, GAIN, amplifier output supply/thermal state and speaker terminal matrix.
7. Revisit UDMA I2S only after confirming physical access to its master pins.

### If trying Armstrong's PWM audio route

1. Confirm PB1 / physical header pin 29 availability and that it is not already in use.
2. Obtain an analogue-input audio amplifier compatible with the speaker and supply.
3. Build the 1 kΩ / 100 nF low-pass as an initial experiment, treating it as a low-bandwidth filter.
4. Generate a low-level fixed-table sine wave by periodic PWM-duty updates; the existing public SDK example does not implement this sample loop.
5. Measure PWM carrier, update rate, filtered waveform and amplifier input before connecting/raising speaker power.
6. Do not connect filtered PWM to MAX98357A DIN.

### Audio input / microphone

Armstrong states that a microphone needs an op-amp/preamp so the signal is within the Baochip ADC's input range (he says realistically under a volt). The ADC documentation specifies a limited analog input range; a microphone should not be connected directly without checking bias, gain, filtering, protection and the specific board analog input circuitry.

---

## 9. Detailed research files in this directory

- [README.md](README.md) — BIO I2S bench experiment, code files, commands and test procedure.
- [ENGINEERING_RESEARCH.md](ENGINEERING_RESEARCH.md) — BIO/PIO implementation comparisons, timing audit, MAX98357A requirements and four-terminal speaker investigation.
- [UDMA_I2S_AND_PWM_AUDIO_RESEARCH.md](UDMA_I2S_AND_PWM_AUDIO_RESEARCH.md) — dedicated I2S RTL, Xous support, Armstrong PWM implementation, RC filter and Dabao pinout comparison.
- [RESEARCH_SUMMARY.md](RESEARCH_SUMMARY.md) — this consolidated current-state report.

Implementation source in the same directory includes BIO clock, PCM audio, GPIO pad-audit and static line-hold kernels and their Xous wrappers/installers. These are experimental source artifacts, not a validated general-purpose audio driver.

---

## 10. Primary references

### Baochip and Dabao
- https://ci.betrusted.io/bao1x/
- https://ci.betrusted.io/bao1x/udma_i2s.html
- https://ci.betrusted.io/bao1x/pwm.html
- https://github.com/baochip/baochip-1x
- https://github.com/baochip/dabao/blob/main/docs/pinout/pins.csv
- https://github.com/baochip/dabao/tree/main

### Armstrong Subero
- https://github.com/ArmstrongSubero/dabao-sdk
- https://github.com/ArmstrongSubero/dabao-sdk/tree/main/examples/bio_blink
- https://github.com/ArmstrongSubero/dabao-sdk/tree/main/examples/bio_div
- https://github.com/ArmstrongSubero/dabao-sdk/tree/main/examples/bio_fifo
- https://github.com/ArmstrongSubero/dabao-sdk/tree/main/examples/bio_gpio
- https://github.com/ArmstrongSubero/dabao-sdk/tree/main/examples/pwm
- https://github.com/ArmstrongSubero/dabao-sdk/blob/main/src/bao1x/hardware_pwm/pwm.c

### Xous
- https://github.com/betrusted-io/xous-core
- https://github.com/betrusted-io/xous-core/tree/main/libs/bao1x-hal/src/udma
- https://github.com/betrusted-io/xous-core/tree/main/libs/bio-lib

### Independent PIO audio reference
- https://github.com/earlephilhower/arduino-pico/blob/master/libraries/I2S/src/pio_i2s.pio

### Amplifier
- https://www.analog.com/media/en/technical-documentation/data-sheets/MAX98357A-MAX98357B.pdf
