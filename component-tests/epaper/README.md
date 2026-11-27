# Waveshare 2.13-inch e-Paper HAT V4 — Dabao component test

**Hardware identity:** the builder supplied the [Waveshare 2.13-inch e-Paper HAT Manual](https://www.waveshare.com/wiki/2.13inch_e-Paper_HAT_Manual) and identified the panel as V4. Use this standard HAT V4 documentation, not the separate 2.13-inch HAT+ LSK4 schematic/product.

## Canonical wiring

The complete, reviewed prototype wiring plan is maintained in [hardware/WIRING_PLAN.md](../../hardware/WIRING_PLAN.md).

The V4 manual's Raspberry Pi 40-pin mapping is:

| HAT physical pin | Signal | Dabao physical pin | Dabao signal |
|---:|---|---:|---|
| 1 | 3V3 | 36 | 3V3 |
| 2 | RPI_5V (40-pin P1 rail) | 40 | VBUS / 5V |
| 6 | GND | 3 | GND |
| 19 | DIN / MOSI | 12 | PC1 / SPIM2 MOSI |
| 23 | CLK / SCK | 14 | PC0 / SPIM2 CLK |
| 24 | CS / CE0 | 10 | PC3 / SPIM2 CS0 |
| 22 | DC | 31 | PB2 |
| 11 | RST | 32 | PB3 |
| 18 | BUSY | 34 | PB4 |

The HAT schematic shows the 40-pin P1 power rails. The manual's separate 8-pin display cable is a different connector and must not be confused with P1.

**Important correction:** the standard V4 HAT manual/interface does not specify a separate PWR wire. HAT physical pin 12 is a Raspberry Pi GPIO position, not an e-paper PWR input. The PB5-to-HAT-pin-12 wiring and PB5 PWR setup have been removed from the project plan/source. The standard HAT schematic supplied with this manual also does not contain a DS3231 RTC; an RTC, if physically present in the user's assembly, must be identified as a separate module.

## Firmware commands

- `epaper info`
- `epaper debug on|off|status`
- `epaper diag`
- `epaper reset`
- `epaper init`
- `epaper white`
- `epaper black`

The debug toggle only emits e-paper-scoped messages; it does not change the process-wide log filter.

## Known hardware status

HAL/UDMA diagnostics passed, including SPIM2 clock/reset IPC and IFRAM allocation/drop. The display still timed out waiting for BUSY after reset in the latest supplied bench log. Do not interpret the infrastructure diagnostic as proof that the display controller is communicating.

The RESET jumper was found open during continuity testing and was repaired. The panel still needs successful reset/BUSY and SPI initialization tests.

## Safety

Before power-on, check the 3V3/5V/GND rails and signal continuity, and inspect for solder bridges. Keep the 3V3 and 5V rails separate. Do not connect MAX98357A OUT+ or OUT- to GND. Verify the HAT is the V4 board matching the linked manual before using this plan.
