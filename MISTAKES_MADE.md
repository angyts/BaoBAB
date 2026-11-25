# Mistakes made — BaoBAB engineering log

## DRV2605L bring-up (2026-11-26 Singapore time)

Observed: initial NACK with VIN 0.7 V and SDA/SCL 3.3 V; after correcting supply connection, ACK at 0x5A, VIN 3.3 V, ID 7/status E0, 31 register reads and 10/10 identity tests. One earlier console freeze recovered after reboot. Spare motor 38.5 ohms, installed output pair 38.8 ohms. Motor rating unknown; similar 1027 datasheet is not proof of identity. Pulse commands returned Ok but no vibration. Subsequent dump showed clamp 8C and control3 A0 instead of programmed 40/A8. No proof of cause; standby should retain registers (TI datasheet section 8.4.1.4). Brownout, transport, firmware or other reset remain hypotheses.

Mistakes: treated successful bus transactions as physical success; accepted any Ack count on writes; omitted wake delay, mode/amplitude and post-stop readback; used magic register constants and unverified voltage interpretation; wrote playback before mock tests and full datasheet audit; did not establish regulator headroom or motor ratings; implied software duration was a hard safety bound despite blocking HAL calls. Earlier NACK was not a successful device test. No evidence warrants resoldering yet.

Xous bcfdca404: PB11/PB12 AF1 channel0 400kHz. UDMA write constructs address/register/data/STOP and awaits completion; read resets controller as documented workaround. Busy loops without timeout can hang; no evidence yet that writes are broken. Do not modify upstream HAL speculatively.

Corrective action: disable pulse command, test harmless waveform slot while GO=0, verify immediate/delayed/standby/rewake retention, restore original value, report cleanup independently. New audit not compiled or hardware tested. A blocked HAL can still prevent cleanup. Next: mock transport tests, audit exact register definitions and timing, establish power headroom and motor ratings, then bounded playback with independent physical observation. Do not claim hardware validation until observed.

Reference: https://www.ti.com/lit/ds/symlink/drv2605l.pdf

## General rules

Read primary datasheet and pinned HAL before implementation; separate measured facts from assumptions; test no-load register access before actuation; use named constants, explicit state transitions, checked writes and error reporting; test mock failure paths; compile before claiming readiness; record hardware results and failures; breadboard before permanent soldering. No irreversible security changes or real wallet funds.
