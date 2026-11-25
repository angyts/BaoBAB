# BIO-powered I2S audio experiment (Xous / Dabao)

**Status:** clock-only BIO test passed on hardware; BIO GPIO readback passed 4/4; the 440 Hz sample/FIFO path completed but no audible sound was heard. Static line-hold diagnostic is committed but not yet compiled or run.

## 1. Verified wiring

| MAX98357A | Dabao |
|---|---|
| VIN | 3V3 |
| GND | GND |
| DIN | PC8 / BIO24 |
| LRC | PC10 / BIO26 |
| BCLK | PC11 / BIO27 |
| GAIN | Leave open for initial bring-up |
| SD_MODE | Preserve the breakout's existing resistor network |
| OUT+ / OUT- | 8-ohm speaker across both outputs; neither side to GND |

PC9 / BIO25 was proposed first for DIN, but it is excluded from the Dabao console BIO loader allow-list. Use PC8 / BIO24 instead. Physical Dabao pinout: https://github.com/baochip/dabao/blob/main/docs/pinout/index.html

## 2. Clock-only test already passed

The user successfully ran:
```text
i2sclock start
i2sclock status
i2sclock stop
```

Reported divider-derived frequencies:
- BIO quantum: 3,070,175 Hz
- BCLK: 1,535,087 Hz
- LRCLK: 47,971 Hz
- stop: BIO core stopped and pins reported released

These values came from the configured BIO divider calculation, not an oscilloscope/logic-analyser measurement. This demonstrated BIO core start/stop and resource release, not valid audio data.

## 3. 440 Hz tone implementation

The new prototype uses standard I2S with 32-bit channel slots carrying 16-bit PCM:
- Target BIO quantum: 6.144 MHz.
- Target BCLK: 3.072 MHz.
- Target LRCLK: 48 kHz.
- 32 BCLKs per channel slot; 64 BCLKs per stereo frame.
- Standard I2S one-BCLK delay after each LRCLK transition. This was added after the first attempt omitted it.
- 16-bit signed sample occupies the upper 16 data bits after the delay; remaining 15 slot bits are zero.
- 250 ms tone with 10 ms fade-in/out.
- 64-entry sine table, peak magnitude 512 / 32767 (about 1.6% full scale).
- Mono sample duplicated to left and right channels.
- FIFO1 streams the frame count and sample words to BIO; FIFO2 provides a completion token. The host preloads three stereo frames (7 words including count) before starting BIO to reduce startup starvation risk.
- TX FIFO timeout now reports the FIFO level observed at timeout.
- BIO stops sample output after a full stereo frame, drives the outputs low, signals completion, then remains in a quiet loop until host cleanup.
- Xous producer checks FIFO water level and has a bounded FIFO/completion timeout.
- Resource manager claims one BIO core, FIFO1, FIFO2 and pins 24, 26, 27.

Manufacturer datasheet: https://www.analog.com/media/en/technical-documentation/data-sheets/MAX98357A-MAX98357B.pdf

The MAX98357A supports standard I2S and 8–96 kHz sample rates. Its timing diagram requires a one-BCLK delay from LRCLK transition to the next word's MSB. The 32-bit slot target is selected to make the 16-bit sample alignment explicit and give a conventional 3.072 MHz BCLK at 48 kHz.

## 4. Source files

- `i2s_clock_kernel.rs`: clock-only BIO kernel, DIN low.
- `i2s_clock.rs`: clock-only Xous resource-managed wrapper.
- `i2s_audio_kernel.rs`: BIO I2S data transmitter, 32-bit channel slots.
- `i2s_audio.rs`: bounded 440 Hz PCM generator and BIO FIFO interface.
- `i2s_clock_cmd.rs`: console commands, including `i2sclock tone440`.
- `install_audio.py`: installer for audio API/kernel and updated command.

## 5. Install into local Xous checkout

First update BaoBAB:
```bash
cd ~/Documents/Code/BaoBAB
git pull
```

From the local `xous-core` directory:
```bash
python3 ~/Documents/Code/BaoBAB/component-tests/bio-audio/install_audio.py "$PWD"
git diff --check
git status --short
git diff
```

The installer refuses to overwrite existing audio source files or patch a source tree with unexpected module declarations.

Then compile:
```bash
cargo xtask dabao dabao-console
```

Do not flash until compiler output and the diff have been reviewed. This prototype has not yet been compiled in the user's local checkout.

## 6. First tone run — user hardware result

The user reported this console output after rebuilding and flashing the FIFO/register corrections:

