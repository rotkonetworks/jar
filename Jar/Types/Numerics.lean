import Jar.Notation
import Jar.Types.Constants

/-!
# Numeric Type Aliases — Gray Paper §3.4, §4.6–4.7

Bounded numeric types used throughout the specification.
References: `graypaper/text/overview.tex` eq:balance, eq:gasregentry, eq:time.
-/

namespace Jar

-- ============================================================================
-- §4.6 — Balances (eq:balance)
-- ============================================================================

/-- ℕ_B ≡ ℕ_{2^64} : Balance values (64-bit unsigned). GP eq (19). -/
abbrev Balance := UInt64

-- ============================================================================
-- §4.7 — Gas and Registers (eq:gasregentry)
-- ============================================================================

/-- ℕ_G ≡ ℕ_{2^64} : Unsigned gas values (64-bit unsigned). GP eq (24). -/
abbrev Gas := UInt64

/-- ℤ_G ≡ ℤ_{-2^63..2^63} : Signed gas values (64-bit signed). GP eq (24). -/
abbrev SignedGas := Int64

/-- ℕ_R ≡ ℕ_{2^64} : PVM register values (64-bit unsigned). GP eq (24). -/
abbrev RegisterValue := UInt64

-- ============================================================================
-- §4.8 — Time (eq:time)
-- ============================================================================

/-- ℕ_T ≡ ℕ_{2^32} : Timeslot index (32-bit unsigned). GP eq (28). -/
abbrev Timeslot := UInt32

-- ============================================================================
-- §9 — Service identifiers (eq:serviceaccounts)
-- ============================================================================

/-- ℕ_S ≡ ℕ_{2^32} : Service identifier (32-bit unsigned). GP §9. -/
abbrev ServiceId := UInt32

-- ============================================================================
-- §3.4 — Blob lengths
-- ============================================================================

/-- ℕ_L ≡ ℕ_{2^32} : Blob length values. GP §3.4. -/
abbrev BlobLength := UInt32

-- ============================================================================
-- Index types (derived from constants)
-- ============================================================================

/-- Core index: ℕ_{C} where C = 341. -/
abbrev CoreIndex := Fin Jar.C

/-- Validator index: ℕ_{V} where V = 1023. -/
abbrev ValidatorIndex := Fin Jar.V

/-- Ticket entry index: ℕ_{N} where N = 2. -/
abbrev TicketEntryIndex := Fin Jar.N_TICKETS

/-- Epoch slot index: ℕ_{E} where E = 600. -/
abbrev EpochIndex := Fin Jar.E

end Jar
