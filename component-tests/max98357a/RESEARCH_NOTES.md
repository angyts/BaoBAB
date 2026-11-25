# MAX98357A and Baochip I2S research notes

## MAX98357A primary source

Official Analog Devices/Maxim datasheet: https://www.analog.com/media/en/technical-documentation/data-sheets/MAX98357A-MAX98357B.pdf

Product page: https://www.analog.com/en/products/max98357a.html

Datasheet/product information establishes:
- Single supply range: 2.5 V to 5.5 V.
- Accepts standard I2S, left-justified, and 8-channel TDM.
- No MCLK input required.
- Sample rates 8 kHz to 96 kHz.
- Five gain choices: 3, 6, 9, 12, and 15 dB.
- Input pins are DIN, BCLK, LRCLK; supply is VDD; GND common.
- SD_MODE provides shutdown and channel selection.
- GAIN_SLOT selects gain in I2S/LJ modes.
- Speaker output is differential OUTP/OUTN. Neither speaker output should be grounded.
- Amp output power claims in datasheet depend on supply voltage, load, THD and thermal conditions. Do not infer output level from supply alone.

The breakout board schematic and silkscreen still need identification. Pinout and any onboard SD/GAIN resistors vary across breakout vendors. Do not assume raw IC pin behavior maps directly to board labels.

Useful official refs:
- Product overview and electrical range: https://www.analog.com/en/products/max98357a.html
- Datasheet PDF: https://www.analog.com/media/en/technical-documentation/data-sheets/MAX98357A-MAX98357B.pdf
- Adafruit breakout pinout explainer (third-party): https://learn.adafruit.com/adafruit-max98357-i2s-class-d-class-d-mono-amp/pinouts

## Baochip / Xous source observations (from user's local checkout)

- `git grep -n -i 'i2s' -- libs/bio-lib` returned no matches.
- `git grep -n -i 'i2s' -- apps-dabao libs/bao1x-hal` returned no matches.
- `libs/bio-lib/src/lib.rs` documents BIO FIFOs x16-x19, GPIO registers x21-x26, event registers x27-x30, core debug x31.
- `libs/bio-lib/src/ws2812.rs` shows BIO resource claim/init/release patterns, but this is a timing-sensitive GPIO protocol, not an audio peripheral driver.
- `git grep -n -i 'i2s'` across the whole user checkout returned:
  - `libs/bao1x-api/src/udma.rs`: I2S peripheral/event enum definitions.
  - `utralib/bao1x/bao1x_peri.svd`: UDMA_I2S register definitions.
  - `services/codec/src/backend/tlv320aic3100.rs`: I2S codec backend methods and hardware setup for Betrusted audio.
  - `services/codec/src/main.rs`: invokes codec audio start/stop.
- Presence of registers and event IDs does not prove a usable transmit driver exists or establish Dabao pin multiplexing, DMA setup, clocking or FIFO data semantics.
- User's earlier proposed PC9/PC10/PC11 mapping is unverified. Do not wire to these without tracing IOX mux / pin map / I2S RTL.
- A proposed 16 kHz / 16-bit / stereo-slot stream would normally require BCLK=512 kHz for 16 bits x 2 channels x 16 kframes/s, but actual I2S master configuration must be derived from hardware and amplifier datasheet; this target is not yet confirmed.

## Open questions

1. Which UDMA I2S direction and master/slave modes are implemented and accessible?
2. What is the correct I2S TX FIFO/data port and DMA channel/event sequence?
3. Which Dabao pins can mux to I2S BCLK, WS/LRCLK, and SDOUT?
4. Does the current HAL service expose UDMA I2S or is a new Rust wrapper required?
5. What board breakout variant is the user holding, and what are its onboard pullups/gain/channel straps?
6. Which rail can power the amp while driving the 8-ohm, 0.5 W speaker? Need measured regulator headroom and supply plan.
7. Need speaker polarity/output differential wiring verified. Never connect OUTN/OUTP to ground.

## Mistakes to avoid

- Do not guess GPIO pins from a previous project or use generic Pico pin mappings.
- Do not equate an SVD register map with a driver.
- Do not use the haptic process's overly optimistic "ACK == works" criterion for audio; validate clock waveforms and audible output.
- Do not start at maximum gain or maximum supply voltage.
- Do not leave an enabled Class-D amplifier connected to an unknown/shorted load.
- Do not solder until pin mapping, ground, VDD, SD_MODE, GAIN_SLOT and speaker output have been checked.
