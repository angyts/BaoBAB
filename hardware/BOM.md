# Hardware BOM — BaoBAB prototype

**Status:** Items discussed/selected or ordered; this file does not imply every item has arrived. Confirm exact vendor SKU and revision before assembly.

| Component | Purpose | Notes |
| --- | --- | --- |
| Baochip-1x Dabao | Main SoC board | Owned; exact revision to verify |
| Waveshare 2.13-inch e-Paper HAT+ with RTC | Monochrome display and clock | Purchased; check exact HAT revision, pinout and supply |
| 7.5 mm five-way SMD joystick | Menu navigation | Requires wiring/footprint validation |
| Two 3 × 4 mm SMD tactile switches | Back / confirm | Selected TS-1088-AR02016 |
| MAX98357A breakout | I²S digital amplifier | 18 × 19 mm listing; BIO output unproven |
| 8-ohm miniature speaker | Voice output | Purchased; verify power rating |
| DRV2605L breakout | I²C haptic controller | Verify logic and supply voltage |
| 1027 coin ERM vibration motor | Haptic actuator | Verify rated/startup current and voltage |
| 2.54 mm perfboard, headers, fine insulated wire | Prototype connections | No custom PCB in V1 |
| Multimeter | Voltage/continuity tests | Do not probe current across a supply rail |

## Power and pins

Do **not** wire from the earlier conceptual GPIO allocation without checking the exact Dabao schematic and module pinouts. The board's spare 3.3 V regulator capacity has not been verified. USB total power and regulator output capacity are different. No simultaneous high-current peripheral operation until individual loads are validated. Audio gain caps and sequential scheduling reduce load but do not protect against a short or an undersized regulator.

## Assembly order

Dabao alone → display → inputs → haptic driver/motor → audio amplifier/speaker → combined load, with power off before changing connections.
