# E-paper bring-up postmortem and hardware development policy

Date: 2026-11-28 (Singapore). Status: display not operational; Xous recovered. Based on builder's bench logs and measurements, not a completed electrical qualification.

## Evidence and conclusions

| Experiment | Observed result |
|---|---|
| Xous HAL/UDMA diagnostics | IPC, clock/reset acknowledgement and IFRAM allocation/drop passed; not proof of SPI wire traffic |
| GPIO self-test with HAT | DC, RESET, SCK, CS readback passed; PC1 MOSI HIGH read LOW |
| HAL snapshot in GPIO mode | PC1 OUT=1 IN=0 OE=1 PU=0 AF=0 PIOSEL=0 |
| HAT disconnected | MOSI HIGH readback passed |
| Bare-metal C SDK, same HAT | MOSI commanded HIGH measured 0.9 V; LOW 0.05 V; no visible refresh |
| Bare-metal C SDK, HAT removed | MOSI 3.3 V then 0 V |
| HAT external power and reset | 3.3 V measured; BUSY measured 0 V at both ends with continuity confirmed, but software often reported HIGH |
| BUSY experiments | Pull-up sometimes eliminated fluctuations; eight reset pulse/drive combinations failed to establish reliable controller response |
| Hardware SPI white/black | Software reported success on some runs; no visible refresh. One black operation took 512 ms, suspiciously short |
| Alternative V1.5 driver | No visible refresh reported; panel compatibility and exact assembly not established |
| Panel condition | Black specks reported; cause and reversibility unknown |
| Xous restoration | Successful after restoring OS loader as well as kernel and apps; USB console and custom HAL logs returned |

**Leading hypothesis:** electrical loading, inadequate drive or power delivery, or HAT level translation/power sequencing. Insufficient Dabao supply current is plausible but NOT established. External 3.3 V measured at idle does not establish transient headroom or internal HAT rail voltage. A GPIO HIGH collapsing only with the HAT connected is stronger evidence of a load/interface problem than of a global supply shortage. Two software stacks reproduced it; shared pad configuration assumptions remain possible. No evidence establishes that the HAT or panel is permanently defective.

## Power audit — required before final soldering

Current BOM: Dabao, standard Waveshare 2.13-inch HAT V4, MAX98357A and salvaged speaker (8.7 ohm DC resistance), DRV2605L and 1027 ERM, GY-521/MPU6050, five-way joystick and four switches. RTC identity unresolved. PWM audio alternative also planned; do not assume both audio paths run simultaneously.

| Load | Budget method |
|---|---|
| Dabao | Measure USB input idle and worst workload; establish regulator part, rating, thermal derating and board's own 3.3 V consumption |
| E-paper HAT | Measure idle, reset, full-refresh peak and duration; verify exact PCB schematic and internal rails |
| MAX98357A + speaker | Measure peak at intended volume and supply; speaker DC resistance alone does not determine power |
| DRV2605L + ERM | Measure startup/stall-rated and normal vibration current; use motor datasheet |
| GY-521 | Verify breakout regulator and pull-ups; measure active current |
| Switches/joystick | Calculate pull resistor currents and simultaneous presses |
| RTC / PWM alternative | Identify exact modules and measure before inclusion |

No verified regulator headroom or combined current budget exists. Do not declare all peripherals adequately powered. Compute peak simultaneous load on each rail separately, including controller consumption, conversion losses and margin (initial design target 25% headroom, not a device specification). Check USB source/cable and thermal limits. A multimeter may miss short peaks; use scope/current shunt or suitable current monitor. Test individually, then display+audio+haptics together under realistic worst case. Log rail minimum voltage, peak current, resets and signal levels.

### External power experiment

A separate regulated, current-limited 3.3 V supply may help distinguish inadequate module power from signal loading. This is not induction: it is an independently powered rail. Verify the exact HAT revision and documented 8-pin VCC interface, supply rating and sequencing first. With power off, isolate HAT VCC from Dabao 3V3 and any 5 V feed; connect common ground; never parallel regulators or backfeed USB. Keep signals high impedance until both supplies are valid; avoid driving unpowered devices. Compare rail voltage and MOSI HIGH/LOW with the same firmware, and measure current. If MOSI still collapses with stable external power, investigate translator enable, pad drive and wiring. Do not increase drive strength blindly against a possible contention. Do not apply 5 V to a 3.3 V signal or supply pin.

## Software findings

Xous revision bcfdca40486cc7e0cecfb76fed20cd2b84d5a2e9. GPIO API and HAL inspected. Individual and bank reads share the same GPIOIN register; diagnostic snapshots reused HAL offsets, so not independent validation of silicon. PB BIO mapping was incorrectly reversed in our diagnostic (PB2/3/4 are BIO2/3/4 per documented mapping); full PIOSEL zero makes that decoding error irrelevant to the observed test. HAL individual BIO mapping and bitmask mapping disagree; ClearOnly semantics need independent verification. Masked GPIO writes use (old & ~mask) | value instead of masking value; single-pin callers constrain value, so not a demonstrated cause. IFRAM free path local fix converts byte offset to 4 KiB page index. HAL log overhead lengthened reset pulses; avoid synchronous logging inside timing-critical operations. BUSY LOW alone is not proof of refresh; require plausible command response and visible pattern. Implement bounded waits, error cleanup and panel sleep; never bypass BUSY indefinitely.

## C SDK and Xous integration decision