```text
i2sclock status
STOPPED

i2sclock tone440
Played 440 Hz test tone: quantum=6140351 Hz, BCLK=3070175 Hz, LRCLK=47971 Hz, frames=11992, peak_sample=512.
BIO resources released.
```

Interpretation:
- The audio command completed instead of timing out.
- The producer successfully wrote all requested sample words.
- The BIO kernel reached its end-of-buffer path and delivered the completion token.
- Xous reported BIO resource cleanup complete.
- Calculated sample rate was 47,971 Hz; calculated BCLK was 3,070,175 Hz. These are divider-derived values, not physical measurements.
- The run establishes that the BIO/FIFO sample-transfer path completed. It does not alone prove that the MAX98357A produced audible output or that I2S waveforms were measured.

Audible sound check: user reports no sound. Amplifier VIN measured 3.3 V, SD_MODE-to-GND measured 1.5 V, and disconnected speaker DC resistance measured 8.6 ohms. The user's DIN jumper is confirmed on PC8. This points to a need for pin-level/waveform and amplifier-input diagnosis; successful FIFO completion alone is not audible-output proof.
Logic-analyser/oscilloscope validation: pending.

## 7. First playback precautions

Before running `i2sclock tone440`:
1. Verify DIN is connected to PC8 / BIO24, LRC to PC10 / BIO26, BCLK to PC11 / BIO27.
2. Confirm amplifier VIN is 3.3 V relative to amplifier GND and there is no supply short.
3. Speaker must connect only between OUT+ and OUT-. Never ground either output.
4. Keep GAIN open and preserve the breakout's SD_MODE resistor network.
5. Start with the committed bounded 250 ms tone only; do not loop or increase amplitude.
6. Stop if speaker output is unexpectedly loud/distorted, or if the amplifier/board heats, smells, or supply voltage droops.
7. Ideally use a logic analyser/oscilloscope to verify BCLK, LRCLK and DIN; divider readback is not a physical waveform measurement.

## 8. Limitations and known risks

- The original audio transmitter compiled and was flashed, but its first tone command failed with `Timeout(\"BIO TX FIFO did not drain\")`. The next revision adds a correct I2S delay bit, preloads three frames before BIO starts, corrects the sample MSB bit mask, and reports FIFO level on timeout. This revision has not yet been compiled or run.
- BIO FIFO underrun behavior during audio streaming has not been measured. The kernel waits at frame boundaries for sample words; host-side FIFO refill timing may cause clock pauses if the producer cannot keep up.
- The target quantum and actual frequencies must be checked on hardware.
- No separate MCLK is required by MAX98357A.
- The first tone is a bring-up test, not a general-purpose PCM player.
- The exact breakout variant and its onboard GAIN/SD_MODE resistor network should be verified against its vendor schematic if the first test is silent.

## 8. Software pin readback audit

Xous's BIO instruction set defines:
- x21: read GPIO pad input state into x21.
- x22: set output bits selected by x26.
- x23: clear output bits selected by x26 using clear-on-zero semantics.
- x24: configure selected bits as outputs.

The new `i2s_pin_audit_kernel.rs` drives BIO24, BIO26 and BIO27 individually and reports four actual pad-input snapshots through BIO FIFO2:
1. All three outputs low -> expected 0x00000000.
2. DIN high only -> expected 0x01000000.
3. LRCLK high only -> expected 0x04000000.
4. BCLK high only -> expected 0x08000000.

`i2s_pin_audit.rs` reserves one BIO core, FIFO2 and pins 24/26/27 through Xous resource manager, reads all four snapshots with a 500 ms timeout, and releases resources. The console command is `i2sclock pintest`; expected output is four PASS lines.

Install the pin audit after the clock/audio modules:
```bash
python3 /path/to/BaoBAB/component-tests/bio-audio/install_pin_audit.py "$PWD"
git diff --check
git status --short
git diff
```

Then compile with `cargo xtask dabao dabao-console`. Inspect the diff and compiler output before flashing.

**Safety:** run the pin audit with MAX98357A VIN disconnected or its supply otherwise disabled. The audit deliberately generates short static logic-high levels on each audio signal pin; it is not an audio test. It checks BIO-to-pad output and readback, not the wire from the Dabao header to the amplifier input. A logic analyser or direct voltage test at the amplifier pad is needed to verify the external jumper path and signal waveform.

## 9. What this audit can and cannot prove

- PASS verifies the selected BIO bit controls the corresponding Dabao GPIO pad and that the pad input readback sees the expected level.
- FAIL points toward BIO output mask, pad mux/direction, or BIO kernel semantics.
- PASS does not prove that the breadboard jumper reaches the amplifier's DIN/BCLK/LRC pin.
- PASS does not prove 3.072 MHz BCLK timing, 48 kHz LRCLK framing, valid I2S data alignment, SD_MODE threshold margin, sufficient amplifier gain, or audible speaker output.
- After the pin audit, the next software test should drive a repetitive deterministic pattern on one pin at a time and compare it with a logic analyser / multimeter. A multimeter can show static levels but cannot validate MHz clocks.


