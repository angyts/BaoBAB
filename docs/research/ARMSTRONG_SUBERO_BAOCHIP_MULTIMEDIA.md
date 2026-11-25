# Armstrong Subero's Baochip work — multimedia and pocket-computer research

**Research date:** 2026-11-26  
**Purpose:** Identify public work and reusable techniques relevant to BaoBAB, especially the user's recollection of a Baochip-powered pocket computer with music and games (including Pokémon).

## Executive summary

Armstrong Subero maintains a public bare-metal C SDK specifically for the Baochip-1x on the Dabao evaluation board: https://github.com/ArmstrongSubero/dabao-sdk. Its public README describes a Pico-style single-program C environment, hardware peripheral drivers, DMA-backed UART/SPI/I²C, a four-core BIO programmable-I/O subsystem, and examples. This is a valuable second reference alongside Xous and its Rust HAL.

Armstrong's public Baochip articles also describe substantial work against real silicon, including finding documentation errors and SDK bugs, writing a bare-metal HAL, and training a small transformer on the chip. These establish meaningful Baochip experience, but **the sources inspected for this note do not publicly document the particular pocket computer, Pokémon port, or full music demonstration recalled by the user**. It may be unpublished, shown in a video/social post not indexed by the repositories or blog, or otherwise not publicly available. Absence from the material inspected is not evidence that the demonstration did not happen.

The strongest actionable takeaway for BaoBAB is to investigate BIO, BIO DMA, UDMA SPI and audio timing. The public SDK contains a BIO-generated GPIO waveform and a BIO DMA memory-copy example, but no explicit complete I²S audio-player example was identified in its public example list. BIO is a promising building block, not proof that working I²S playback is already solved.

## 1. Public Baochip work

### 1.1 Dabao SDK

Repository: https://github.com/ArmstrongSubero/dabao-sdk  
License stated in repository: Apache-2.0.  
Platform: Baochip-1x RISC-V SoC on the Dabao evaluation board.  
Language: bare-metal C.

The repository README describes a conventional single-program main() model, with an SDK for GPIO, UART, SPI, I²C, PWM, ADC, timers, BIO, hardware cryptography, and on-chip ReRAM. It describes UDMA use for peripheral transfers, including blocking and non-blocking UART, SPI and I²C APIs.

The README's component/example summary at the time inspected lists 17 drivers and 25 examples; the current RV Embedded projects page advertises 19 drivers and 26 examples. Treat these counts as different snapshots of a changing SDK, not as a single fixed count.

Useful repository entry points:
- SDK overview and build/flash instructions: https://github.com/ArmstrongSubero/dabao-sdk
- Examples directory: https://github.com/ArmstrongSubero/dabao-sdk/tree/main/examples
- BIO driver: https://github.com/ArmstrongSubero/dabao-sdk/tree/main/src/bao1x/hardware_bio
- BIO DMA driver: https://github.com/ArmstrongSubero/dabao-sdk/tree/main/src/bao1x/hardware_bio_dma
- SPI driver: https://github.com/ArmstrongSubero/dabao-sdk/tree/main/src/bao1x/hardware_spi
- SDK README known hardware quirks: https://github.com/ArmstrongSubero/dabao-sdk#known-hardware-quirks

### 1.2 Published articles

**Dabao SDK: A Bare Metal C SDK for the Baochip-1x RISC-V SoC** — 2026-05-08  
https://rvembedded.com/blog_post/8/

Describes the C HAL, target board, driver coverage, examples and motivation for a no-Rust-toolchain route into Baochip development.

**Thirty Thousand People Now Have a Baochip-1x** — 2026-08-10  
https://rvembedded.com/blog_post/10/

Describes hands-on silicon bring-up, the public C HAL, reported documentation corrections and SDK fixes, and a companion Dabao learning/lab book. Useful context for hardware quirks and practical board bring-up.

**Your Baochip Dabao Board Arrived? Quickstart with Bare Metal C** — 2026-08-13  
https://rvembedded.com/blog_post/11/

Practical board bring-up advice: headers, USB data cable, serial console, build/flash process and boot1 REPL behaviour. This is a separate bare-metal programming reference, not the Xous flashing procedure used by this BaoBAB prototype.

**I Trained GPT on a RISC-V MCU with No FPU in 37KB of Memory** — 2026-09-04  
https://rvembedded.com/blog_post/13/

Describes a small decoder-only transformer with backpropagation and Adam training compiled for Baochip-1x. The article describes software floating-point arithmetic on RV32IMAC without an FPU. This demonstrates constrained on-chip computation experiments; it does not imply that large language models or general AI are practical for BaoBAB.

**RISC-V Embedded projects page**  
https://rvembedded.com/projects/

The page describes the Dabao SDK as a hardware-verified SDK with a growing driver and example set. Its advertised 19 drivers / 26 examples differ from older counts in the repository README, so check the live repository for the latest files.

## 2. BIO: why it matters

