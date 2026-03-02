//! Decoding functions (Appendix C of the Gray Paper).

use crate::error::CodecError;

/// Trait for types that can be decoded from the JAM wire format.
pub trait Decode: Sized {
    /// Decode a value from the given byte slice, returning the value
    /// and the number of bytes consumed.
    fn decode(data: &[u8]) -> Result<(Self, usize), CodecError>;
}

/// Decode a JAM compact/variable-length natural number (inverse of encode_natural/encode_compact).
///
/// JAM prefix-length encoding: leading 1-bits of the first byte indicate
/// the number of additional bytes. The first byte also carries high bits
/// of the value; remaining bytes are little-endian.
///
/// Returns `(value, bytes_consumed)`.
pub fn decode_natural(data: &[u8]) -> Result<(usize, usize), CodecError> {
    let (val, consumed) = decode_compact(data)?;
    Ok((val as usize, consumed))
}

/// Decode a JAM compact-encoded u64 value.
///
/// Returns `(value, bytes_consumed)`.
pub fn decode_compact(data: &[u8]) -> Result<(u64, usize), CodecError> {
    ensure_bytes(data, 1)?;
    let header = data[0];
    let len = header.leading_ones() as usize; // 0..=8

    if len == 8 {
        // 0xFF: read next 8 bytes as u64 LE
        ensure_bytes(data, 9)?;
        let value = u64::from_le_bytes(data[1..9].try_into().unwrap());
        return Ok((value, 9));
    }

    ensure_bytes(data, 1 + len)?;

    // Threshold: the minimum header value for this length class
    let threshold: u64 = if len == 0 {
        0
    } else {
        256 - (1u64 << (8 - len))
    };

    // High bits from header byte
    let header_value = (header as u64) - threshold;

    // Low bits from remaining bytes (little-endian)
    let mut low: u64 = 0;
    for i in 0..len {
        low |= (data[1 + i] as u64) << (8 * i);
    }

    let value = (header_value << (8 * len)) | low;
    Ok((value, 1 + len))
}

fn ensure_bytes(data: &[u8], needed: usize) -> Result<(), CodecError> {
    if data.len() < needed {
        Err(CodecError::UnexpectedEof {
            needed,
            available: data.len(),
        })
    } else {
        Ok(())
    }
}

impl Decode for u8 {
    fn decode(data: &[u8]) -> Result<(Self, usize), CodecError> {
        ensure_bytes(data, 1)?;
        Ok((data[0], 1))
    }
}

impl Decode for u16 {
    fn decode(data: &[u8]) -> Result<(Self, usize), CodecError> {
        ensure_bytes(data, 2)?;
        Ok((u16::from_le_bytes([data[0], data[1]]), 2))
    }
}

impl Decode for u32 {
    fn decode(data: &[u8]) -> Result<(Self, usize), CodecError> {
        ensure_bytes(data, 4)?;
        Ok((
            u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            4,
        ))
    }
}

impl Decode for u64 {
    fn decode(data: &[u8]) -> Result<(Self, usize), CodecError> {
        ensure_bytes(data, 8)?;
        Ok((
            u64::from_le_bytes([
                data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
            ]),
            8,
        ))
    }
}

impl Decode for grey_types::Hash {
    fn decode(data: &[u8]) -> Result<(Self, usize), CodecError> {
        ensure_bytes(data, 32)?;
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&data[..32]);
        Ok((grey_types::Hash(bytes), 32))
    }
}

impl Decode for grey_types::Ed25519PublicKey {
    fn decode(data: &[u8]) -> Result<(Self, usize), CodecError> {
        ensure_bytes(data, 32)?;
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&data[..32]);
        Ok((grey_types::Ed25519PublicKey(bytes), 32))
    }
}

impl Decode for grey_types::BandersnatchSignature {
    fn decode(data: &[u8]) -> Result<(Self, usize), CodecError> {
        ensure_bytes(data, 96)?;
        let mut bytes = [0u8; 96];
        bytes.copy_from_slice(&data[..96]);
        Ok((grey_types::BandersnatchSignature(bytes), 96))
    }
}

impl Decode for grey_types::Ed25519Signature {
    fn decode(data: &[u8]) -> Result<(Self, usize), CodecError> {
        ensure_bytes(data, 64)?;
        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(&data[..64]);
        Ok((grey_types::Ed25519Signature(bytes), 64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{encode_natural, Encode};

    #[test]
    fn test_decode_natural_roundtrip() {
        for value in [0, 1, 127, 128, 255, 300, 16384, 1_000_000] {
            let mut buf = Vec::new();
            encode_natural(value, &mut buf);
            let (decoded, consumed) = decode_natural(&buf).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(consumed, buf.len());
        }
    }

    #[test]
    fn test_decode_u32_roundtrip() {
        let value: u32 = 0x12345678;
        let encoded = value.encode();
        let (decoded, consumed) = u32::decode(&encoded).unwrap();
        assert_eq!(decoded, value);
        assert_eq!(consumed, 4);
    }

    #[test]
    fn test_decode_hash_roundtrip() {
        let hash = grey_types::Hash([0xAB; 32]);
        let encoded = crate::encode::Encode::encode(&hash);
        let (decoded, consumed) = grey_types::Hash::decode(&encoded).unwrap();
        assert_eq!(decoded, hash);
        assert_eq!(consumed, 32);
    }
}