## 10. User-reported silent-output diagnosis

User reports no audible sound after the command reported successful 440 Hz transmission. Additional measurements:
- MAX98357A VIN-to-GND: 3.3 V.
- SD_MODE-to-GND: 1.5 V (near the upper mode-selection threshold; exact board resistor network/threshold margin should be checked).
- Speaker disconnected resistance: 8.6 ohms, consistent with an 8-ohm nominal speaker.
- User confirms DIN jumper moved to PC8.

A new readback test was committed:
- `i2s_pin_audit_kernel.rs`: toggles the three BIO pins one at a time and uses BIO x21 to read the GPIO pad input state.
- `i2s_pin_audit.rs`: claims BIO core/FIFO2/pins using Xous resource management, reads four snapshots with a timeout and releases resources.
- `i2s_clock_cmd.rs`: exposes `i2sclock pintest`.
- `install_pin_audit.py`: installs the files and updates the command/module registry.

Expected pad readbacks:
| Snapshot | Expected |
|---|---:|
| all-low | 0x00000000 |
| DIN high (PC8/BIO24) | 0x01000000 |
| LRCLK high (PC10/BIO26) | 0x04000000 |
| BCLK high (PC11/BIO27) | 0x08000000 |

The test confirms BIO-to-pad output/readback. It does not test the external breadboard jumper continuity to the amplifier input, high-speed timing, I2S correctness at the IC pin, amplifier gain, or the SD_MODE threshold margin.

## 10. Staged tone amplitude control

The console tone command now accepts only four predefined peak PCM amplitudes:
```text
i2sclock tone440 512
i2sclock tone440 1024
i2sclock tone440 2048
i2sclock tone440 4096
```
Bare `i2sclock tone440` remains the default 512 level. The API rejects zero and values above 4096; the console also rejects values not in the four-step list.

| Step | Peak PCM magnitude | Percent of signed 16-bit positive full scale |
|---|---:|---:|
| 1 | 512 | 1.6% |
| 2 | 1,024 | 3.1% |
| 3 | 2,048 | 6.2% |
| 4 | 4,096 | 12.5% |

Each playback remains a 250 ms 440 Hz tone with 10 ms fades. The 4096 ceiling is a conservative software test limit, not a calibrated loudness or guaranteed speaker power limit. Actual output depends on amplifier gain, supply, breakout design and speaker; stop for harsh distortion, abnormal heating or supply droop. Do not infer that 100% PCM is safe for the 0.5 W speaker.

This change has been committed but not yet installed, compiled or flashed on the user's device.


## 11. External jumper line-hold diagnostic

The on-chip `i2sclock pintest` passed 4/4, proving the BIO signal masks and pad readback. It does not prove that each breadboard jumper reaches the MAX98357A input.

The new optional `i2sclock linehold` test holds one selected signal at a static logic HIGH while holding the other two LOW:
- `i2sclock linehold din` -> PC8 / BIO24 high.
- `i2sclock linehold lrclk` -> PC10 / BIO26 high.
- `i2sclock linehold bclk` -> PC11 / BIO27 high.
- `i2sclock linehold stop` -> stop BIO, unmap pins and release resources.

The BIO test is resource-managed and has no audio clocks. It is intended for a DMM continuity/voltage check at the far end of each jumper. The three module pins are input-only from the amplifier side; a steady high from Dabao is a normal logic level while VIN is powered.

Install after the existing audio and pintest support:
```bash
python3 /path/to/BaoBAB/component-tests/bio-audio/install_line_hold.py "$PWD"
git diff --check
git status --short
git diff
```
Build with `cargo xtask dabao dabao-console`, inspect the diff/compiler output, then flash.

Suggested test procedure:
1. Leave MAX98357A VIN powered at 3.3V and keep common GND connected. Keep the speaker connected as usual; no tone is being played.
2. Run `i2sclock linehold din`. Measure from the DIN signal at the amplifier-side header to amplifier GND. Expect close to 3.3V. Then run `i2sclock linehold stop`.
3. Repeat for `lrclk` and `bclk`, stopping after each test. Expect close to 3.3V at the corresponding amplifier-side input.
4. If an input does not read high, inspect that wire/jumper and its breadboard row; do not adjust firmware audio amplitude.
5. Do not use the DMM to infer a MHz clock; this diagnostic holds a static level only.

