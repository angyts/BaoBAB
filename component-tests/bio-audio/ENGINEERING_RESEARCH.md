# BIO I2S Engineering Research Notes

**Research date:** 2026-11-26  
**Project:** BaoBAB / Dabao / MAX98357A  
**Purpose:** compare real BIO and PIO serial-output implementations, audit the current BaoBAB transmitter, and assess the four-terminal speaker observation.

## Executive summary

1. The public `ArmstrongSubero/dabao-sdk` repository contains useful Baochip BIO reference programs: a measured 100 kHz square-wave output, divider tests, FIFO tests, and GPIO pad-readback tests. Its public examples tree does **not** currently show a ready-made I2S/BIO driver.
2. The public `earlephilhower/arduino-pico` I2S implementation is a strong architecture reference. It uses RP2040 PIO side-set to drive BCLK and word-select in a fixed instruction cadence while the output shift register serialises data. It is not directly portable to Baochip BIO, but its timing discipline is relevant.
3. A source audit confirms that BaoBAB's current `i2s_audio_kernel.rs` has data-dependent branch paths inside each bit cell and computes reported BCLK/LRCLK values as if two quantum waits alone set the entire period. They do not: ordinary BIO instructions between waits also consume core cycles. The reported values are therefore nominal estimates, not actual measured output frequencies. This requires correction before calling the output clock validated.
4. The same MAX98357A timing specification calls for minimum 15 ns BCLK high and low times, 10 ns data and LRCLK setup/hold, and standard-I2S one-BCLK delay. A logic-analyser or oscilloscope capture at the amplifier input is now the decisive test.
5. A public teardown describes a miniature 8 ohm / 0.25 W speaker with four rear terminals, where only the two outer terminals are used. That is a close match to the user's description, but it does not identify the user's exact speaker. The user's 8.6 ohm DC reading is compatible with an 8 ohm nominal single-coil speaker.

No firmware was changed by this research note. Do not increase the test amplitude further until the speaker rating and I2S waveform are verified.

---

## 1. Baochip BIO references from other engineers

### 1.1 Armstrong Subero — Dabao SDK

Repository: https://github.com/ArmstrongSubero/dabao-sdk

The SDK README describes a C SDK for Baochip-1x with GPIO, UART, SPI, I2C, PWM, ADC, BIO, DMA and examples. The BIO examples include:
- `bio_blink`: square-wave output on PB2 using BIO core 0.
- `bio_div`: tests integer/fractional quantum divider periods and synchronous core starts.
- `bio_fifo`: tests host/core FIFO behaviour and event-level signalling.
- `bio_gpio`: tests GPIO output masks, pad readback, events, and core clock counter.
- `bio_snap`: tests synchronised GPIO input/output sampling.

The public examples directory does not list an I2S or audio transmitter. I did not find a public BIO I2S implementation in this repository.

#### bio_blink: the smallest known-good BIO waveform pattern

File: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/bio_blink/bio_blink.c

The program does the following:
1. Loads a small instruction stream into BIO core 0.
2. Configures one GPIO bit as an output.
3. Sets the output HIGH.
4. Executes `x20` to wait for the next configured quantum.
5. Clears the output LOW.
6. Executes `x20` again.
7. Loops.

The source comments identify the expected output as a 100 kHz square wave on PB2, to be verified with an oscilloscope. The code configures an integer divider of 1750 from a 700 MHz BIO clock. This is a useful reference for building a deterministic edge generator with little work between output changes.

**Lesson for BaoBAB:** keep the edge-generating part of a serial bit-cell extremely small and predictable. Do not place a data-dependent branch chain between BCLK edges.

#### bio_div: verify divider behaviour rather than assuming it

File: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/bio_div/main.c

The comments and tests explain:
- Integer divider values produce a fixed quantum period.
- Fractional divider settings can alternate between neighbouring periods.
- Two BIO cores started together can run their divider quanta in step.
- The program measures core-cycle timestamps between quanta through FIFO output.

**Lesson for BaoBAB:** a configured quantum frequency does not automatically equal a pin's output frequency. The pin waveform depends on the entire instruction sequence between GPIO transitions. The transmitter must either count those cycles or measure the signal.

