import Jar.Commitment.Goldilocks

/-!
# GKR Protocol — Layered Circuit Verification via Sumcheck

The GKR protocol (Goldwasser-Kalai-Rothblum 2008) verifies layered
arithmetic circuits by running one sumcheck per layer, from output
to input. The final claim reduces to a single polynomial evaluation
at the input layer — which the DA encoding provides for free.

## Architecture

```
PVM Execution Trace (layered circuit over Goldilocks)
  ↓
GKR sumcheck (per-layer, output → input)
  ↓
Final claim: W(r) = v at random point r
  ↓
Bridge: verify W(r) against DA commitment (GF(2^32) tensor encoding)
  ↓
Light client accepts WorkResult without re-executing PVM
```

## PVM as a layered circuit

Each PVM instruction is one layer. The wiring function describes
how registers and memory map between layers:

- **Layer 0 (output)**: WorkResult hash, gas_used, exit reason
- **Layer k (instruction k from end)**: register file + memory state
  after executing instruction k
- **Layer N (input)**: initial PVM state (work package payload,
  code blob, initial registers)

Gate types over Goldilocks:
- ADD(a, b) = a + b — one field addition (u64 add is native!)
- MUL(a, b) = a * b — one field multiplication
- SUB(a, b) = a - b — one field subtraction
- LT(a, b) — bit decomposition + comparison (~64 gates)
- LOAD/STORE — memory checking via permutation argument

## Cost

- Prover: O(N) field operations where N = circuit size (trace length × width)
- Verifier: O(d · log N) where d = depth (number of layers)
- The "accidental computer" saves the polynomial commitment: the DA
  encoding IS the input layer commitment, so W(r) at the input layer
  comes from the tensor encoding for free.

## References

- GKR: Goldwasser, Kalai, Rothblum 2008
- Thaler, "Proofs, Arguments, and Zero-Knowledge" §4.6
- The Accidental Computer §3-4
- Jolt (Lasso + Surge lookups): https://eprint.iacr.org/2023/1217
-/

namespace Jar.Commitment.GKR

open Jar.Commitment.Goldilocks

-- ============================================================================
-- Layered Circuit Representation
-- ============================================================================

/-- A gate in the arithmetic circuit. -/
inductive Gate where
  | add (left right : Nat)   -- output = in[left] + in[right]
  | mul (left right : Nat)   -- output = in[left] * in[right]
  | const (val : GoldilocksElem)  -- output = constant
  deriving Inhabited

/-- A layer in the circuit: a list of gates.
    Each gate reads from the PREVIOUS layer's outputs. -/
structure Layer where
  gates : Array Gate
  deriving Inhabited

/-- A layered arithmetic circuit.
    Layer 0 is the output, layer (depth-1) is closest to input. -/
structure LayeredCircuit where
  layers : Array Layer
  inputSize : Nat  -- number of input wires
  deriving Inhabited

namespace LayeredCircuit

def depth (c : LayeredCircuit) : Nat := c.layers.size

def outputSize (c : LayeredCircuit) : Nat :=
  if c.layers.isEmpty then 0
  else c.layers[0]!.gates.size

end LayeredCircuit

-- ============================================================================
-- GKR Sumcheck Round
-- ============================================================================

/-- GKR round claim: at layer i, the verifier holds a claim
    V_i(r) = v for some random point r and claimed value v. -/
structure GKRClaim where
  /-- Random evaluation point (from previous round's sumcheck). -/
  point : Array GoldilocksExt2
  /-- Claimed evaluation value. -/
  value : GoldilocksExt2

/-- GKR sumcheck round data for one layer.
    Proves: Σ_{b,c ∈ {0,1}^k} add_i(r,b,c)·(V_{i+1}(b) + V_{i+1}(c))
                              + mul_i(r,b,c)·V_{i+1}(b)·V_{i+1}(c)
          = V_i(r)
    reducing to claims about V_{i+1} at random points. -/
structure GKRLayerProof where
  /-- Sumcheck round polynomials (degree-2 univariates). -/
  roundPolys : Array (GoldilocksExt2 × GoldilocksExt2 × GoldilocksExt2)
  /-- Claimed V_{i+1} evaluations at the random points. -/
  claimedEvals : GoldilocksExt2 × GoldilocksExt2

/-- Complete GKR proof for a layered circuit. -/
structure GKRProof where
  /-- Per-layer sumcheck proofs, from output to input. -/
  layerProofs : Array GKRLayerProof
  /-- Final claim at the input layer: V_input(r) = v.
      This is verified against the DA polynomial commitment. -/
  inputClaim : GKRClaim

-- ============================================================================
-- GKR Verifier
-- ============================================================================

/-- Verify a GKR proof for a layered circuit.

    The verifier:
    1. Starts with the claimed output: V_0(r_0) = output_value
    2. For each layer i: runs sumcheck verifier, reduces claim from
       layer i to layer i+1
    3. At the input layer: obtains claim V_input(r_final) = v_final
    4. Returns the input claim for cross-field verification against
       the DA commitment (via Bridge)

    The verifier does NOT check the input claim itself — that's
    the Bridge's job (cross-field: Goldilocks → GF(2^32)). -/
def verifyGKR (_circuit : LayeredCircuit) (_proof : GKRProof)
    (_outputClaim : GKRClaim) : Option GKRClaim :=
  -- TODO: implement per-layer sumcheck verification
  -- For each layer:
  --   1. Check sumcheck round consistency
  --   2. Reduce claim to next layer via random linear combination
  --   3. Verify wiring function at the random point
  --
  -- Returns the final input-layer claim if all checks pass.
  some _proof.inputClaim

-- ============================================================================
-- PVM Instruction Set as Circuit Layers
-- ============================================================================

/-- PVM register file state (13 registers, each a Goldilocks element).
    In Goldilocks, u64 values embed directly: no carry circuits needed. -/
structure PVMRegisters where
  regs : Array GoldilocksElem  -- length 13
  pc : GoldilocksElem
  gas : GoldilocksElem
  deriving Inhabited

/-- Encode one PVM instruction as a circuit layer.
    Each instruction reads the previous register state and produces
    the next register state.

    Over Goldilocks (char > 2^64), the gate count per instruction:
    - ADD/SUB: 1 gate (field add/sub IS integer add/sub for values < p)
    - MUL: 1 gate (field mul with range check on output)
    - SLT/SLTU: ~64 gates (bit decomposition for comparison)
    - AND/OR/XOR: ~64 gates (bitwise ops via bit decomposition)
    - SLL/SRL/SRA: ~64 gates (shift via bit decomposition)
    - LOAD/STORE: ~1 gate + permutation argument for memory consistency
    - BRANCH: ~1 gate (conditional select on comparison result)

    Compare with GF(2^32) where ADD alone needs ~96 gates (carry chain).
    The ~100x improvement for arithmetic instructions is why Goldilocks
    is the right field for execution verification. -/
def instructionToLayer (_opcode : UInt8) (_operands : Array GoldilocksElem)
    : Layer :=
  -- TODO: implement per-opcode circuit generation
  -- This is the core of the GKR execution proof.
  -- Each opcode type produces a different wiring pattern.
  { gates := #[] }

end Jar.Commitment.GKR
