//! PVM instruction set (Appendix A.5 of the Gray Paper).
//!
//! The PVM uses a RISC-V rv64em inspired ISA with variable-length instructions.
//! Instructions are categorized by their argument types.

/// PVM opcodes (ζᵢ values from Appendix A.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // A.5.1: Instructions with no arguments
    Trap = 0,
    Fallthrough = 17,

    // A.5.2: Instructions with one immediate argument
    Ecalli = 78,

    // A.5.3: Instructions with arguments of one register and one immediate
    JumpInd = 19,
    LoadImm = 4,
    LoadImmU64 = 52,

    // A.5.4: Instructions with one register and two immediates
    StoreImmU8 = 62,
    StoreImmU16 = 79,
    StoreImmU32 = 69,
    StoreImmU64 = 67,

    // A.5.7: Instructions with one immediate and one offset
    LoadImmJump = 80,
    BranchEqImm = 81,
    BranchNeImm = 82,
    BranchLtUImm = 83,
    BranchLeUImm = 84,
    BranchGeUImm = 85,
    BranchGtUImm = 86,
    BranchLtSImm = 87,
    BranchLeSImm = 88,
    BranchGeSImm = 89,
    BranchGtSImm = 90,

    // A.5.9: Two-register instructions
    MoveReg = 100,
    Sbrk = 101,
    CountSetBits64 = 102,
    CountSetBits32 = 103,
    LeadingZeroBits64 = 104,
    LeadingZeroBits32 = 105,
    TrailingZeroBits64 = 106,
    TrailingZeroBits32 = 107,
    SignExtend8 = 108,
    SignExtend16 = 109,
    ZeroExtend16 = 110,
    ReverseBytes = 111,

    // A.5.10: Two registers + one immediate
    StoreIndU8 = 120,
    StoreIndU16 = 121,
    StoreIndU32 = 122,
    StoreIndU64 = 123,
    LoadIndU8 = 124,
    LoadIndI8 = 125,
    LoadIndU16 = 126,
    LoadIndI16 = 127,
    LoadIndU32 = 128,
    LoadIndI32 = 129,
    LoadIndU64 = 130,
    AddImm32 = 131,
    AndImm = 132,
    XorImm = 133,
    OrImm = 134,
    MulImm32 = 135,
    SetLtUImm = 136,
    SetLtSImm = 137,
    ShloLImm32 = 138,
    ShloRImm32 = 139,
    SharRImm32 = 140,
    NegAddImm32 = 141,
    SetGtUImm = 142,
    SetGtSImm = 143,
    ShloLImm64 = 144,
    ShloRImm64 = 145,
    SharRImm64 = 146,
    NegAddImm64 = 147,
    MulImm64 = 148,
    AddImm64 = 149,

    // A.5.11: Two registers + one offset
    BranchEq = 150,
    BranchNe = 151,
    BranchLtU = 152,
    BranchLeU = 153,
    BranchGeU = 154,
    BranchGtU = 155,
    BranchLtS = 156,
    BranchLeS = 157,
    BranchGeS = 158,
    BranchGtS = 159,

    // A.5.12: Three-register instructions
    Add32 = 160,
    Sub32 = 161,
    And = 162,
    Xor = 163,
    Or = 164,
    Mul32 = 165,
    DivU32 = 166,
    DivS32 = 167,
    RemU32 = 168,
    RemS32 = 169,
    ShloL32 = 170,
    ShloR32 = 171,
    SharR32 = 172,
    Add64 = 173,
    Sub64 = 174,
    Mul64 = 175,
    DivU64 = 176,
    DivS64 = 177,
    RemU64 = 178,
    RemS64 = 179,
    ShloL64 = 180,
    ShloR64 = 181,
    SharR64 = 182,
    SetLtU = 183,
    SetLtS = 184,
    CmovIz = 185,
    CmovNz = 186,

    // Three-register load/store
    StoreIndRegU8 = 187,
    StoreIndRegU16 = 188,
    StoreIndRegU32 = 189,
    StoreIndRegU64 = 190,
    LoadIndRegU8 = 191,
    LoadIndRegI8 = 192,
    LoadIndRegU16 = 193,
    LoadIndRegI16 = 194,
    LoadIndRegU32 = 195,
    LoadIndRegI32 = 196,
    LoadIndRegU64 = 197,
}