#### bio_gpio: the pad readback path has latency

File: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/bio_gpio/main.c

This example explicitly documents that GPIO pad input readback through `x21` lags the output write. Its comments describe two cycles of pad latency and two more cycles of input synchronizer latency, and it intentionally inserts delay before reading the changed pin state.

**Lesson for BaoBAB:** the earlier `i2sclock pintest` demonstrates pin-mask/mux/readback behaviour, but its readback timing is not a substitute for a high-speed oscilloscope/logic-analyser capture. A dynamic internal self-test should allow adequate pad/synchronizer settling and should not be treated as external timing verification.

#### bio_fifo: FIFO semantics

File: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/bio_fifo/main.c

The example describes BIO FIFOs as eight storage words plus the output register, tests host/core producer-consumer behaviour, and exercises FIFO-level events.

**Lesson for BaoBAB:** FIFO completion is valuable evidence that the core consumed the programmed sample buffer; it does not prove the output waveform met the I2S electrical timing.

### 1.2 Xous BIO examples

Relevant current Xous source:
- BIO API: https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-api/src/bio.rs
- BIO resource model: https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-api/src/bio_resources.rs
- BIO Rust HAL: https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/bio.rs
- BIO hardware implementation: https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/bio_hw.rs
- WS2812 BIO example: https://github.com/betrusted-io/xous-core/blob/main/libs/bio-lib/src/ws2812.rs

The Xous WS2812 example demonstrates the correct software ownership pattern: claim resources through the BIO resource manager, initialise the BIO core, map pins, start the program, feed data via FIFO, stop, and release resources.

I did not find a ready-made I2S transmitter in the public Xous BIO library source. The WS2812 example is a useful resource/FIFO pattern, but it is not an I2S bit-cell timing template.

---

## 2. RP2040 PIO I2S reference implementation

Repository: https://github.com/earlephilhower/arduino-pico  
I2S assembly: https://github.com/earlephilhower/arduino-pico/blob/master/libraries/I2S/src/pio_i2s.pio

The `pio_i2s_out` program uses:
- A PIO state machine output shift register for data.
- `side_set 2` to drive BCLK and word-select (LRCLK) as part of each PIO instruction.
- A fixed sequence of `out pins` and `jmp x--` instructions for channel bit loops.
- A special final-bit instruction to transition word-select at the I2S frame boundary.
- A TX FIFO, joined for transmission, with DMA support in the driver layer.

Relevant PIO code (source is linked above):
- `out pins, 1 side 0b00`
- `jmp x--, left1 side 0b01`
- `out pins, 1 side 0b10` for the channel boundary
- Similar fixed instruction cadence for the right channel.

This architecture is especially relevant because the PIO engine changes clock outputs as a side effect of every instruction. BCLK and LRCLK timing follows the state-machine instruction cadence rather than a sequence of branches in host CPU code.

**What can be adapted to Baochip BIO:**
- Use one fixed-length loop for each serial bit.
- Keep clock output changes in the same instruction positions on every bit.
- Keep zero and one data bits on equal-cycle paths.
- Keep sample data in a shift register or a precomputed bitstream; avoid conditionals in the critical edge loop.
- Use FIFO/DMA or a carefully bounded FIFO producer for samples.
- Toggle LRCLK only at a deterministic frame boundary and include the I2S one-bit delay.

**What cannot be copied directly:** RP2040 PIO has hardware side-set, shift registers, and a particular PIO instruction set. Baochip BIO is a PicoRV32-like programmable core with special GPIO/FIFO/quantum registers. The structure must be re-expressed and cycle-counted for BIO.

A second reference is the official Arduino-ESP32 I2S API, which documents standard I2S with left and right slots and configurable 8/16/24/32-bit slot widths. This is a peripheral-driver reference rather than a BIO reference:
https://docs.espressif.com/projects/arduino-esp32/en/latest/api/i2s.html

---

## 3. Review of current BaoBAB audio kernel

File: `i2s_audio_kernel.rs` in this directory.

### 3.1 What is aligned with the datasheet

