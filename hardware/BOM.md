# Hardware BOM — BaoBAB prototype

**Status updated 2026-11-27.** This inventory records the actual prototype parts and current verification status. It does not imply that combined-power operation or every peripheral has been validated.

| Component | Purpose | Current identification / status |
|---|---|---|
| Baochip-1x Dabao | Main SoC board | Present; use official [Dabao pinout](https://github.com/baochip/dabao/blob/main/docs/pinout/pins.csv) |
| Waveshare 2.13-inch e-Paper HAT V4 | Monochrome display | Builder supplied the [standard 2.13-inch HAT manual](https://www.waveshare.com/wiki/2.13inch_e-Paper_HAT_Manual); do not conflate with HAT+ LSK4 |
| 7.5 mm five-way SMD joystick | Navigation | Present; six terminals (three on each side) reported by builder; COM/contact arrangement still requires continuity map |
| Four 2-terminal tactile push buttons | Back / confirm / menu / cancel | Present; direct GPIO assignment not final |
| MAX98357A breakout | I2S digital amplifier | Present; BIO I2S transfer completed but user reported no audible sound; live waveforms unmeasured |
| Salvaged miniature speaker | Voice output | Disconnected DC resistance measured 8.7 ohms; nominal impedance/power rating not confirmed from marking |
| DRV2605L breakout | I2C haptic controller | Present; previously identified successfully and user reported motor vibration |
| 1027 coin ERM vibration motor | Haptic actuator | Present; voltage/current rating and combined regulator headroom still to verify |
| GY-521 / MPU6050 breakout | Accelerometer and gyroscope | Visually identified from board marking; not yet tested on shared I2C bus |
| RTC | Timekeeping | No RTC appears in the standard HAT V4 manual/schematic supplied. Treat any RTC as a separate module until its actual part number/board is identified |
| 2.54 mm perfboard, headers, insulated wire | Prototype wiring | Present; colour convention documented in [hardware/WIRING_PLAN.md](WIRING_PLAN.md) |
| Multimeter | Electrical checks | Required for continuity, resistance and voltage checks |

## Power and assembly

- The spare Dabao 3.3 V regulator capacity has not been verified. Keep module power branches removable and bring devices up individually.
- Do not connect 5 V to the MAX98357A, DRV2605L or GY-521 in this prototype plan.
- Power off before soldering or changing connections.
- Verify 3V3-to-GND, 5V-to-GND and 5V-to-3V3 isolation before power-on.
- Do not ground either MAX98357A speaker output.
- The nine physical switch actions (five-way joystick plus four buttons) do not yet have a confirmed direct-GPIO allocation. Do not solder those signal connections until the pinmux/console/SD-QSPI audit and joystick continuity mapping are complete.
