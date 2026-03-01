//! Reed-Solomon erasure coding in GF(2^16) for JAM data availability (Appendix H).
//!
//! Rate: 342:1023 using the algorithm of Lin, Chung, and Han 2014.
//! Uses a Cantor basis representation for efficient FFT operations.

/// The number of data chunks (systematic part).
pub const DATA_CHUNKS: usize = 342;

/// The total number of coded chunks (data + parity).
pub const TOTAL_CHUNKS: usize = 1023;

/// The number of parity chunks.
pub const PARITY_CHUNKS: usize = TOTAL_CHUNKS - DATA_CHUNKS;

/// Basic erasure piece size in octets (WE = 684).
pub const PIECE_SIZE: usize = 684;

/// Number of erasure-coded pieces per segment (WP = 6).
pub const PIECES_PER_SEGMENT: usize = 6;

/// Segment size in octets (WG = WP * WE = 4104).
pub const SEGMENT_SIZE: usize = PIECES_PER_SEGMENT * PIECE_SIZE;

/// Encode a data blob into TOTAL_CHUNKS coded chunks (eq H.4).
///
/// The input `data` must have a length that is a multiple of `PIECE_SIZE`.
/// Returns 1023 chunks, each of size `data.len() / DATA_CHUNKS * 2`.
pub fn encode(_data: &[u8]) -> Vec<Vec<u8>> {
    // TODO: Implement Reed-Solomon encoding in GF(2^16)
    // Using Lin-Chung-Han 2014 algorithm with Cantor basis
    unimplemented!("Reed-Solomon encoding not yet implemented")
}

/// Recover original data from any DATA_CHUNKS of the TOTAL_CHUNKS chunks (eq H.5).
///
/// Each element is (chunk_data, chunk_index).
pub fn recover(_chunks: &[(Vec<u8>, usize)]) -> Result<Vec<u8>, ErasureError> {
    // TODO: Implement Reed-Solomon recovery
    unimplemented!("Reed-Solomon recovery not yet implemented")
}

/// Errors from erasure coding operations.
#[derive(Debug)]
pub enum ErasureError {
    /// Not enough chunks to recover (need at least DATA_CHUNKS).
    InsufficientChunks { have: usize, need: usize },
    /// Invalid chunk index.
    InvalidIndex(usize),
    /// Chunk size mismatch.
    SizeMismatch,
}

impl std::fmt::Display for ErasureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientChunks { have, need } => {
                write!(f, "insufficient chunks: have {have}, need {need}")
            }
            Self::InvalidIndex(idx) => write!(f, "invalid chunk index: {idx}"),
            Self::SizeMismatch => write!(f, "chunk size mismatch"),
        }
    }
}

impl std::error::Error for ErasureError {}