The source currently:
- Uses PC8/BIO24 as DIN, PC10/BIO26 as LRCLK, PC11/BIO27 as BCLK.
- Uses low LRCLK for left and high for right.
- Inserts one BCLK period after each LRCLK transition before the first data bit.
- Uses 32 clocks per channel slot and 64 clocks per stereo frame.
- Places a 16-bit signed sample in the upper 16 sample positions of a 32-bit I2S channel slot, with the lower 15 unused bits zero after the I2S delay bit.
- Duplicates mono samples into the left and right slots.

These format choices are consistent with the MAX98357A supporting standard I2S, 16/24/32-bit resolution and 32/48/64 BCLKs per LRCLK. The one-BCLK delay is required for MAX98357A standard I2S.

### 3.2 Confirmed source-level timing issue

The current kernel contains a conditional path for each data bit:
- Test whether the sample MSB is zero or one.
- A one bit executes a GPIO set followed by a jump.
- A zero bit executes a GPIO clear without that jump.
- The code then waits on the quantum and performs BCLK high/low transitions.

Thus the sequence between BCLK edges is not instruction-identical for zero and one bits. The low phase also includes sample shifting, loop-counter work and branch instructions before the next bit begins.

The host-side report currently calculates:
- BCLK = actual quantum / 2
- LRCLK = actual quantum / 128

Those formulas account only for the two explicit quantum waits per BCLK period and omit the BIO instructions that execute between the waits. The values should be regarded as nominal divider-based estimates, not actual measured pin frequencies. Since LRCLK toggles every 32 BCLK cycles, the clock ratio should remain 64 BCLK per full LRCLK frame, but the absolute output rates are not established by the console printout.

A one-instruction difference at a 700 MHz BIO core rate is about 1.43 ns; the exact effective BCLK and jitter must be derived from the actual BIO clock, instruction timing, divider, and execution paths, or better, measured at the output pin. Do not assert that the timing defect alone explains the silent speaker: the size of the actual deviation and the MAX98357A's tolerance have not been measured.

### 3.3 Recommended redesign

Before adding more volume or streaming support:
1. Make each data-bit cell a fixed number of BIO core cycles for both 0 and 1.
2. Ensure data is established before the BCLK rising edge and remains stable through the required hold interval.
3. Use `x21` masked GPIO write or another fixed-length branch-free bit selection strategy where it makes the output sequence simpler. Verify the actual BIO x21 write semantics before relying on it.
4. Use the quantum wait as a calibrated coarse timing interval, and include all non-quantum instructions in the bit-period calculation.
5. Make BCLK high and low intervals deterministic and above the datasheet minimum pulse widths.
6. Keep LRCLK transitions and the one-bit delay in exact, frame-counted positions.
7. Keep the BIO core generating BCLK and LRCLK continuously during the test; stop clocks only in a deliberately designed mute/shutdown sequence, with SD_MODE handling considered.
8. Add assembler/source comments with cycle counts for each bit-cell path.
9. Add a small deterministic test pattern (e.g. alternating 0xAAAA and 0x5555, then 0x0000 and 0xFFFF) before returning to sine samples.
10. Measure BCLK, LRCLK, DIN at the MAX98357A input pins. Confirm BCLK frequency, BCLK/LRCLK ratio, duty cycle, I2S one-bit delay, DIN setup/hold, and LRCLK startup timing.

### 3.4 MAX98357A requirements worth checking

Official datasheet (Rev. 16, Feb 2026):
https://www.analog.com/media/en/technical-documentation/data-sheets/MAX98357A-MAX98357B.pdf

Important items:
- MAX98357A uses standard I2S; MAX98357B uses left-justified timing. Confirm the actual module IC says MAX98357A, not MAX98357B.
- The MAX98357A samples data on BCLK rising edge.
- Standard I2S needs one BCLK of delay after LRCLK transition.
- It supports 16/24/32-bit I2S data.
- BCLK must be 32, 48, or 64 times LRCLK.
- Electrical characteristics list BCLK high and low minimum pulse widths of 15 ns, and 10 ns DIN/LRCLK setup and hold.
- Table 5 selects left when SD_MODE is above the B2 trip point, right between B1 and B2, mono average between B0 and B1, and shutdown below B0.
- B2 threshold range is 1.245 V min / 1.4 V typical / 1.5 V max; a measured 1.5 V has little/zero margin and should not be treated as a robustly established channel configuration.
- MAX98357A supports 48 kHz LRCLK. The datasheet says LRCLK must start within half an LRCLK period of BCLK starting.
- Do not let BCLK continue while removing/interrupting LRCLK; the datasheet warns that this can cause unexpected output including a large DC output voltage.

