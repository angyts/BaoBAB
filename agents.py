"""BaoBAB engineering instructions for AI coding agents.

Read this before proposing hardware, firmware, wiring or build changes.
These rules derive from the November 2026 Dabao bring-up failures.
"""

GUIDELINES = """
1. MANUFACTURER FIRST
Read the exact board schematic, pinout, datasheet, errata and manufacturer
manual before writing code or proposing soldering. Identify the exact revision;
HAT V4 and HAT+ are not interchangeable. Prefer original open-source vendor
drivers, then known working community implementations. Preserve copyright,
record upstream revision and hashes, and adapt only platform-dependent code.
Do not reinvent a working protocol or silently combine different SDK backends.

2. VERIFY EVERY HARDWARE ASSUMPTION
Trace register definitions to generated SVD/header and taped-out RTL. Never
invent addresses, offsets, alternate functions, bit positions or voltage limits.
Distinguish physical header numbering, GPIO numbering and peripheral mux.
Check pin ownership, boot straps, voltage domains, pad drive, pulls and BIO.
Two reads of the same register are not independent evidence. IPC acknowledgement
is not proof of a physical transaction. Verify actual signals with instruments.
Record observations separately from hypotheses and never claim hardware success
without visible or independently measured evidence.

3. POWER BEFORE PERMANENT ASSEMBLY
Identify regulator ratings and available current after SoC consumption. Budget
peak simultaneous loads, audio, motor startup, display refresh and thermal
headroom. Idle 3.3 V does not prove adequate transient supply. Never parallel
regulators or backfeed USB. Use common ground, verified supply sequencing and
current-limited external supplies where appropriate. Do not increase GPIO drive
against suspected contention. Power off before rewiring. Keep harnesses removable.
Do not pry bonded e-paper glass or force short flex cables. Verify FPC pitch,
contact orientation and electrical pinout before using an alternative driver.

4. LESSONS FROM THE V4 HAT
PC1 MOSI measured 0.9 V HIGH and 0.05 V LOW with HAT, but 3.3 V and 0 V
without HAT, using identical bare-metal C firmware. Rust also failed with HAT.
This suggests electrical loading or shared pad/power assumptions, not proof of
insufficient regulator current or a defective HAT. BUSY voltage and software
readback disagreed. Do not bypass BUSY or claim a refresh merely because it is LOW.
Vendor reset is 20/2/20 ms; polling interval and post-release delay are 10 ms.
Avoid synchronous logging inside timing-critical operations. The generic vendor
PWR GPIO is not proof that a particular HAT connects it. Verify schematic first.
Use bounded waits, abort subsequent transfers on failure, and safe shutdown.

5. C AND RUST
C SDK is an independent reference because bare metal removes Xous IPC and
scheduling, not because C is intrinsically superior. Prefer portable vendor C
protocol code linked into a Rust Xous app via a compatible RV32 ILP32 ABI.
Use authorized HAL callbacks for GPIO/SPI/timing; no raw physical MMIO, startup
code, linker scripts or interrupt ownership in ordinary apps. Preserve process
isolation and signing boundaries. Test host mocks and target compilation.
Do not describe a mock test as a hardware test.

6. FIRMWARE SAFETY
Inspect actual UF2 block addresses before flashing. The C SDK image at
0x60060000 overwrote the Xous OS loader. Restoring only xous.uf2 and apps.uf2
failed; restoring loader.uf2 recovered the board. OS loader is not boot0/boot1.
Back up matching loader, kernel and apps images; verify recovery entry and
signatures. Do not alter boot0/boot1, provisioning, fuses or bootwait without
explicit approval. Bootwait changes consume irreversible counter increments.
Prefer apps-only deployment when no HAL changes are needed. Never assume
successful cp means successful installation. Bare-metal firmware without USB
CDC may appear dead. Do not recommend a clean rebuild when a verified backup
will suffice. Preserve local modifications and never reset or clean them away.

7. QUALIFICATION AND TIME
For each new peripheral, read manuals, reuse vendor tests, verify power and
signals, exercise every supported mode and error path, then integrate with Xous
and test combined load. Bundle diagnostics in one build: version, pin holds,
input sampling, bounded timeouts, command traces, full capability test and safe
cleanup. Avoid repeated reflashes for tiny changes. Use actual compiler results
and report what was not tested. Never claim a commit/build/flash succeeded
without evidence. If blocked, say exactly what is missing and provide the
shortest safe next action. Prefer prebuilt tools over lengthy source installs.
Do not ask the developer to repeat measurements already supplied.

8. SECURITY AND CHANGE CONTROL
BaoBAB is a testnet-only experimental wallet. Never use real funds or keys.
Do not weaken isolation to make peripheral programming easier. Review changes,
keep experiments separate, preserve rollback, and commit only authorized work.
Document failed experiments and corrections, not only successes. Do not turn
uncertain wiring into a confirmed plan. A deadline is not a reason to fabricate
confidence. Reuse first, implement last.
"""

if __name__ == "__main__":
    print(GUIDELINES.strip())
