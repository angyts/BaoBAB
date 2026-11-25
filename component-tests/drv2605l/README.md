# DRV2605L — component test 01

Status: breadboard wiring continuity reported good by builder; powered test not yet performed. Experimental only.

## Wiring candidate

| Dabao | DRV2605L |
|---|---|
| PB12 | SDA |
| PB11 | SCL |
| GND | GND |
| 3V3 | VIN |
| — | IN (open, verify breakout default) |

Verify board pinmux, pull-up voltage, regulator capacity and breakout supply before powering. USB disconnected during wiring. No other peripherals attached. Motor must not be activated until its voltage/current rating and driver configuration are verified.

## Test procedure

1. Verify all four end-to-end connections and absence of unintended shorts, including VIN/GND.
2. Verify I2C idle levels and configure PB11/PB12 for open-drain I2C with suitable pull-ups. Start at 100 kHz.
3. Read register 0x00 (STATUS) at seven-bit address 0x5A using a register-address write followed by a repeated-start read of one byte.
4. Print raw status and device ID (bits 7:5); expected DRV2605L ID is 7. Report fault bits separately. Do not write mode, waveform or GO registers.
5. Repeat ten times; report errors and disconnect power before changing wiring.

## Rust test logic

See `probe.rs` for transport-independent test logic. The platform adapter must be implemented against the actual Xous/Baochip SDK; this is not yet a flashable firmware binary.

## Acceptance

- [x] Builder reports continuity checks good.
- [ ] Supply and pinmux verified.
- [ ] I2C read succeeds ten times with expected device ID.
- [ ] No unexpected reset or heating.
- [ ] Record SDK revision, actual voltage, status bytes and errors.

References: https://www.ti.com/lit/ds/symlink/drv2605l.pdf and https://baochip.github.io/dabao/pinout/ .