Current user's measurements:
- Amplifier VIN-to-GND: 3.3 V.
- SD_MODE-to-GND: 1.5 V.
- Speaker disconnected resistance across the wired outer pair: 8.6 ohms.
- User reports the DIN/LRCLK/BCLK static line-hold checks all measured the expected high voltage at the amplifier end.
- BIO pad readback pin test: 4/4 PASS.
- BIO/FIFO 440 Hz command completed, but user reports no audible sound.

Static line tests do not verify high-speed waveform timing. The next decisive electrical evidence is an oscilloscope/logic analyser capture during transmission.

---

## 4. Four-terminal miniature speaker observation

The user reports that the back of the speaker has four physical terminals and only the outer two are connected.

A closely matching published example exists:
- Margaret Liu, “Dissecting a Vibration Activated Sound Module” (2019): https://margaretliu.dev/posts/vibration-activated-sound-module/
- The article describes a small 8 ohm / 0.25 W miniature speaker with four terminals, where only the two far-end terminals are soldered to the board. It notes the unused terminal arrangement but does not establish exactly why every terminal is present.
- This is a close match to the user's description, but it is not proof that the user's speaker is the same model.
- The user's 8.6 ohm DC reading across the wired outer pair is consistent with an 8 ohm nominal speaker.

Do not assume that four physical terminals automatically mean a dual-voice-coil speaker. There are at least two possible designs:
1. A single voice coil whose connection tabs are duplicated or whose unused tabs support an alternate assembly/wiring option.
2. A dual-voice-coil speaker with two electrically separate coil pairs.

A conventional dual-voice-coil speaker normally has two isolated coil pairs. Manufacturer documentation for DVC drivers shows four terminals arranged as two independent +/− pairs, which can be wired independently, in series, or in parallel. Example: https://support.garmin.com/en-US/?faq=4GMOh9778Z32BLUe5ntru5

### Safe way to identify the terminals next time

With the speaker completely disconnected from the amplifier:
1. Number the four contacts from left to right as T1, T2, T3, T4 (viewing the rear of the speaker).
2. Measure resistance for all six pairs: T1-T2, T1-T3, T1-T4, T2-T3, T2-T4, T3-T4.
3. Record the readings; do not apply external power to the speaker terminals.
4. Interpretation:
   - One clear pair around 8–9 ohms, with the other terminals unconnected/open, suggests a single coil using only those tabs.
   - Two separate finite-resistance pairs suggest two independent voice coils.
   - Near-zero pairs may indicate duplicated tabs that are internally common.
5. Keep the speaker connected only across MAX98357A OUT+ and OUT−. Neither speaker lead should be connected to ground or to the 3.3 V supply.

Look for a marking on the speaker back such as `8Ω 0.25W`, `8Ω 0.5W`, or a part number before deciding the software amplitude ceiling. Do not assume the earlier 0.5 W estimate was correct.

---

## 5. Recommended order of work

1. Do not change volume/amplitude further.
2. Confirm the exact MAX98357 variant and breakout schematic if the board markings are available.
3. Check the speaker terminal resistance matrix and rating marking.
4. Redesign the BIO bit-cell with fixed instruction timing and an accurate divider-to-output frequency calculation.
5. Compile and review the kernel before flashing.
6. Run a deterministic digital pattern at a low level; capture BCLK/LRCLK/DIN using a logic analyser or oscilloscope at the amplifier header.
7. Only after the serial protocol is measured and validated, try the 440 Hz tone again.
8. If valid I2S is present at the IC and the speaker is confirmed wired to OUT+/OUT−, investigate the MAX98357A breakout's SD_MODE/Gain resistor configuration and supply/analog output.