The Dabao SDK README describes BIO as a quad-core programmable I/O coprocessor, clocked up to 700 MHz in its stated clock tree. It is conceptually similar to RP2040 PIO: small programs run close to pins and can generate or sample precise signal sequences without the main CPU toggling every edge.

Potential uses:
- Generating precise digital waveforms and custom protocols.
- Handling timing-sensitive serial interfaces.
- Offloading repetitive I/O work from the main CPU.
- Coordinating peripheral signals while application code performs other work.

### 2.1 Public BIO example: waveform generation

File: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/bio_blink/bio_blink.c

The example loads a short instruction sequence onto BIO core 0 and toggles PB2 to generate a nominal 100 kHz square wave. Its comments explicitly state that the main CPU remains free to perform other work while BIO runs.

This is a concrete demonstration of BIO-generated output on hardware. It is not an audio or I²S example.

### 2.2 Public BIO DMA example

File: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/bio_dma/bio_dma.c  
Driver: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/src/bao1x/hardware_bio_dma/bio_dma.c

The example uses BIO core 3 as a memory-to-memory copy engine. It compares the transfer against CPU memcpy, exercises blocking and non-blocking calls, verifies copied buffers, and repeats 1 KiB transfers. The driver comments state that core 3 is reserved for the DMA service while it is active.

This matters architecturally: BIO DMA and application BIO programs share the same hardware resource pool. A BaoBAB design using BIO core 3 for DMA must account for the fact that the SDK's DMA implementation reserves it.

### 2.3 BIO limitations and setup details

The public SDK documents that BIO access filters must be configured for BIO cores to access permitted memory/peripherals. Its driver initialisation sequence halts cores, loads/verifies instruction memory, configures clocking and drains FIFOs. This is not a drop-in GPIO API; BIO programs and pin mappings need careful setup and validation.

Do not assume the BIO coprocessor automatically provides an I²S implementation. An I²S transmitter still needs correct BCLK/LRCLK/data timing, word alignment, sample buffering, output-pin mapping, and sustained throughput.

---

## 3. SPI, display and DMA lessons

The SDK includes:
- **examples/spi_async/spi_async.c** — blocking versus non-blocking full-duplex SPI, with loopback verification and CPU work during a background transfer.
- **src/bao1x/hardware_spi/spi.c** — UDMA SPI implementation.
- SPI driver API in **src/bao1x/hardware_spi/include/hardware/spi.h**.

The async SPI example is a useful model for transferring display data without occupying the CPU for the entire transfer. However, it tests SPI loopback, not the Waveshare e-paper HAT+ controller. An actual display driver still needs controller-specific command sequences, busy handling, reset, framebuffer encoding and refresh management.

The SDK README explicitly documents several hardware quirks:
- MDMA (PL230) is reported as not writable from bare metal; use BIO DMA instead.
- SDIO has reported RTL bugs; the SDK recommends SPI mode for SD cards.
- SPI full-duplex command ID is documented as 0xC, not 0x8.
- BIO access filters must be configured appropriately before BIO cores can access SRAM/peripherals.
- The SDK reports perclk as 99 MHz rather than 100 MHz and says the PLL fractional divider is broken (integer-only from the 48 MHz crystal).
- The README marks a protected RRAM security-vector region and warns against writing there.

These are SDK-published warnings. Confirm current source, silicon revision and official errata before relying on any low-level workaround.

---

## 4. Audio and the remembered pocket computer

### 4.1 What is publicly documented

The public Dabao SDK README lists BIO, DMA, SPI, PWM, timers and relevant low-level driver infrastructure. The repository's inspected public example list includes BIO waveform generation and BIO DMA tests, but no explicit complete I²S audio playback example was identified during this research.

Therefore:
- BIO looks like a plausible way to generate timing-sensitive audio signals.
- DMA-style buffering could help keep the CPU from servicing every sample.
- A codec/amplifier such as BaoBAB's MAX98357A still requires a correct I²S-compatible signal source.
- Continuous audio playback requires a working sample clock, stable buffering, handling of underruns, and correct channel/word format.
- These are research hypotheses and proposed experiments, not demonstrated outcomes from the public SDK inspected here.

The MAX98357A used in BaoBAB is an I²S input amplifier; it does not generate audio samples. BaoBAB must provide the sample stream and I²S clocks, either through a validated peripheral or a custom BIO implementation.

### 4.2 User-recalled demonstration

The user recalls Armstrong Subero demonstrating a Baochip-based pocket computer/handheld with music and games, including Pokémon. The user is confident the hardware was Baochip-based and suggested the implementation may not have been published.

Research performed for this note:
- Inspected the public **ArmstrongSubero/dabao-sdk** repository, its README, examples tree and BIO/SPI source examples.
- Inspected relevant public RV Embedded articles and the projects page.
- Did not locate a public source repository, detailed write-up, or example in the inspected material that establishes the specific Pokémon/music handheld.

