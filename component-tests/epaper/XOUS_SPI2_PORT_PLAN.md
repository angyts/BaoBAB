# Xous SPI2 port plan — Waveshare e-Paper HAT+ V4

## Dabao pin map

| Chip pad | SoC alternate function | Dabao header | HAT physical pin | Use |
|---|---|---:|---:|---|
| PC0 | SPIM_CLK_B[2] | 14 | 23 | SCK |
| PC1 | SPIM_SD0_B[2] | 12 | 19 | MOSI / DIN |
| PC2 | SPIM_SD1_B[2] | 11 | — | MISO, unused |
| PC3 | SPIM_CSN0_B[2] | 10 | 24 | CS |
| PB2 | GPIO | 31 | 22 | DC |
| PB3 | GPIO | 32 | 11 | RESET |
| PB4 | GPIO input | 34 | 18 | BUSY |
| PB5 | GPIO output, HIGH enables panel power | 35 | 12 (BCM GPIO18) | PWR enable |

SPI signals are on SPIM channel 2, AF2. GPIO lines remain normal GPIO.

## Xous SPIM details

The current Xous HAL exposes `Spim::new` and `Spim::new_with_ifram` and uses:
- `PeriphId::Spim2` for the channel clock gate.
- `SpimChannel::Channel2`.
- `SpimCs::Cs0`.
- `SpimClkPol::LeadingEdgeRise`.
- `SpimClkPha::CaptureOnLeading` (mode 0).
- MSB-first (default).
- An IFRAM-backed TX buffer and command list.

The UDMA global HAL service exposes clock enable and reset operations. The constructor's `sys_clk_freq` must match the clock actually reaching the SPIM block. Do not guess this value; check the Dabao clock configuration and compare with working Xous display code.

Existing reference implementations:
- `services/graphics-server/src/backend/bao1x.rs` constructs the SPIM engine for the Baochip display backend.
- `libs/bao1x-hal/src/sh1107.rs` demonstrates a higher-level display driver using SPIM and GPIO.
- `libs/bao1x-hal/src/udma/spim.rs` is the source of truth for transfer and DMA semantics.

## Pin setup

The IOX interface should configure:
- PC0, PC1 and PC3 to AF2.
- PB2 and PB3 as outputs.
- PB4 as an input (Schmitt trigger enabled). Pull-up/down should be configured only after checking the panel schematic and idle state.

The SPI controller and GPIO service must own these pads consistently. Do not set PC3 simultaneously as a GPIO CS and SPIM CS.

## Transaction and BUSY safety

- Wait for SPIM TX DMA completion before changing DC or reusing TX buffer.
- Use an explicit finite deadline for every BUSY wait.
- If BUSY never deasserts, report timeout and stop; never hang the REPL.
- Initial SPI rate: 1–2 MHz; increase only once the panel refreshes reliably.
- Keep RTC disconnected for this first display bring-up.
- Do not issue partial refresh commands until the full-refresh sequence is working.

## Important unresolved items

1. The precise Dabao clock passed to the SPIM constructor must be confirmed from the current board clock setup.
2. CS setup/hold and per-byte/per-command CS behaviour must be checked against the Xous SPIM command engine. GPIO CS may be preferable if needed.
3. The Xous GPIO API/service and safe ownership pattern need to be wired into the console command.
4. The 5 V rail at the HAT was not meter-verified by the user; only HAT pin 1 at 3.3 V was confirmed.


## Wiring correction — 2026-11-26

Use the corrected HAT physical header pins: MOSI 19, SCK 23, CS 24, DC 22, RESET 11, BUSY 18. The earlier values 17/19/21/16 were wrong. HAT physical pin 17 is 3.3 V; never connect the Dabao MOSI output to it. Keep the system unpowered while correcting the jumper wires.


## Waveshare panel power enable

The Waveshare upstream Raspberry Pi V4 configuration defines `PWR_PIN = 18` (BCM numbering) and asserts it HIGH in `module_init()`. BCM GPIO18 is physical Raspberry Pi HAT pin 12. The Dabao connection is PB5 / physical header pin 35. The console command must drive this pin HIGH before resetting the controller.
