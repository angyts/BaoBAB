# Roadmap — Baochip hackathon 2026

Target submission: **December 15, 2026** (repository, demonstration video, write-up).

## Nov 5–10: Foundation
- [ ] Verify Dabao toolchain, Xous build and USB console.
- [ ] Start Anvil with mainnet fork from user-owned RPC; ensure no broadcast.
- [ ] Generate transaction fixtures and first deterministic risk report.

## Nov 11–18: Desktop simulation
- [ ] ETH transfer and ERC-20 approve/transfer parsing.
- [ ] Detect max allowance, wrong chain, unsupported calls and misleading descriptions.
- [ ] Record block context and simulation limitations.

## Nov 19–29: Device signing boundary
- [ ] Define versioned USB protocol.
- [ ] Implement strict Sepolia-only payload verification.
- [ ] Demonstrate approval/rejection and independently checked disposable-key signature.
- [ ] Evaluate reuse of existing open-source Xous Ethereum signer work.

## Nov 30–Dec 7: Physical personality
- [ ] E-ink + joystick/buttons.
- [ ] Haptic driver + 1027 motor.
- [ ] BIO I²S audio proof of concept + MAX98357A + speaker.
- [ ] Power-rail validation and conservative playback cap.

## Dec 8–12: Silicon demonstration and hardening
- [ ] Reproducible Xous/MMU or BIO experiment with measured results.
- [ ] Negative security tests, failure-mode handling and integration demo.

## Dec 13–15: Submit
- [ ] Clean build instructions, dependency licences and attribution.
- [ ] Video and write-up showing live board behaviour.
- [ ] Publish actual implemented-versus-planned status.

## After hackathon
Custom PCB, battery, enclosure, Bitcoin/Lightning and optional AI.
