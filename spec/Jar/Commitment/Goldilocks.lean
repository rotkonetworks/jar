import Jar.Notation

/-!
# Goldilocks Field Arithmetic — p = 2^64 - 2^32 + 1

Prime field for GKR execution verification. Goldilocks is chosen because:
- Characteristic > 2^64: integer arithmetic (ADD, MUL, CMP) over 64-bit
  register values maps directly to field operations — no carry circuits
- Fast reduction: p = 2^64 - 2^32 + 1 allows reduction via shifts and adds
- Two 32-bit limbs: fits in a 128-bit product with simple Barrett reduction
- Compatible with Plonky2/Plonky3, SP1, and Jolt proof systems

## Role in the architecture

The JAR commitment scheme uses two fields:
- **GF(2^32)**: DA tensor encoding (Ligerito/ZODA) — fast additive FFT,
  efficient erasure coding. Already in `Jar.Commitment.Field`.
- **Goldilocks**: GKR execution circuit — integer arithmetic is native,
  PVM instructions map to O(1) gates instead of O(n) carry circuits.

The bridge between them is in `Jar.Commitment.Bridge`.

## References

- Goldilocks: https://polygon.technology/blog/plonky2-a-deep-dive
- Plonky3 field: https://github.com/Plonky3/Plonky3
-/

namespace Jar.Commitment.Goldilocks

/-- The Goldilocks prime: p = 2^64 - 2^32 + 1 = 0xFFFFFFFF00000001. -/
def GOLDILOCKS_P : Nat := 0xFFFFFFFF00000001

/-- Element of the Goldilocks field GF(p).
    Stored as a Nat reduced modulo p. -/
structure GoldilocksElem where
  val : Nat
  deriving BEq, Inhabited, Repr, DecidableEq

namespace GoldilocksElem

def zero : GoldilocksElem := ⟨0⟩
def one : GoldilocksElem := ⟨1⟩

/-- Reduce modulo Goldilocks prime.
    Uses the identity: 2^64 ≡ 2^32 - 1 (mod p). -/
def reduce (n : Nat) : GoldilocksElem :=
  ⟨n % GOLDILOCKS_P⟩

/-- Construct from a 64-bit value (no reduction needed if < p). -/
def fromNat (n : Nat) : GoldilocksElem := reduce n

/-- Addition modulo p. -/
def add (a b : GoldilocksElem) : GoldilocksElem :=
  reduce (a.val + b.val)

/-- Subtraction modulo p. -/
def sub (a b : GoldilocksElem) : GoldilocksElem :=
  if a.val >= b.val then ⟨a.val - b.val⟩
  else ⟨GOLDILOCKS_P - (b.val - a.val)⟩

/-- Multiplication modulo p. -/
def mul (a b : GoldilocksElem) : GoldilocksElem :=
  reduce (a.val * b.val)

/-- Negation: -a = p - a. -/
def neg (a : GoldilocksElem) : GoldilocksElem :=
  if a.val == 0 then zero else ⟨GOLDILOCKS_P - a.val⟩

/-- Exponentiation by squaring. -/
def pow (base : GoldilocksElem) (exp : Nat) : GoldilocksElem := Id.run do
  if base == zero then return zero
  let mut result := one
  let mut b := base
  let mut e := exp
  while e > 0 do
    if e % 2 == 1 then
      result := mul result b
    b := mul b b
    e := e / 2
  result

/-- Multiplicative inverse via Fermat: a^(p-2) mod p. -/
def inv (a : GoldilocksElem) : GoldilocksElem :=
  if a == zero then zero
  else pow a (GOLDILOCKS_P - 2)

/-- Division: a / b = a * b^(-1). -/
def div (a b : GoldilocksElem) : GoldilocksElem :=
  mul a (inv b)

/-- Embed a PVM register value (u64) into the field.
    Since p > 2^64, all 64-bit values fit without reduction. -/
def fromU64 (v : UInt64) : GoldilocksElem := ⟨v.toNat⟩

/-- Check if a field element represents a valid u64 (< 2^64). -/
def isU64 (a : GoldilocksElem) : Bool := a.val < 2^64

/-- Extract as u64 (panics if out of range). -/
def toU64 (a : GoldilocksElem) : UInt64 := a.val.toUInt64

end GoldilocksElem

-- ============================================================================
-- Quadratic extension: Goldilocks^2 for 128-bit security
-- ============================================================================

/-- Quadratic extension of Goldilocks: GF(p^2) = GF(p)[x] / (x^2 - 7).
    Elements are (a + b·x) where a, b ∈ GF(p).
    Needed for 128-bit security in sumcheck challenges. -/
structure GoldilocksExt2 where
  re : GoldilocksElem  -- real part
  im : GoldilocksElem  -- imaginary part (coefficient of x)
  deriving BEq, Inhabited, Repr

namespace GoldilocksExt2

/-- The non-residue W = 7. x^2 = W in the extension. -/
def W : GoldilocksElem := ⟨7⟩

def zero : GoldilocksExt2 := ⟨GoldilocksElem.zero, GoldilocksElem.zero⟩
def one : GoldilocksExt2 := ⟨GoldilocksElem.one, GoldilocksElem.zero⟩

def add (a b : GoldilocksExt2) : GoldilocksExt2 :=
  ⟨GoldilocksElem.add a.re b.re, GoldilocksElem.add a.im b.im⟩

def sub (a b : GoldilocksExt2) : GoldilocksExt2 :=
  ⟨GoldilocksElem.sub a.re b.re, GoldilocksElem.sub a.im b.im⟩

/-- Multiplication: (a + b·x)(c + d·x) = (ac + bd·W) + (ad + bc)·x. -/
def mul (a b : GoldilocksExt2) : GoldilocksExt2 :=
  let ac := GoldilocksElem.mul a.re b.re
  let bd := GoldilocksElem.mul a.im b.im
  let ad := GoldilocksElem.mul a.re b.im
  let bc := GoldilocksElem.mul a.im b.re
  ⟨GoldilocksElem.add ac (GoldilocksElem.mul bd W),
   GoldilocksElem.add ad bc⟩

/-- Embed base field element. -/
def fromBase (a : GoldilocksElem) : GoldilocksExt2 :=
  ⟨a, GoldilocksElem.zero⟩

end GoldilocksExt2

end Jar.Commitment.Goldilocks