Use [dabao-sdk](https://github.com/ArmstrongSubero/dabao-sdk) as an independent reference and hardware test harness. Bare metal removes Xous IPC, scheduling and HAL layers; C itself is not inherently more direct than bare-metal Rust. SDK GPIO bit-banging also avoids UDMA. Official Waveshare C protocol can be adapted, but the tested C sequence was not a fully independent vendor port. The test compiled and executed (voltage transitions); display refresh failed.

Preferred ongoing architecture:

    Xous diagnostic application (Rust)
        -> C protocol/test library (FFI, static archive)
        -> Rust callback or C shim for Xous IPC
        -> authorized HAL service
        -> hardware

C compiled into apps does not bypass process isolation. Separate portable protocol code from bare-metal MMIO/startup/interrupt/DMA code. Compile with compatible RV32 ILP32 ABI, freestanding settings and explicit ownership of buffers; link with Rust build.rs or equivalent, expose narrow extern C interfaces, use HAL-backed GPIO/SPI/delay callbacks. No SDK startup, linker script, global interrupt setup or raw physical MMIO in normal Xous apps. If direct register testing is essential, use a narrowly scoped authorized HAL service or separately approved bare-metal test with recovery plan. This is a proposed architecture, not implemented or compiled. Keep signing keys disposable; do not weaken Xous isolation for a display.

## Firmware recovery — verified lesson

The C SDK image is NOT an apps image. Actual UF2 address ranges from builder's files (end exclusive):

| Image | Start | End |
|---|---|---|
| loader.uf2 | 0x60060000 | 0x60075B00 |
| baobab_epaper.uf2 | 0x60060000 | 0x60062900 |
| xous.uf2 | 0x6009FD00 | 0x601BE000 |
| apps.uf2 | 0x602FFD00 | 0x60351000 |

C overwrote the OS loader's first 0x2900 bytes. Restoring kernel and apps alone failed. Restoring loader.uf2 recovered Xous. OS loader is NOT boot0/boot1. Before any future bare-metal flash, back up matching three Xous UF2 images, inspect every block address, confirm recovery entry and preserve boot chain/security settings. SDK flash reported unchanged boot hashes and bootwait already disabled; persistent mode avoided a counter change in this observed run. Do not blindly reuse flags; inspect current script and state. Bootwait changes consume irreversible counter increments. Do not flash boot0/boot1 or alter provisioning. Normal C test had no USB CDC, so lack of USB enumeration did not prove bricking. Boot1 port ended 01; Xous port 03 on this board. Clean worktree build failed after kernel compilation with missing-file error; SVD_PATH suspected but not proven; no need to repeat it for recovery.

## Repeatable peripheral qualification gate

1. Identify exact part/revision, datasheet, pinout, voltage, current, bus and ownership; document uncertainty.
2. Verify unpowered continuity and rail isolation; test power alone.
3. Use C SDK reference or portable C test with known GPIO patterns, stable HIGH/LOW holds, input pulls, timing and readback; verify physical voltage independently.
4. Exercise all relevant capabilities, limits, error cases, timeouts and recovery. For SPI verify actual waveforms/CS; for I2C identify device and register reads; for audio measure clocks/data and audible output; for haptics test motor current and stop; for switches debounce and simultaneous inputs.
5. Bundle diagnostics in one build with interactive USB console, version marker and bounded logging. Prefer apps-only updates; HAL changes require appropriate system image. Validate firmware addresses and rollback before flashing.
6. Port portable C tests into Xous app, repeat on same wiring, compare results; do not call software acknowledgement hardware success.
7. Qualify simultaneous power and pin conflicts; record evidence and only then solder/finalize assembly.

## Mechanical assembly

Builder reports screen cannot be removed from HAT. Do not pry glass, peel bonded panel, force FPC or apply heat. Keep HAT+panel as a single assembly unless exact retention/connector method is confirmed. The spare V1.5 board is not a validated substitute; matching 24-pin FPC does not prove electrical compatibility.

Use open prototype with a supporting front carrier: screen/HAT centered, joystick left, buttons diamond right, Dabao and other modules behind or below. Mount HAT by existing mounting holes/standoffs if available, protect glass and ribbon from pressure, leave connector and reset/prog accessible. Use short removable harnesses, strain relief and separate removable power branches; do not solder directly to glass/FPC. Verify dimensions, mounting hole positions and connector clearance before cutting carrier. If HAT cannot be made operational, procure a complete known-compatible panel+driver assembly rather than forcing separation. Input GPIO allocation remains unresolved; do not finalize soldering.

## References

- [Dabao pinout](https://baochip.github.io/dabao/pinout/)
- [Baochip RTL overview](https://baochip.github.io/baochip-1x/ch00-00-rtl-overview.html)
- [Xous GPIO HAL at tested revision](https://github.com/betrusted-io/xous-core/blob/bcfdca40486cc7e0cecfb76fed20cd2b84d5a2e9/libs/bao1x-hal/src/iox.rs)
- [Waveshare standard HAT manual](https://www.waveshare.com/wiki/2.13inch_e-Paper_HAT_Manual)
- [Waveshare V4 reference driver](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi_JetsonNano/python/lib/waveshare_epd/epd2in13_V4.py)
- [C SDK flashing script](https://github.com/ArmstrongSubero/dabao-sdk/blob/main/tools/serial_flash.py)
- [Prototype BOM](../../hardware/BOM.md)

Next: power audit and non-destructive mechanical inspection; no further display refresh until electrical interface and sleep behavior are verified.
