# BaoBAB specification — v0.1 (draft)

**Date:** 2026-11-05. **Status:** Requirements proposal; not yet implemented.

## Goals and constraints

1. Deliver a reproducible hackathon demonstration on physical Dabao hardware by 2026-12-15.
2. Desktop simulation first; no AI required.
3. Initial signing support restricted to Ethereum Sepolia (chain ID 11155111), disposable keys.
4. USB-C data and power; no battery, wireless or custom PCB for the hackathon.
5. No real funds or production security claims.

## Desktop client

- Python on macOS; RPC URL supplied by user through environment/config, never committed.
- Mainnet fork via Anvil using the user's own mainnet RPC for **simulation only**. Forked chain IDs must not be assumed to differ from mainnet; do not broadcast or sign mainnet payloads in hackathon mode.
- Separate Sepolia RPC for testnet broadcast; a public endpoint is sufficient initially.
- Parse unsigned EIP-1559 transaction envelopes initially; extend only with tests.
- Decode ETH transfers and selected ERC-20 transfer/approve selectors with strict address/ABI checks.
- Anvil execution plus traces/receipts where available; report reverts, gas and observable state changes with block number and limitations.
- Rule-based warnings for unlimited allowance, unknown recipient, amount threshold, unsupported calldata and supplied-label mismatch.
- Resolve ENS on the desktop but label it as unverified unless cryptographic resolution proof is implemented.
- Generate audio clips on Mac and transfer them to device; do not use spoken descriptions as the sole security confirmation.

## Device firmware

- Xous application/service with a narrow USB protocol.
- Verify chain ID 11155111, transaction envelope, recipient, value, fee fields and supported ABI operations from the **actual signing payload**.
- Display full address through pagination or scrolling; do not truncate away the user's only verification path.
- Treat simulator text, ENS labels and voice bytes as untrusted.
- Reject malformed, mismatched and unsupported transactions by default.
- Require a fresh physical confirmation for each signing request; rejection must return no signature.
- Use disposable testnet key only; verify signing algorithm/available accelerators against actual SDK.
- Implement haptic alert with DRV2605L; audio via MAX98357A requires a validated I²S/BIO driver.
- Coordinate motor, display refresh and audio to reduce concurrent load, with audio gain/duration and haptic time caps. **Scheduling is not a substitute for adequate power regulation.**

## USB protocol — to design

Versioned length-delimited messages with bounded sizes, request IDs, explicit network and payload hash, integrity checks and timeouts. Distinguish unsigned transaction, advisory report, audio chunks, approval result and signature. The device must bind displayed fields and approval to the exact signing payload hash. No assumed transport or wire format until a working Xous USB interface is confirmed.

## Acceptance tests

1. Safe Sepolia transfer: correct device display, deliberate approval, signature verified independently.
2. Unlimited ERC-20 approval: deterministic warning, voice/haptic response, no silent signing.
3. Misleading host description: device derives actual fields and rejects inconsistent trusted-field claims.
4. Unknown method or malformed payload: refuse clear-signing and signing.
5. Chain ID 1: reject signing even when the desktop is using a mainnet fork.
6. Cancel, disconnect or timeout: no signature.
7. Record actual current/voltage and verify stable operation of each peripheral before combined testing.
8. Publish reproducible logs, fixtures and test instructions.

## Out of scope

AI, autonomous transaction broadcast, production key storage, general-purpose wallet support, arbitrary DeFi clear-signing, Bitcoin, Lightning, battery, miniaturisation.

## Open questions

- Exact Dabao exposed GPIO pinmux, power regulator headroom and board revision.
- Waveshare HAT+ revision and compatible e-ink/RTC driver.
- Available Xous USB and signing-service interfaces.
- BIO I²S timing and buffer scheduling.
- Safe allocation of RRAM for non-secret application data.
