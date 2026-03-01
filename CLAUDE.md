# Grey — JAM Blockchain Node Implementation in Rust

Grey is an implementation of the JAM (Join-Accumulate Machine) protocol as specified
in the [Gray Paper v0.7.2](https://github.com/gavofyork/graypaper/releases/download/v0.7.2/graypaper-0.7.2.pdf).

## Project Structure

This is a Rust workspace with crates in `crates/`. The main executable is `grey` and
all libraries are prefixed with `grey-`.

```
crates/
  grey/              # Binary crate — the node executable
  grey-types/        # Core protocol types, constants, and data structures
  grey-codec/        # Serialization/deserialization (Appendix C)
  grey-crypto/       # Cryptographic primitives (Blake2b, Keccak, Ed25519, Bandersnatch, BLS)
  grey-pvm/          # Polkadot Virtual Machine (Appendix A)
  grey-merkle/       # State Merklization & Merkle tries (Appendix D & E)
  grey-erasure/      # Reed-Solomon erasure coding (Appendix H)
  grey-state/        # Chain state representation and transitions (Sections 4-13)
  grey-consensus/    # Safrole block production & GRANDPA finality (Sections 6, 19)
  grey-services/     # Service accounts, accumulation, refinement (Sections 9, 12)
  grey-network/      # P2P networking, block/transaction propagation
```

## Implementation Plan

### Phase 1: Foundation Types & Primitives

1. **`grey-types`** — Core types matching the formal specification:
   - Numeric types: `NB` (u64 balances), `NG` (u64 gas), `NS` (u32 service IDs),
     `NT` (u32 timeslots), `NR` (u64 register values)
   - Protocol constants: `V=1023`, `C=341`, `E=600`, `P=6`, etc. (Appendix I.4.4)
   - Header `H` (eq 5.1): parent hash, state root, extrinsic hash, timeslot,
     epoch/winning-tickets/offenders markers, author index, VRF sig, seal
   - Block `B = (H, E)` (eq 4.2)
   - Extrinsic `E = (ET, ED, EP, EA, EG)` (eq 4.3)
   - State `σ = (α, β, θ, γ, δ, η, ι, κ, λ, ρ, τ, ϕ, χ, ψ, π, ω, ξ)` (eq 4.4)
   - Validator keys `K = B336` with components `kb`, `ke`, `kl`, `km` (eq 6.8-6.12)
   - Work reports `R`, work digests `D`, availability specs `Y` (Section 11)
   - Service accounts `A` (eq 9.3)
   - Tickets `T` (eq 6.6)

2. **`grey-codec`** — JAM serialization codec (Appendix C):
   - Fixed-width little-endian integer encoding `El` (eq C.12)
   - Variable-length sequence encoding with length prefix (eq C.1-C.4)
   - Tuple encoding as concatenation of element encodings
   - Optional/discriminated encoding with `¿` prefix (eq C.5-C.7)
   - Boolean/bitstring encoding (eq C.9)
   - Dictionary encoding as sorted key-value pairs (eq C.10)
   - Block serialization `E(B)` (eq C.16-C.35)
   - Header serialization `E(H)` and unsigned `EU(H)` (eq C.22-C.23)

3. **`grey-crypto`** — Cryptographic primitives (Section 3.8):
   - Blake2b-256 hash `H` (via `blake2` crate)
   - Keccak-256 hash `HK` (via `sha3` crate)
   - Ed25519 signatures (via `ed25519-dalek`)
   - Bandersnatch VRF signatures & Ring VRF proofs (Appendix G)
   - BLS12-381 signatures (Appendix, via `blst` crate)
   - Fisher-Yates shuffle `F` (Appendix F)

### Phase 2: Virtual Machine

4. **`grey-pvm`** — Polkadot Virtual Machine (Appendix A):
   - RISC-V rv64em based ISA with 13 registers (64-bit each)
   - Pageable RAM: 32-bit addressable, 4096-byte pages, R/W/inaccessible
   - Instruction set: ~150 instructions across categories:
     - No-args: `trap`, `fallthrough`
     - Immediate-only: `ecalli`
     - Register+immediate: `jump_ind`, `load_imm`, `load_imm_jump`, branches
     - Two-register: `move_reg`, `sbrk`, bit manipulation, sign extend
     - Two-register+immediate: loads, stores, ALU ops
     - Three-register: ALU ops, branches, conditional moves, loads/stores
   - Gas metering: each instruction costs `ϱΔ` gas
   - Exit reasons: `∎` (halt), `☇` (panic), `∞` (out of gas), `` (page fault), `h̵` (host-call)
   - Standard program initialization `Y(p, a)` (eq A.37-A.43)
   - Argument invocation `ΨM` (eq A.44)
   - Host-call handling `ΨH` with state-mutator function (eq A.36)
   - Four invocation contexts:
     - `ΨI` — Is-Authorized (eq B.1-B.2)
     - `ΨR` — Refine (eq B.3-B.5)
     - `ΨA` — Accumulate (eq B.6-B.20)
     - Guest VM instances for inner PVM (eq B.4)
   - Host-call functions: gas, fetch, lookup, read, write, info, bless, assign,
     designate, checkpoint, new, upgrade, transfer, eject, solicit, forget, etc.

### Phase 3: State & Merklization

5. **`grey-merkle`** — Merklization (Appendices D & E):
   - Binary Patricia Merkle Trie with 64-byte nodes (eq D.3-D.5)
   - Branch nodes: 1-bit discriminator + two 255/256-bit child hashes
   - Leaf nodes: embedded-value or regular (with value hash)
   - State-key constructor `C` (eq D.1)
   - State serialization `T(σ)` → mapping from B31 keys to values (eq D.2)
   - State Merklization `Mσ(σ)` → H (32-byte commitment)
   - Well-balanced binary Merkle tree `MB` (eq E.1)
   - Constant-depth binary Merkle tree `M` (eq E.4)
   - Merkle Mountain Ranges & Belts (eq E.7-E.10)

6. **`grey-erasure`** — Erasure coding (Appendix H):
   - Reed-Solomon in GF(2^16) with rate 342:1023
   - Cantor basis representation for efficient FFT
   - Chunking function `C_k` for variable-size data (eq H.4)
   - Recovery function `R_k` from any 342-of-1023 chunks (eq H.5)
   - Segment encoding/decoding with k=6 for 4104-byte segments

### Phase 4: State Transitions

7. **`grey-state`** — Chain state and transition logic (Sections 4-13):
   - Block-level state transition `Υ(σ, B) = σ'` (eq 4.1)
   - Dependency graph for parallelizable computation (eq 4.5-4.20)
   - Timekeeping: `τ' = HT` (eq 6.1)
   - Recent history `β` tracking (Section 7)
   - Authorization pool and queue management (Section 8)
   - Judgments processing `ψ` (Section 10)
   - Reporting and assurance pipeline (Section 11):
     - Guarantor assignments with rotation (eq 11.18-11.22)
     - Availability assurances processing (eq 11.10-11.17)
     - Work report guarantee validation (eq 11.23-11.42)
   - Accumulation: `∆+`, `∆*`, `∆1` functions (Section 12)
   - Preimage integration (eq 12.35-12.38)
   - Validator activity statistics (Section 13)

### Phase 5: Consensus

8. **`grey-consensus`** — Safrole & GRANDPA (Sections 6, 19):
   - Safrole block production:
     - Epoch/slot management (E=600 slots, P=6 seconds)
     - Seal-key sequence generation (eq 6.24)
     - Ticket accumulation and contest (eq 6.29-6.35)
     - Fallback key sequence (eq 6.26)
     - Outside-in sequencer `Z` (eq 6.25)
     - Key rotation on epoch boundaries (eq 6.13)
     - Entropy accumulation (eq 6.22-6.23)
     - Epoch/winning-tickets markers (eq 6.27-6.28)
   - GRANDPA finality:
     - Best chain selection (eq 19.1-19.4)
     - Finalization with auditing condition
     - Fork resolution preferring ticketed blocks
   - Beefy distribution: BLS signatures on finalized blocks (Section 18)

### Phase 6: Services & Work Processing

9. **`grey-services`** — Service accounts & work pipeline (Sections 9, 12, 14):
   - Service account model: storage `s`, code hash `c`, balance `b`,
     preimage lookups `p`/`l`, gas limits `g`/`m` (eq 9.3)
   - Minimum balance computation (eq 9.8)
   - Privileged services: `χM` (manager), `χA` (assigner), `χV` (designator),
     `χR` (registrar), `χZ` (always-accumulate) (eq 9.9)
   - Work-package structure `P` (eq 14.2)
   - Work-item structure `W` (eq 14.3)
   - In-core computation pipeline (Section 14):
     - Authorization: `ΨI` invocation
     - Refinement: `ΨR` invocation per work-item
     - Segment import/export via DA layer
   - Work-report computation `Ξ(p, c)` (eq 14.12)
   - Auditing protocol (Section 17): tranche-based audit assignments

### Phase 7: Networking & Node

10. **`grey-network`** — P2P networking:
    - Block propagation and import
    - Work-package distribution
    - Erasure-coded chunk distribution for availability
    - Audit announcements and judgments
    - GRANDPA vote propagation
    - Beefy commitment distribution

11. **`grey`** — Node executable:
    - CLI interface with configuration
    - Genesis state initialization
    - Block import pipeline
    - Validator mode (block production, guaranteeing, assuring, auditing)
    - RPC interface for external queries
    - Storage backend for state persistence

## Key Protocol Constants (Appendix I.4.4)

| Constant | Value | Description |
|----------|-------|-------------|
| V | 1,023 | Total validators |
| C | 341 | Total cores |
| E | 600 | Epoch length (timeslots) |
| P | 6 | Slot period (seconds) |
| H | 8 | Recent history size |
| N | 2 | Ticket entries per validator |
| Q | 80 | Authorization queue size |
| O | 8 | Authorization pool size |
| K | 16 | Max tickets per extrinsic |
| I | 16 | Max work items per package |
| GA | 10,000,000 | Accumulate gas limit |
| GI | 50,000,000 | Is-Authorized gas limit |
| GR | 5,000,000,000 | Refine gas limit |
| GT | 3,500,000,000 | Total accumulation gas |

## Development Guidelines

- Specification reference: Gray Paper v0.7.2 (cached at `/tmp/graypaper/`)
- Use `#[cfg(test)]` for unit tests within each crate
- Follow the specification's naming where reasonable, mapping Greek letters to descriptive Rust names
- Use strong typing: distinct newtypes for hashes, keys, indices, etc.
- Prefer `no_std` compatibility where feasible for core crates
- Use `thiserror` for error types, `serde` for auxiliary serialization
