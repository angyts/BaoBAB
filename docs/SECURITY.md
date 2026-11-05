# Security model — prototype

**BaoBAB is NOT a production hardware wallet. Use only disposable testnet keys and testnet assets.**

## Trust boundaries

- **Untrusted host:** Mac client, RPC responses, ENS names, transaction simulations, contract labels, AI (if added), and streamed speech.
- **Device responsibility:** Independently parse the exact unsigned transaction, enforce Sepolia chain ID and supported-operation allowlist, display verified fields, require physical approval, and sign only the approved payload.
- **Signing material:** Disposable testnet-only secret. Do not rely on undocumented or deprecated Baochip key slots. The official Baochip RRAM documentation describes a deprecated crypto key-slot region.
- **Boot / firmware:** Xous isolation, secure boot, signing services and physical resistance must be audited before any stronger claim.

## Required invariants

1. Never sign chain ID 1 in hackathon mode.
2. Never allow a host-provided explanation or audio clip to override device-derived facts.
3. Never sign an unknown or malformed payload by default.
4. Confirmation must be physically initiated after critical fields are shown.
5. A timeout, reset, crash, cable disconnect or rejection produces no signature.
6. The returned signature must correspond to the exact payload hash shown for approval.
7. No real wallet seed phrases, private keys or mainnet funds in development.

## Limits

Simulation uses a state snapshot and may not predict future contract behaviour, proxy upgrades, mempool ordering or off-chain consequences. ENS resolution on the host is a label, not independent identity proof. A correct MMU isolation test is not proof of production-grade key custody.

## Testing

Include positive and negative transaction fixtures, fuzzing of parsers where practical, chain-ID mismatch tests, misleading-description tests, and explicit hardware measurements. Do not test physical glitch attacks or irreversible boot/security counters on the only development board.

## Reporting

Please avoid publishing real secrets, private RPC URLs or identifying information in issues. Document suspected vulnerabilities privately with maintainers before disclosure.
