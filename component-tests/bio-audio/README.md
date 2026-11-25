# BIO-powered I2S audio experiment (Xous / Dabao)

**Status:** clock-only BIO test passed on hardware. A bounded 440 Hz PCM tone implementation is committed and flashed once, but its first run timed out waiting for the BIO TX FIFO to drain. The timeout means the BIO consumer did not drain the producer queue quickly enough; root cause was not conclusively isolated. Fixes below are committed but have not yet been rebuilt or run.

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
