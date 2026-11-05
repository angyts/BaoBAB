# BaoBAB 🐾

**The hardware wallet that argues with you before you lose your crypto.**

BaoBAB is an experimental, open-source, simulation-first defensive Ethereum signing device for the [Baochip-1x / Dabao](https://baochip.github.io/baochip-1x/) running Xous.

**Status:** Planning / early prototype. No security claims or working hardware integration are implied by this repository.  
**Hackathon target:** December 15, 2026.  
**Initial signing network:** Sepolia only, disposable testnet keys. **Do not use with real funds.**

## The idea

A Mac client prepares and simulates a proposed Ethereum transaction, decodes supported operations, and produces deterministic warnings. A USB-connected BaoBAB independently checks the actual signing payload, shows critical fields on e-ink, vibrates and speaks warnings, and requires physical confirmation before producing a signature.

The personality is playful. The signing boundary is not.

## Architecture

```text
Mac desktop (Python)
  Ethereum RPC / ENS → Anvil simulation → ABI/clear-signing metadata
                    → deterministic risk rules → speech generation
                                    |
                             USB transport
                                    |
Dabao / Xous (Rust)
  independently parse unsigned payload → enforce chain/policy
  → show verified amount/recipient/permissions → physical confirmation
  → sign with disposable Sepolia key → return signature
                                    |
                          Mac optionally broadcasts
```

Simulation reports, ENS names, contract labels and audio generated on the Mac are **untrusted advisory data**. The device must derive its security-critical display from the exact bytes it signs. Unknown calls are not silently presented as safe. Simulation predicts execution at a particular state; it cannot guarantee future outcomes.

## Hackathon MVP

- Mac transaction preparation, Anvil fork simulation and deterministic risk reports.
- Clear review of native ETH transfers and narrowly supported ERC-20 `transfer` / `approve` calls.
- Sepolia-only signing, disposable keys and explicit physical approval.
- E-ink status and risk face, joystick/buttons, haptic warnings and spoken messages.
- One measured Baochip-specific technical demonstration (e.g. Xous process isolation or BIO audio), with reproducible tests.

**Not in V1:** AI analysis, Bitcoin/Lightning, battery, custom PCB, production-grade key custody, arbitrary contract clear-signing.

## Hardware

Dabao board; Waveshare 2.13-inch e-Paper HAT+ with RTC; five-way joystick; two tactile buttons; MAX98357A I²S amplifier and 8-ohm speaker; DRV2605L haptic driver and 1027 coin vibration motor; perfboard, headers and wiring. All interfaces, power rails and pin mappings must be validated against board/module schematics **before wiring**.

## Software candidates

- Xous + Baochip firmware tooling (device)
- Python + web3.py (Mac)
- Foundry Anvil (forked EVM simulation)
- ABI decoding and ERC-7730 metadata where applicable
- macOS text-to-speech / prerecorded voice, transferred as audio over USB

We will evaluate existing open-source signer code before implementing signing primitives. No AI API is required for V1.

## Development

See [Specification](docs/SPECIFICATION.md), [Security model](docs/SECURITY.md), [Roadmap](docs/ROADMAP.md) and [Hardware BOM](hardware/BOM.md).

## Safety and licensing

Experimental research prototype only. No real private keys, real funds or production wallet use. A project licence will be chosen after dependency and redistribution review; until then, contributions and reuse are subject to applicable copyright law.

**BaoBAB — Because “Are you sure?” isn't enough.**
