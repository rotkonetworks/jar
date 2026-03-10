import Jar.Notation

/-!
# Protocol Constants — Gray Paper Appendix I.4.4

All constant values from the GP specification.
References: `graypaper/text/definitions.tex` lines 240–290,
            `graypaper/preamble.tex` lines 248–289.
-/

namespace Jar

-- ============================================================================
-- Consensus & Validators
-- ============================================================================

/-- V : Total number of validators. GP: 𝖵 = 1023. -/
def V : Nat := 1023

/-- C : Total number of cores. GP: 𝖢 = 341. -/
def C : Nat := 341

/-- E : Epoch length in timeslots. GP: 𝖤 = 600. -/
def E : Nat := 600

/-- P : Slot period in seconds. GP: 𝖯 = 6. -/
def P : Nat := 6

/-- H_RECENT : Recent history size in blocks. GP: 𝖧 = 8. -/
def H_RECENT : Nat := 8

/-- N : Ticket entries per validator. GP: 𝖭 = 2. -/
def N_TICKETS : Nat := 2

/-- Y : Ticket submission end slot. GP: 𝖸 = 500. -/
def Y_TAIL : Nat := 500

/-- R : Validator-core rotation period in timeslots. GP: 𝖱 = 10. -/
def R_ROTATION : Nat := 10

/-- A : Audit tranche period in seconds. GP: 𝖠 = 8. -/
def A_TRANCHE : Nat := 8

/-- F : Audit bias factor. GP: 𝖥 = 2. -/
def F_BIAS : Nat := 2

-- ============================================================================
-- Work processing
-- ============================================================================

/-- I : Max work items per package. GP: 𝖨 = 16. -/
def I_MAX_ITEMS : Nat := 16

/-- J : Max dependency items in a work-report. GP: 𝖩 = 8. -/
def J_MAX_DEPS : Nat := 8

/-- K : Max tickets per extrinsic. GP: 𝖪 = 16. -/
def K_MAX_TICKETS : Nat := 16

/-- T : Max extrinsics per work-package. GP: 𝖳 = 128. -/
def T_MAX_EXTRINSICS : Nat := 128

/-- U : Availability timeout in timeslots. GP: 𝖴 = 5. -/
def U_TIMEOUT : Nat := 5

-- ============================================================================
-- Gas allocations
-- ============================================================================

/-- G_A : Gas allocated per work-report accumulation. GP: 𝖦_A = 10,000,000. -/
def G_A : Nat := 10_000_000

/-- G_I : Gas allocated for Is-Authorized. GP: 𝖦_I = 50,000,000. -/
def G_I : Nat := 50_000_000

/-- G_R : Gas allocated for Refine. GP: 𝖦_R = 5,000,000,000. -/
def G_R : Nat := 5_000_000_000

/-- G_T : Total accumulation gas per block. GP: 𝖦_T = 3,500,000,000. -/
def G_T : Nat := 3_500_000_000

-- ============================================================================
-- Authorization
-- ============================================================================

/-- O : Authorization pool size per core. GP: 𝖮 = 8. -/
def O_POOL : Nat := 8

/-- Q : Authorization queue size per core. GP: 𝖰 = 80. -/
def Q_QUEUE : Nat := 80

-- ============================================================================
-- Preimages and lookups
-- ============================================================================

/-- D : Preimage expunge period in timeslots. GP: 𝖣 = 19,200. -/
def D_EXPUNGE : Nat := 19_200

/-- L : Max lookup anchor age in timeslots. GP: 𝖫 = 14,400 (= 24 hours). -/
def L_MAX_ANCHOR : Nat := 14_400

-- ============================================================================
-- Size limits
-- ============================================================================

/-- W_A : Max is-authorized code size. GP: 𝖶_A = 64,000. -/
def W_A : Nat := 64_000

/-- W_B : Max work-package blob size. GP: 𝖶_B = 13,791,360. -/
def W_B : Nat := 13_791_360

/-- W_C : Max service code size. GP: 𝖶_C = 4,000,000. -/
def W_C : Nat := 4_000_000

/-- W_E : Erasure coding piece size. GP: 𝖶_E = 684. -/
def W_E : Nat := 684

/-- W_G : Segment size (= W_P × W_E). GP: 𝖶_G = 4,104. -/
def W_G : Nat := 4_104

/-- W_M : Max segment imports. GP: 𝖶_M = 3,072. -/
def W_M : Nat := 3_072

/-- W_P : Erasure pieces per segment. GP: 𝖶_P = 6. -/
def W_P : Nat := 6

/-- W_R : Max work-report variable-size blob. GP: 𝖶_R = 49,152. -/
def W_R : Nat := 49_152

/-- W_T : Transfer memo size. GP: 𝖶_T = 128. -/
def W_T : Nat := 128

/-- W_X : Max segment exports. GP: 𝖶_X = 3,072. -/
def W_X : Nat := 3_072

-- ============================================================================
-- PVM configuration
-- ============================================================================

/-- Z_P : PVM page size. GP: 𝖹_P = 2^12 = 4,096. -/
def Z_P : Nat := 4096

/-- Z_Z : PVM initialization zone size. GP: 𝖹_Z = 2^16 = 65,536. -/
def Z_Z : Nat := 65536

/-- Z_I : PVM initialization input size. GP: 𝖹_I = 2^24 = 16,777,216. -/
def Z_I : Nat := 16_777_216

/-- Z_A : PVM dynamic address alignment. GP: 𝖹_A = 2. -/
def Z_A : Nat := 2

/-- Number of PVM registers. 13 in the GP. -/
def PVM_REGISTERS : Nat := 13

-- ============================================================================
-- Economic constants (GP: B_I, B_L, B_S)
-- ============================================================================

/-- B_I : Additional minimum balance per mapping item. GP: 𝖡_I.
    Value TBD per GP §4.6. Using 10 as placeholder. -/
def B_I : Nat := 10

/-- B_L : Additional minimum balance per data octet. GP: 𝖡_L.
    Value TBD. Using 1 as placeholder. -/
def B_L : Nat := 1

/-- B_S : Base minimum balance for a service. GP: 𝖡_S.
    Value TBD. Using 100 as placeholder. -/
def B_S : Nat := 100

-- ============================================================================
-- Minimum public service index
-- ============================================================================

/-- S : Minimum public service index. GP: 𝖲 = 256. -/
def S_MIN : Nat := 256

-- ============================================================================
-- Time
-- ============================================================================

/-- JAM Common Era epoch: 1200 UTC on January 1, 2025.
    = 1,735,732,800 seconds after Unix Epoch. -/
def JAM_EPOCH_UNIX : Nat := 1_735_732_800

end Jar
