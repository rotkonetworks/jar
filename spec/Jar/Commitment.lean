-- DA commitment layer (GF(2^32), Ligerito/ZODA)
import Jar.Commitment.Field
import Jar.Commitment.ReedSolomon
import Jar.Commitment.Merkle
import Jar.Commitment.Transcript
import Jar.Commitment.Utils
import Jar.Commitment.Encode
import Jar.Commitment.Sumcheck
import Jar.Commitment.DA
import Jar.Commitment.Proof
import Jar.Commitment.Prover
import Jar.Commitment.Verifier

-- Circuit infrastructure (general-purpose WI proofs over GF(2^32))
import Jar.Commitment.Circuit
import Jar.Commitment.WitnessEncoding
import Jar.Commitment.WIProof

-- Execution verification layer (Goldilocks, GKR)
import Jar.Commitment.Goldilocks
import Jar.Commitment.GKR
import Jar.Commitment.Bridge

/-!
# Verifiable Computation over DA-Committed Data

Two-field architecture for proving PVM execution to light clients
without re-execution, building on the "accidental computer" observation
that DA tensor encoding IS a polynomial commitment.

## Architecture

```
Work package bytes
  ↓
DA tensor encoding (GF(2^32), Ligerito/ZODA)          ← ~130µs, done at block time
  = polynomial commitment over binary field
  = erasure_root in WorkReport
  ↓
GKR execution proof (Goldilocks, p = 2^64 - 2^32 + 1) ← proves PVM trace
  = layered circuit sumcheck, one round per layer
  reduces to: W(r) = v at input layer
  ↓
Cross-field bridge                                      ← single comparison
  = embed(W_binary(r)) == W_gold(r)
  DA partial evaluation provides W_binary(r) for free
  ↓
Light client verifies WorkResult in O(d · log N) time
```

### Why two fields?

**GF(2^32)** for DA encoding:
- Fast additive FFT (no root-of-unity constraints)
- Efficient tensor coding (Ligerito/ZODA)
- SIMD-friendly XOR operations
- Natural for erasure coding

**Goldilocks** for execution verification:
- char > 2^64: PVM integer arithmetic maps to O(1) field operations
- ADD = 1 gate (not 96 gates for carry chain in GF(2^32))
- MUL = 1 gate (not ~1000 gates for schoolbook binary multiply)
- Compatible with Jolt/SP1 lookup-based instruction verification

### Modules

**DA commitment layer** (GF(2^32)):
1. `Field` — GF(2^32) + GF(2^128) arithmetic
2. `ReedSolomon` — Binary field additive FFT, systematic encoding
3. `Merkle` — Complete binary Merkle tree with batched proofs
4. `Transcript` — Fiat-Shamir transcript
5. `Utils` — Lagrange basis, multilinear folding
6. `Encode` — Column-major matrix → RS columns → Merkle commit
7. `Sumcheck` — Tensorized dot product, polynomial induction
8. `DA` — Tensor ZODA encoding + `intoWitness` bridge
9. `Proof` / `Prover` / `Verifier` — Recursive Ligerito protocol

**Circuit infrastructure** (general-purpose, over GF(2^32)):
10. `Circuit` — Constraint system (AND, XOR, FieldMul, Boolean, Range)
11. `WitnessEncoding` — Witness → multilinear polynomial
12. `WIProof` — Witness-indistinguishable proof bridge

**Execution verification layer** (Goldilocks):
13. `Goldilocks` — Prime field p = 2^64 - 2^32 + 1, quadratic extension
14. `GKR` — Layered circuit sumcheck protocol, PVM instruction encoding
15. `Bridge` — Cross-field verification: Goldilocks ↔ GF(2^32)

### Light client flow

```
Full node                              Light client
─────────                              ────────────
1. Receive work package
2. Execute PVM (refine pipeline)
3. DA-encode bundle (tensor ZODA)     ← encoding IS the commitment
4. Store EncodedBlock                 ← witness for later proofs
5. (On request) Generate GKR proof    ← proves execution over Goldilocks
6. Send proof + bridge claim     ───→ 7. Verify GKR (O(d·log N))
                                      8. Verify bridge (1 comparison)
                                      9. Accept WorkResult ✓
```

### What's implemented vs what's future work

| Component | Status |
|-----------|--------|
| DA tensor encoding (GF(2^32)) | ✓ Implemented, integrated into Grey guarantor |
| Ligerito recursive protocol | ✓ Implemented, 106 Rust tests pass |
| Merkle commitment + proofs | ✓ Implemented |
| Goldilocks field arithmetic | ✓ Specified |
| GKR protocol structure | ✓ Specified (types + interfaces) |
| Cross-field bridge | ✓ Specified |
| PVM → layered circuit encoding | ○ Future (per-opcode circuit generation) |
| Memory consistency (permutation argument) | ○ Future |
| Lookup-based instruction verification (Jolt-style) | ○ Future |

## References

- Ligerito: https://angeris.github.io/papers/ligerito.pdf
- The Accidental Computer: https://angeris.github.io/papers/accidental-computer.pdf
- ZODA: https://eprint.iacr.org/2025/034.pdf
- Jolt: https://eprint.iacr.org/2023/1217
- Thaler, "Proofs, Arguments, and Zero-Knowledge"
-/

namespace Jar.Commitment

-- DA commitment layer (GF(2^32))
export Field (GF32 GF128 gf32Add gf32Mul gf32Inv embedGF32)
export DA (EncodedBlock)
export Proof (LigeritoProof ProverConfig VerifierConfig mkProverConfig mkVerifierConfig)
export Prover (prove proveFromDABlock)
export Verifier (verify)

-- Circuit infrastructure (general-purpose)
export Circuit (Circuit CircuitBuilder WireId Witness Constraint)
export WitnessEncoding (WitnessPolynomial ConstraintPolynomial LigeritoInstance)
export WIProof (WIProof proveCircuit verifyProof proveAndVerify)

-- Execution verification layer (Goldilocks + GKR)
export Goldilocks (GoldilocksElem GoldilocksExt2)
export GKR (LayeredCircuit GKRProof GKRClaim verifyGKR)
export Bridge (CrossFieldClaim verifyCrossField LightClientProof verifyLightClientProof)

end Jar.Commitment