A successful far-end static voltage test proves continuity and DC level, not high-speed signal integrity. If all three pass and audio remains silent, the next decisive test is an oscilloscope/logic-analyser capture at the amplifier input pins (BCLK, LRCLK and DIN) during a tone. That validates actual waveform, frequency, timing and data framing.


## 12. External static-level test — user reports all pass

The user ran the `linehold` tests and reports that all three signal wires measured the expected HIGH voltage at the amplifier end:
- DIN / PC8 / BIO24: expected HIGH level observed.
- LRCLK / PC10 / BIO26: expected HIGH level observed.
- BCLK / PC11 / BIO27: expected HIGH level observed.

Together with the earlier BIO pad readback 4/4 PASS, this establishes:
1. BIO can drive the three selected pad outputs.
2. The selected pad-to-header jumper paths pass a static DC HIGH test.
3. The three external wires are not simply open or connected to an entirely different row.

It still does **not** verify dynamic waveforms, clock frequency, duty cycle, I2S framing at the amplifier pin, or the amplifier's analog output. Next decisive evidence is a logic-analyser/oscilloscope capture of BCLK, LRCLK and DIN during the 440 Hz command. A software-only BIO readback can add an internal dynamic self-check, but it cannot replace an external timing measurement.


## 13. Code review after all three far-end static voltages passed

The user reports that the static `linehold` measurements at the MAX98357A end of all three wires are correct. Combined with the BIO pad-readback test, DC pin mapping and continuity checks pass.

A source audit identified a software timing defect in the current `i2s_audio_kernel.rs`:
- The command reports `BCLK = actual_quantum_hz / 2` and `LRCLK = actual_quantum_hz / 128` as though each BCLK period were exactly two BIO quantum waits.
- The serializer does not actually place BCLK edges exactly two quantum waits apart. Between successive BCLK edges, the loop executes data-test instructions, conditional branches, data set/clear instructions, shifts, counter updates and a loop branch. Some paths have different instruction counts for 0 and 1 bits.
- Therefore the reported BCLK/LRCLK values are divider-derived estimates, **not measured or guaranteed output frequencies**. The bit-cell low time is not explicitly bounded to match the high time, and data-dependent branch paths may introduce clock duty-cycle/jitter variation.
- Successful FIFO completion proves that BIO consumed the supplied words and reached the completion path; it does not validate that the resulting clock waveform meets the MAX98357A timing requirements.

This is a credible software-side explanation for silence. Do not increase the tone amplitude further. The next firmware revision should redesign the serial bit-cell timing so every bit has a deterministic, balanced BCLK high/low period, with data prepared outside the timing-critical edge sequence. The host must report only a calculated target until a physical timing measurement is available.

MAX98357A Rev. 16 (February 2026) specifies 10 ns setup and 10 ns hold for DIN and LRCLK relative to BCLK (electrical characteristics). It accepts I2S, with a one-BCLK delay after LRCLK transition; BCLK is sampled on rising edge and LRCLK low indicates left channel. It supports 8–96 kHz, with specifically enumerated LRCLK frequencies including 48 kHz. See:
- https://www.analog.com/media/en/technical-documentation/data-sheets/MAX98357A-MAX98357B.pdf
- Table 5 / Figure 4: SD_MODE channel/shutdown selection and B0/B1/B2 thresholds.
- Figure 8: 32-bit I2S resolution timing.

SD_MODE measured at 1.5 V is **not shutdown** (B0 max 0.355 V), but it is at the B2 maximum comparator threshold (B2 min 1.245 V, typ 1.4 V, max 1.5 V). The selected channel mode is therefore not guaranteed by that measurement alone. Since the current tone duplicates identical samples to both L/R slots, channel selection alone should not explain silence; nevertheless, SD_MODE should be given a clear margin above B2 or set to a deliberate supported selection after checking the exact breakout circuit.

Recommended next validation:
1. Rewrite the BIO serializer to generate a deterministic bit cell with symmetric BCLK high and low intervals and balanced 0/1 paths; keep DIN stable before the BCLK rising edge.
2. Add a short dynamic pad-readback self-test (x21) that checks BCLK and LRCLK transitions and bit counts at the pad; this is an on-chip internal check, not a substitute for scope/logic-analyser capture.
3. Once available, capture BCLK, LRCLK, DIN at the MAX98357A inputs and check 3.072 MHz BCLK (for 48 kHz, 32-bit/channel slots), 48 kHz LRCLK, one-bit I2S delay, 32 BCLKs per channel, and DIN setup/hold.
4. Only after waveform validation, re-test the amplifier at a low fixed amplitude.

