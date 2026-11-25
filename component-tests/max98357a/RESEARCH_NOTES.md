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

## Follow-up: exact Baochip UDMA I2S source inspection

Inspection of public `betrusted-io/xous-core` source (matches the user's checked-out source paths):

### API and event declarations

`libs/bao1x-api/src/udma.rs` defines:
- `PeriphId::I2s = 1 << 13` (UDMA peripheral clock/reset bit).
- `PeriphEventId::I2s = 52`.
- `EventI2sOffset::{Rx=0, Tx=1}`.
- Generic `PeriphEventType::I2s` and event mapping support.

### SVD register map

`utralib/bao1x/bao1x_peri.svd` defines `UDMA_I2S` at base address `0x5010E000`, register block size `0x30`:

| Offset | Register | Purpose from field names |
|---:|---|---|
| 0x00 | REG_RX_SADDR | RX start address |
| 0x04 | REG_RX_SIZE | RX transfer size |
| 0x08 | REG_RX_CFG | RX continuous, datasize, enable, clear |
| 0x10 | REG_TX_SADDR | TX start address |
| 0x14 | REG_TX_SIZE | TX transfer size |
| 0x18 | REG_TX_CFG | TX continuous, datasize, enable, clear |
| 0x20 | REG_I2S_CLKCFG_SETUP | Master/slave/common clock dividers and enables; external/internal selector bits |
| 0x24 | REG_I2S_SLV_SETUP | Slave word count, bits per word, bit order, stereo enable, peripheral enable |
| 0x28 | REG_I2S_MST_SETUP | Master word count, bits per word, bit order, stereo enable, peripheral enable |
| 0x2C | REG_I2S_PDM_SETUP | PDM parameters (not needed for MAX98357A) |

The SVD is a register schema, not a complete programmer's guide. It does not, by itself, establish DMA channel request wiring, exact data packing, FIFO semantics, clock divider formula, safe enable ordering, or physical IOX pin mux assignments.

### HAL status

`libs/bao1x-hal/src/udma/mod.rs` exports `adc`, `i2c`, `spim`, and `uart` modules, but no `i2s` module. Generic `GlobalConfig` supports peripheral clock gating, reset, and event mapping. The dedicated UDMA I2S block is therefore present in the public API/SVD layer but does not have an existing high-level Baochip HAL driver in this checkout.

### Important distinction: legacy audio service

`services/codec/src/backend/tlv320aic3100.rs` uses the separate legacy `utra::audio` peripheral and an I2C-controlled TLV320AIC3100 codec. It documents standard I2S, master clocks and 16-bit sample / 32-bit word / stereo setup at 8 kHz (512 kHz BCLK), and manages a legacy audio FIFO and interrupt handler. This is useful as a conceptual example of streaming PCM and frame buffering, but it is **not** an implementation of UDMA_I2S and it controls a different audio codec. We should not transplant its register accesses into a MAX98357A driver.

### MAX98357A confirmed from manufacturer information

The official MAX98357A datasheet and Analog Devices product page state:
- Supply VDD 2.5–5.5 V.
- Standard I2S, left-justified and TDM inputs; MAX98357A is the I2S version.
- No MCLK pin / no MCLK required.
- Sample rates 8–96 kHz.
- GAIN_SLOT selects gain in I2S/LJ modes; selectable gains 3/6/9/12/15 dB.
- SD_MODE is shutdown and channel-select control; pulling low shuts down.
- Speaker outputs OUTP and OUTN are differential; neither is ground.

Sources:
- https://www.analog.com/media/en/technical-documentation/data-sheets/MAX98357A-MAX98357B.pdf
- https://www.analog.com/en/products/max98357a.html
- https://github.com/betrusted-io/xous-core/blob/master/libs/bao1x-api/src/udma.rs
- https://github.com/betrusted-io/xous-core/blob/master/utralib/bao1x/bao1x_peri.svd
- https://github.com/betrusted-io/xous-core/blob/master/libs/bao1x-hal/src/udma/mod.rs
- https://github.com/betrusted-io/xous-core/blob/master/services/codec/src/backend/tlv320aic3100.rs

## Remaining hard blockers before wiring

1. Trace the UDMA I2S RTL (link in SVD: `rtl/ips/udma/udma_i2s/rtl/udma_i2s_reg_if.sv`) and associated TX engine to learn descriptor/transfer semantics and divider calculations.
2. Determine how UDMA I2S master BCLK, WS/LRCLK, and SDOUT are routed through Dabao IOX. No pinout assignment has been proven. Previously guessed PC9/PC10/PC11 are explicitly unverified and must not be connected or soldered based on the guess.
3. Verify the exact MAX98357A breakout variant and its onboard SD_MODE/GAIN_SLOT strap circuitry from a clear photo/schematic.
4. Confirm the available supply rail and current margin for the class-D amplifier and the 8 ohm / 0.5 W speaker. VDD's supported voltage range does not mean every board rail has enough current.
5. Implement and compile a standalone UDMA_I2S driver or safe minimal test before enabling the amplifier.

## Follow-up: Baochip RTL trace (public `baochip/baochip-1x` main branch)

### UDMA and TX channel observations

- `rtl/modules/ifsub/rtl/udma_sub.sv` instantiates `udma_i2s_top` and connects it to dedicated UDMA TX and RX channels. The TX path has a request/grant/data/valid/ready handshake and a UDMA TX event. The RX I2S source maps to event offset 0; TX maps to offset 1.
- `libs/bao1x-api/src/udma.rs` sets I2S event base 52; therefore the TX event number is 53 (52 + `EventI2sOffset::Tx`).
- `i2s_tx_channel.sv` consumes FIFO words for channel 0 then channel 1 in stereo mode, waits for a word-select edge, shifts serialized sample bits, and requests the next word at word boundaries. This means the TX data stream is sample words, not an opaque PCM byte stream. Exact UDMA/DMA transfer size, channel packing and memory alignment still need a full RTL trace before coding.
- The RTL clock generator captures and uses configured divider values to toggle a generated clock. Exact BCLK and WS division relationship must be traced through `i2s_clkws_gen.sv` and top-level mux configuration; do not assume the divider formula from the SVD field name alone.
- `udma_i2s_top.sv` exposes master data output(s), master clock and word-select signals. `i2s_txrx.sv` has separate channel 0 and channel 1 data outputs. The MAX98357A is mono and can be fed one channel; whether to use channel 0/left slot depends on the exact SD_MODE strap/channel selection.
- The TX path's early FIFO behavior is important: it loads left and right channel words before running; underflow/late data could cause bad output. A safe driver should use continuous DMA, frame buffers, and explicit end-of-stream silence / disable ordering rather than isolated register pokes.

### Pin mapping still unresolved

`libs/bao1x-hal/src/iox.rs` provides generic AF selection and BIO PIO muxing. For PC pins, the BIO mapping is distinct from the UDMA I2S peripheral mapping. Merely seeing `IoxFunction::AF1` or a pin name does not identify which AF1 peripheral function is physically wired to a particular Dabao header. The SoC pad/IOX alternate-function table and Dabao breakout pinout must be traced before choosing physical pins. The earlier PC9/PC10/PC11 suggestion remains unverified.

RTL inspected:
- https://github.com/baochip/baochip-1x/blob/main/rtl/modules/ifsub/rtl/udma_sub.sv
- https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/udma_i2s_top.sv
- https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/i2s_txrx.sv
- https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/i2s_tx_channel.sv
- https://github.com/baochip/baochip-1x/blob/main/rtl/ips/udma/udma_i2s/rtl/i2s_clk_gen.sv
- https://github.com/betrusted-io/xous-core/blob/master/libs/bao1x-hal/src/iox.rs


## Critical pin-mux discovery: dedicated UDMA I2S signals are on Port D

Inspection of `baochip/baochip-1x/rtl/modules/ifsub/rtl/soc_ifsub.sv` shows the I2S master signals are attached to AF1 pads:
- `af1_D[10]` = `i2sm_sd` (I2S master data out)
- `af1_D[11]` = `i2sm_ws` (I2S master word select / LRCLK)
- `af1_D[12]` = `i2sm_sck` (I2S master bit clock / BCLK)

The output channel 1 is unconnected in `ifsub1_intf.sv`; channel 0 is the data output used by the master interface. That is sufficient for a mono MAX98357A if channel selection/slot is matched, but data framing still requires test.

The official Dabao 40-pin pinout page lists the header GPIO pins as PB and PC (alongside power/ground), not PD10/PD11/PD12. Source: https://github.com/baochip/dabao/blob/main/docs/pinout/index.html . Therefore the dedicated UDMA I2S master pins do not appear to be exposed on the standard expansion headers. We have not established whether PD10-12 are available on test pads or another connector. Do not wire the MAX98357A to these pins unless a board schematic confirms physical access.

This changes the implementation decision:
- Preferred UDMA I2S block is present and its master TX pins are fixed to PD10/11/12 AF1.
- Standard Dabao GPIO expansion headers expose PB/PC, so a direct UDMA I2S breakout connection is currently not demonstrated.
- Practical prototype fallback is to evaluate a BIO-based I2S transmitter on three verified, accessible PB/PC pins (e.g. previous PC9/PC10/PC11 are merely candidate free pins, not yet confirmed as mechanically free or non-conflicting). BIO can generate clocks and data in deterministic instruction sequences, but sample streaming, FIFO buffering, clock budget and concurrency must be designed and tested.
- Alternative is to locate PD10/11/12 test pads / board signals from Dabao schematic and see whether jumper wires can reach them. No soldering until verified.

Sources:
- https://github.com/baochip/baochip-1x/blob/main/rtl/modules/ifsub/rtl/soc_ifsub.sv
- https://github.com/baochip/baochip-1x/blob/main/rtl/modules/ifsub/rtl/ifsub1_intf.sv
- https://github.com/baochip/dabao/blob/main/docs/pinout/index.html