impl Opcode {
    /// Try to decode an opcode from a byte.
    pub fn from_byte(byte: u8) -> Option<Self> {
        // Use a match to validate the opcode
        match byte {
            0 => Some(Self::Trap),
            4 => Some(Self::LoadImm),
            17 => Some(Self::Fallthrough),
            19 => Some(Self::JumpInd),
            52 => Some(Self::LoadImmU64),
            62 => Some(Self::StoreImmU8),
            67 => Some(Self::StoreImmU64),
            69 => Some(Self::StoreImmU32),
            78 => Some(Self::Ecalli),
            79 => Some(Self::StoreImmU16),
            80 => Some(Self::LoadImmJump),
            81..=90 => {
                // Branch instructions with immediate
                Some(match byte {
                    81 => Self::BranchEqImm,
                    82 => Self::BranchNeImm,
                    83 => Self::BranchLtUImm,
                    84 => Self::BranchLeUImm,
                    85 => Self::BranchGeUImm,
                    86 => Self::BranchGtUImm,
                    87 => Self::BranchLtSImm,
                    88 => Self::BranchLeSImm,
                    89 => Self::BranchGeSImm,
                    90 => Self::BranchGtSImm,
                    _ => unreachable!(),
                })
            }
            100..=111 => {
                Some(match byte {
                    100 => Self::MoveReg,
                    101 => Self::Sbrk,
                    102 => Self::CountSetBits64,
                    103 => Self::CountSetBits32,
                    104 => Self::LeadingZeroBits64,
                    105 => Self::LeadingZeroBits32,
                    106 => Self::TrailingZeroBits64,
                    107 => Self::TrailingZeroBits32,
                    108 => Self::SignExtend8,
                    109 => Self::SignExtend16,
                    110 => Self::ZeroExtend16,
                    111 => Self::ReverseBytes,
                    _ => unreachable!(),
                })
            }
            120..=149 => {
                Some(match byte {
                    120 => Self::StoreIndU8,
                    121 => Self::StoreIndU16,
                    122 => Self::StoreIndU32,
                    123 => Self::StoreIndU64,
                    124 => Self::LoadIndU8,
                    125 => Self::LoadIndI8,
                    126 => Self::LoadIndU16,
                    127 => Self::LoadIndI16,
                    128 => Self::LoadIndU32,
                    129 => Self::LoadIndI32,
                    130 => Self::LoadIndU64,
                    131 => Self::AddImm32,
                    132 => Self::AndImm,
                    133 => Self::XorImm,
                    134 => Self::OrImm,
                    135 => Self::MulImm32,
                    136 => Self::SetLtUImm,
                    137 => Self::SetLtSImm,
                    138 => Self::ShloLImm32,
                    139 => Self::ShloRImm32,
                    140 => Self::SharRImm32,
                    141 => Self::NegAddImm32,
                    142 => Self::SetGtUImm,
                    143 => Self::SetGtSImm,
                    144 => Self::ShloLImm64,
                    145 => Self::ShloRImm64,
                    146 => Self::SharRImm64,
                    147 => Self::NegAddImm64,
                    148 => Self::MulImm64,
                    149 => Self::AddImm64,
                    _ => unreachable!(),
                })
            }
            150..=159 => {
                Some(match byte {
                    150 => Self::BranchEq,
                    151 => Self::BranchNe,
                    152 => Self::BranchLtU,
                    153 => Self::BranchLeU,
                    154 => Self::BranchGeU,
                    155 => Self::BranchGtU,
                    156 => Self::BranchLtS,
                    157 => Self::BranchLeS,
                    158 => Self::BranchGeS,
                    159 => Self::BranchGtS,
                    _ => unreachable!(),
                })
            }
            160..=197 => {
                Some(match byte {
                    160 => Self::Add32,
                    161 => Self::Sub32,
                    162 => Self::And,
                    163 => Self::Xor,
                    164 => Self::Or,
                    165 => Self::Mul32,
                    166 => Self::DivU32,
                    167 => Self::DivS32,
                    168 => Self::RemU32,
                    169 => Self::RemS32,
                    170 => Self::ShloL32,
                    171 => Self::ShloR32,
                    172 => Self::SharR32,
                    173 => Self::Add64,
                    174 => Self::Sub64,
                    175 => Self::Mul64,
                    176 => Self::DivU64,
                    177 => Self::DivS64,
                    178 => Self::RemU64,
                    179 => Self::RemS64,
                    180 => Self::ShloL64,
                    181 => Self::ShloR64,
                    182 => Self::SharR64,
                    183 => Self::SetLtU,
                    184 => Self::SetLtS,
                    185 => Self::CmovIz,
                    186 => Self::CmovNz,
                    187 => Self::StoreIndRegU8,
                    188 => Self::StoreIndRegU16,
                    189 => Self::StoreIndRegU32,
                    190 => Self::StoreIndRegU64,
                    191 => Self::LoadIndRegU8,
                    192 => Self::LoadIndRegI8,
                    193 => Self::LoadIndRegU16,
                    194 => Self::LoadIndRegI16,
                    195 => Self::LoadIndRegU32,
                    196 => Self::LoadIndRegI32,
                    197 => Self::LoadIndRegU64,
                    _ => unreachable!(),
                })
            }
            _ => None,
        }
    }

    /// Gas cost for this instruction (ϱΔ).
    /// All standard instructions cost 1 gas unit.
    pub fn gas_cost(&self) -> u64 {
        1
    }
}
