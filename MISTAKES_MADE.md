# Mistakes made — BaoBAB engineering log

## DRV2605L bring-up (2026-11-26 Singapore time)

Observed: initial NACK with VIN 0.7 V and SDA/SCL 3.3 V; after correcting supply connection, ACK at 0x5A, VIN 3.3 V, ID 7/status E0, 31 register reads and 10/10 identity tests. One earlier console freeze recovered after reboot. Spare motor 38.5 ohms, installed output pair 38.8 ohms. Motor rating unknown; similar 1027 datasheet is not proof of identity. Pulse commands returned Ok but no vibration. Subsequent dump showed clamp 8C and control3 A0 instead of programmed 40/A8. No proof of cause; standby should retain registers (TI datasheet section 8.4.1.4). Brownout, transport, firmware or other reset remain hypotheses.

Mistakes: treated successful bus transactions as physical success; accepted any Ack count on writes; omitted wake delay, mode/amplitude and post-stop readback; used magic register constants and unverified voltage interpretation; wrote playback before mock tests and full datasheet audit; did not establish regulator headroom or motor ratings; implied software duration was a hard safety bound despite blocking HAL calls. Earlier NACK was not a successful device test. No evidence warrants resoldering yet.

Xous bcfdca404: PB11/PB12 AF1 channel0 400kHz. UDMA write constructs address/register/data/STOP and awaits completion; read resets controller as documented workaround. Busy loops without timeout can hang; no evidence yet that writes are broken. Do not modify upstream HAL speculatively.

Corrective action: disable pulse command, test harmless waveform slot while GO=0, verify immediate/delayed/standby/rewake retention, restore original value, report cleanup independently. New audit not compiled or hardware tested. A blocked HAL can still prevent cleanup. Next: mock transport tests, audit exact register definitions and timing, establish power headroom and motor ratings, then bounded playback with independent physical observation. Do not claim hardware validation until observed.

Reference: https://www.ti.com/lit/ds/symlink/drv2605l.pdf

## General rules

Read primary datasheet and pinned HAL before implementation; separate measured facts from assumptions; test no-load register access before actuation; use named constants, explicit state transitions, checked writes and error reporting; test mock failure paths; compile before claiming readiness; record hardware results and failures; breadboard before permanent soldering. No irreversible security changes or real wallet funds.

## Follow-up: register audit and RTP encoding

Hardware audit passed immediate, delayed, standby and rewake readback and restoration. TI section 8.5.8.1.1 figure 26 exposes a specific prior mistake: unsigned open-loop RTP uses midpoint as neutral, not zero. Old stop wrote zero before standby, potentially commanding negative drive; old 10–30% unsigned commands were not positive low-intensity pulses. New diagnostic uses signed RTP, zero neutral, 0x7F forward, disables startup boost, sets clamp 0x64, enters standby before clearing RTP, and traces configuration retention. Motor and regulator ratings still unverified; trace is experimental, not a hard timed safety bound. New revision not compiled or hardware tested. No OTP writes.

## Speaker bring-up: I2S pin accessibility assumption

We found UDMA_I2S in the Baochip SVD and initially treated its existence as if it could be wired directly to the Dabao expansion header. RTL inspection showed UDMA I2S master signals are AF1 on Port D pins PD10 (SD), PD11 (WS/LRCLK), PD12 (SCK/BCLK), while the official Dabao 40-pin GPIO pinout lists PB and PC header GPIOs, not those PD pins. The exact accessibility of PD pins on test pads/other connectors is not yet known. Do not wire guessed PC9/PC10/PC11 as UDMA I2S. If using accessible PB/PC pads, we must explicitly implement/validate a BIO-generated I2S stream and check resource/timing conflicts. Lesson: a peripheral's register map and presence in RTL do not imply its pads are exposed on the dev-board header; trace SoC AF mux, package pads, board routing and header pinout before proposing wiring.
