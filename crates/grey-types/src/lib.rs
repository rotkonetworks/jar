//! Core types, constants, and data structures for the JAM protocol.
//!
//! This crate defines the foundational types matching the Gray Paper specification v0.7.2.
//! Greek-letter state components are mapped to descriptive Rust names.

pub mod constants;
pub mod header;
pub mod state;
pub mod validator;
pub mod work;

use std::fmt;

/// A 32-byte cryptographic hash value (H in the spec).
/// Used for Blake2b-256 output, block hashes, state roots, etc.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    /// The zero hash H₀.
    pub const ZERO: Self = Self([0u8; 32]);

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", hex::encode(self.0))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// An Ed25519 public key (H̄ in the spec). Subset of B32.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Ed25519PublicKey(pub [u8; 32]);

impl fmt::Debug for Ed25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ed25519({})", hex::encode(self.0))
    }
}

/// A Bandersnatch public key (H̃ in the spec). Subset of B32.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BandersnatchPublicKey(pub [u8; 32]);

impl fmt::Debug for BandersnatchPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bandersnatch({})", hex::encode(self.0))
    }
}

/// A BLS12-381 public key (B^BLS in the spec). Subset of B144.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BlsPublicKey(pub [u8; 144]);

impl Default for BlsPublicKey {
    fn default() -> Self {
        Self([0u8; 144])
    }
}

impl fmt::Debug for BlsPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BLS({}...)", hex::encode(&self.0[..8]))
    }
}

/// A Bandersnatch ring root (B° in the spec). Subset of B144.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BandersnatchRingRoot(pub [u8; 144]);

impl Default for BandersnatchRingRoot {
    fn default() -> Self {
        Self([0u8; 144])
    }
}

impl fmt::Debug for BandersnatchRingRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RingRoot({}...)", hex::encode(&self.0[..8]))
    }
}

/// An Ed25519 signature. B64.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ed25519Signature(pub [u8; 64]);

impl Default for Ed25519Signature {
    fn default() -> Self {
        Self([0u8; 64])
    }
}

impl fmt::Debug for Ed25519Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ed25519Sig({}...)", hex::encode(&self.0[..8]))
    }
}

/// A Bandersnatch signature. B96.
#[derive(Clone, PartialEq, Eq)]
pub struct BandersnatchSignature(pub [u8; 96]);

impl Default for BandersnatchSignature {
    fn default() -> Self {
        Self([0u8; 96])
    }
}

impl fmt::Debug for BandersnatchSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BanderSig({}...)", hex::encode(&self.0[..8]))
    }
}

/// A Bandersnatch Ring VRF proof. B784.
#[derive(Clone, PartialEq, Eq)]
pub struct BandersnatchRingVrfProof(pub Vec<u8>);

/// A BLS signature.
#[derive(Clone, PartialEq, Eq)]
pub struct BlsSignature(pub Vec<u8>);

/// Balance type: NB = N_{2^64} (eq 4.21).
pub type Balance = u64;

/// Gas type: NG = N_{2^64} (eq 4.23).
pub type Gas = u64;

/// Signed gas type: ZG = Z_{-2^63...2^63} (eq 4.23).
pub type SignedGas = i64;

/// Service identifier: NS = N_{2^32} (eq 9.1).
pub type ServiceId = u32;

/// Timeslot index: NT = N_{2^32} (eq 4.28).
pub type Timeslot = u32;

/// Core index: NC = N_C where C = 341.
pub type CoreIndex = u16;

/// Validator index: NV = N_V where V = 1023.
pub type ValidatorIndex = u16;

/// Register value: NR = N_{2^64} (eq 4.23).
pub type RegisterValue = u64;

/// An opaque blob of bytes.
pub type Blob = Vec<u8>;
