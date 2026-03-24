//! Tensor DA commitment for JAM data availability.
//!
//! Arranges work package bundle data as a matrix of GF(2^32) field elements,
//! RS-encodes columns via binary field FFT, and commits rows via Merkle tree.
//! The resulting root replaces the flat chunk Merkle tree as erasure_root.
//!
//! Supports **row opening proofs**: given row indices, open those rows with a
//! batched Merkle proof. Verifier checks against erasure_root in ~8µs.
//!
//! # Architecture
//!
//! ```text
//! bundle bytes → GF(2^32) elements → m×n matrix → RS-encode columns → Merkle rows → root
//! ```
//!
//! The EncodedBlock stores the full encoded matrix + Merkle tree, enabling
//! row opening proofs on demand.
//!
//! # Modules
//!
//! - [`field`]: GF(2^32) and GF(2^128) binary field arithmetic
//! - [`reed_solomon`]: Binary field FFT and systematic RS encoding
//! - [`merkle`]: BLAKE3 Merkle tree with batched inclusion proofs
//! - [`encode`]: Column-major matrix encoding pipeline
//! - [`da`]: Tensor encoding + row opening API
//! - [`utils`]: Lagrange basis, multilinear polynomial operations

#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod field;
pub mod reed_solomon;
pub mod merkle;
pub mod encode;
pub mod da;
pub mod utils;
mod error;

pub use error::Error;
pub type Result<T> = core::result::Result<T, Error>;