**Status: reported demonstration, not independently verified from a public source in this research.** Do not attribute unrelated emulators to Armstrong or imply that his ESP32-C3 NES example is Baochip-based. It is a separate platform-specific project: https://rvembedded.com/blog_post/2/. The author's other platform projects should not be conflated with Baochip work.

The best follow-up source would be the original demonstration video, social post, event talk, or a direct pointer from the author. Until then, retain the user's recollection as a lead, not as a confirmed public reference.

---

## 5. Implications for BaoBAB

BaoBAB currently uses Xous and Rust, not Armstrong's single-program bare-metal C SDK. We should **not replace Xous** simply because the C SDK is useful. Treat the SDK as a reference implementation and a source of documented hardware quirks.

Recommended investigations, ordered by value:

1. **BIO feasibility spike:** Build a minimal Xous-side experiment or isolated bare-metal test that produces a known waveform on an unused pin, then measure it with a logic analyser/oscilloscope. Confirm toolchain and pin mapping before attempting audio.
2. **BIO audio/I²S spike:** Generate I²S BCLK, LRCLK and data for a short known PCM tone. Validate bit-clock rate, sample framing, jitter, continuous playback, underrun recovery and stop behaviour before connecting the MAX98357A.
3. **SPI throughput experiment:** Benchmark a representative buffer transfer and determine whether the e-paper HAT+ driver can use asynchronous UDMA SPI efficiently.
4. **Display rendering strategy:** The current e-paper is 250 × 122 monochrome and refresh is slow. Use compact 1-bit framebuffers, partial updates only where supported, and avoid Game Boy-style animation expectations on the current display.
5. **Memory budget:** Measure actual Xous process memory, frame buffers and audio buffers. Do not assume the bare-metal SDK's memory layout or single-program model transfers directly to Xous.
6. **Audio scheduling:** If BIO handles I²S timing, define ownership of BIO cores and DMA. The SDK's BIO DMA design uses core 3, so resource conflicts must be handled explicitly.
7. **Reusable driver/API:** Keep the current Xous/Rust HAL as the production application interface, and document any low-level reference code separately.

### BaoBAB-specific design note

The current BaoBAB prototype has:
- Dabao/Baochip-1x running Xous.
- Waveshare 2.13-inch e-paper HAT+.
- MAX98357A I²S audio amplifier and 8-ohm speaker.
- DRV2605L and 1027 coin vibration motor.
- One five-way joystick and four individual buttons planned.

For audio, a first proof of concept should be a low-amplitude, short-duration tone into the MAX98357A, with the speaker rating protected. Do not attempt high-volume output or unattended looping until the power budget and speaker protection are measured.

---

## 6. References

1. Armstrong Subero, Dabao SDK (source, README, examples, drivers): https://github.com/ArmstrongSubero/dabao-sdk
2. Armstrong Subero, “Dabao SDK: A Bare Metal C SDK for the Baochip-1x RISC-V SoC” (2026-05-08): https://rvembedded.com/blog_post/8/
3. Armstrong Subero, “Thirty Thousand People Now Have a Baochip-1x” (2026-08-10): https://rvembedded.com/blog_post/10/
4. Armstrong Subero, “Your Baochip Dabao Board Arrived? Quickstart with Bare Metal C” (2026-08-13): https://rvembedded.com/blog_post/11/
5. Armstrong Subero, “I Trained GPT on a RISC-V MCU with No FPU in 37KB of Memory” (2026-09-04): https://rvembedded.com/blog_post/13/
6. RV Embedded projects page: https://rvembedded.com/projects/
7. Baochip Coder's Guide: https://baochip.github.io/baochip-1x/
8. BIO loader tool: https://github.com/baochip/bio-loader
9. Armstrong's BIO waveform example: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/bio_blink/bio_blink.c
10. Armstrong's BIO DMA example: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/bio_dma/bio_dma.c
11. Armstrong's SPI asynchronous example: https://github.com/ArmstrongSubero/dabao-sdk/blob/main/examples/spi_async/spi_async.c
12. Separate ESP32-C3 NES emulator article (not Baochip): https://rvembedded.com/blog_post/2/

## 7. Research status

| Question | Status |
|---|---|
| Does Armstrong publish a Baochip/Dabao bare-metal SDK? | Confirmed |
| Does it cover BIO, BIO DMA and SPI? | Confirmed |
| Is there a public BIO waveform example? | Confirmed |
| Is there a public BIO DMA memory-copy example? | Confirmed |
| Did the inspected public repo include a complete I²S audio-player demo? | Not located |
| Is the user's remembered Baochip handheld publicly documented in the inspected sources? | Not located; could be unpublished |
| Can BaoBAB use BIO for I²S audio? | Promising hypothesis; requires hardware experiment |
| Should BaoBAB replace Xous with the C SDK? | No decision; current plan is Xous/Rust with C SDK as reference |

**Do not treat the remembered handheld, I²S audio feasibility, or any untested BIO arrangement as proven capabilities of the current BaoBAB build.**
