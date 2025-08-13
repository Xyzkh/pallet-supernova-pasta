# SuperNova Verifier — Pasta (SuperNova v1)

![CI](https://github.com/Xyzkh/pallet-supernova-pasta/actions/workflows/ci.yml/badge.svg)

A verifier for SuperNova proofs on the **Pasta** curve (Pallas/Vesta).

This repo contains:
- a **verifier CLI** (`src/`) to verify proofs  
- a **generator** (`supernova-gen/`) to create `vk.json`, `proof.json`, and `inputs.json`  
- **frozen fixtures** (`fixtures/pasta-fib-n10/`) with hashes & metadata  
- **CI** (GitHub Actions) that builds → lints → tests → generates a real proof → verifies it

---

## Structure
```plaintext
.
├─ Cargo.toml
├─ src/                     # verifier CLI (prints "Verification result: true/false")
├─ supernova-gen/           # generator for vk/proof/inputs (release recommended)
├─ fixtures/
│  └─ pasta-fib-n10/        # real example (num_steps=10) + METADATA.md + hashes
└─ .github/workflows/ci.yml # CI pipeline
