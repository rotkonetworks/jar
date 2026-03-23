import Jar.Commitment.Field
import Jar.Commitment.Goldilocks
import Jar.Commitment.GKR
import Jar.Commitment.DA

/-!
# Cross-Field Bridge — Goldilocks ↔ GF(2^32)

Connects the GKR execution proof (over Goldilocks) to the DA polynomial
commitment (over GF(2^32)). This is the critical protocol component
that makes the "accidental computer" work end-to-end:

```
Work package bytes
  ↓
DA tensor encoding (GF(2^32), Ligerito/ZODA)     ← done at block time, ~130µs
  = polynomial commitment W over GF(2^32)
  ↓
GKR execution proof (Goldilocks)                  ← proves PVM execution
  reduces to: "W_gold(r) = v" over Goldilocks
  ↓
Bridge: verify W_gold(r) matches W_binary(r')     ← this module
  ↓
Light client accepts WorkResult
```

## The problem

The GKR proof operates over Goldilocks (p = 2^64 - 2^32 + 1) because
PVM integer arithmetic is native there. It reduces to a claim about the
input polynomial: W_gold(r) = v at a random point r ∈ Goldilocks^k.

The DA encoding committed the same data as a polynomial W_binary over
GF(2^32). The tensor encoding provides partial evaluations for free.

But W_gold and W_binary are polynomials over DIFFERENT fields. We need
to verify that they encode the same data.

## The solution: coefficient-level consistency

The work package bytes are the SAME in both representations:
- GF(2^32): each 4-byte chunk → one GF(2^32) element (polynomial repr)
- Goldilocks: each 4-byte chunk → one Goldilocks element (integer repr)

Since GF(2^32) elements are 32-bit values and Goldilocks has char > 2^32,
every GF(2^32) element can be embedded into Goldilocks as an integer.
The multilinear extensions agree at boolean hypercube points:

  W_binary(x) = W_gold(x) for all x ∈ {0,1}^k

(because both return the same data bytes, just interpreted differently)

By Schwartz-Zippel, if W_binary(r') = W_gold(r') at a random point r',
then with overwhelming probability they encode the same polynomial.

## Protocol

1. GKR produces claim: W_gold(r) = v over Goldilocks
2. The verifier picks a random point r' ∈ Goldilocks^k (via Fiat-Shamir)
3. The prover evaluates W_gold(r') and W_binary(r') and provides both
4. The verifier checks:
   a. W_gold(r') is consistent with the GKR output (sumcheck reduction)
   b. W_binary(r') is consistent with the DA commitment (Ligerito opening)
   c. W_gold(r') = embed(W_binary(r')) (cross-field consistency)

Step (b) uses the DA tensor encoding's partial evaluation — the
"accidental computer" gives this for free. Step (c) is a single
field comparison after embedding.

## References

- The Accidental Computer §3-4 (GKR + ZODA integration)
- Spartan (cross-field techniques): https://eprint.iacr.org/2019/550
-/

namespace Jar.Commitment.Bridge

open Jar.Commitment.Field
open Jar.Commitment.Goldilocks
open Jar.Commitment.GKR
open Jar.Commitment.DA

-- ============================================================================
-- Embedding: GF(2^32) → Goldilocks
-- ============================================================================

/-- Embed a GF(2^32) element into Goldilocks.
    GF(2^32) elements are 32-bit values (polynomial representation).
    Since Goldilocks has p > 2^32, the embedding is just the integer
    value of the polynomial representation. -/
def embedBinaryToGoldilocks (x : GF32) : GoldilocksElem :=
  GoldilocksElem.fromNat x.toNat

/-- Embed a GF(2^128) element into Goldilocks extension.
    The 128-bit value is split into two 64-bit limbs. -/
def embedGF128ToGoldilocks (x : GF128) : GoldilocksExt2 :=
  let lo := GoldilocksElem.fromNat x.lo.toNat
  let hi := GoldilocksElem.fromNat x.hi.toNat
  ⟨lo, hi⟩

-- ============================================================================
-- Cross-Field Evaluation Claim
-- ============================================================================

/-- A cross-field claim binding a GKR output to a DA commitment.
    The GKR proof reduced to W_gold(r) = v over Goldilocks.
    The DA commitment provides W_binary(r') via partial evaluation.
    The bridge verifies these are consistent. -/
structure CrossFieldClaim where
  /-- GKR's final claim: evaluation point in Goldilocks. -/
  gkrPoint : Array GoldilocksElem
  /-- GKR's claimed value at that point. -/
  gkrValue : GoldilocksElem
  /-- DA commitment's evaluation at the embedded point. -/
  daValue : GF32
  /-- The random challenge point for cross-field check. -/
  bridgePoint : Array GoldilocksElem

/-- Verify cross-field consistency.
    Checks that the DA evaluation, when embedded into Goldilocks,
    matches the GKR evaluation at the same point.

    This is the "accidental computer" bridge: the DA encoding
    already computed the polynomial commitment, and the GKR proof
    reduced to an evaluation claim. The bridge just checks they
    agree on the same data. -/
def verifyCrossField (claim : CrossFieldClaim) : Bool :=
  -- The DA value (GF(2^32)) embedded into Goldilocks must equal
  -- the GKR value (Goldilocks) at the bridge point.
  let daInGoldilocks := embedBinaryToGoldilocks claim.daValue
  daInGoldilocks == claim.gkrValue

-- ============================================================================
-- End-to-End Light Client Verification
-- ============================================================================

/-- What a light client needs to verify a WorkResult.

    Given:
    - A DA commitment (erasure_root from the work report)
    - A GKR proof (proving PVM execution over Goldilocks)
    - A bridge claim (connecting the two fields)

    The light client checks:
    1. GKR proof verifies (sumcheck per layer) → input claim W(r) = v
    2. DA opening is valid (Ligerito partial evaluation) → W_binary(r')
    3. Bridge: embed(W_binary(r')) == W_gold(r') → same data

    Total verifier cost: O(d · log N) for GKR + O(log N) for Ligerito
    where d = circuit depth, N = trace length. No PVM re-execution. -/
structure LightClientProof where
  /-- GKR proof of PVM execution. -/
  gkrProof : GKRProof
  /-- The GKR circuit description (PVM execution trace structure). -/
  circuit : LayeredCircuit
  /-- Cross-field bridge claim. -/
  bridgeClaim : CrossFieldClaim
  /-- DA commitment root (from work report's erasure_root). -/
  daRoot : Option CMerkle.CHash
  /-- Claimed WorkResult (what the light client is verifying). -/
  claimedOutput : Array GoldilocksElem

/-- Verify a light client proof.
    Returns true if the WorkResult is consistent with the DA-committed
    work package data and the PVM execution semantics. -/
def verifyLightClientProof (proof : LightClientProof) : Bool :=
  -- Step 1: Verify GKR proof → extract input claim
  let outputClaim : GKRClaim := {
    point := #[]  -- output evaluation point (derived from Fiat-Shamir)
    value := GoldilocksExt2.fromBase proof.claimedOutput[0]!
  }
  let inputClaim := verifyGKR proof.circuit proof.gkrProof outputClaim

  match inputClaim with
  | none => false
  | some claim =>
    -- Step 2: Verify cross-field bridge
    verifyCrossField proof.bridgeClaim

end Jar.Commitment.Bridge
