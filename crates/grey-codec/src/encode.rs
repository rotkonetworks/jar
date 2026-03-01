//! Encoding functions (Appendix C of the Gray Paper).

/// Trait for types that can be encoded to the JAM wire format.
pub trait Encode {
    /// Encode this value, appending bytes to the given buffer.
    fn encode_to(&self, buf: &mut Vec<u8>);

    /// Encode this value and return the bytes.
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode_to(&mut buf);
        buf
    }
}

/// Encode a variable-length natural number (eq C.1-C.4).
///
/// Used as a length prefix for variable-length sequences.
pub fn encode_natural(value: usize, buf: &mut Vec<u8>) {
    let mut v = value;
    loop {
        if v < 128 {
            buf.push(v as u8);
            break;
        }
        buf.push((v as u8 & 0x7F) | 0x80);
        v >>= 7;
    }
}

// Fixed-width little-endian integer encodings (eq C.12).

impl Encode for u8 {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.push(*self);
    }
}

impl Encode for u16 {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encode for u32 {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encode for u64 {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encode for bool {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.push(if *self { 1 } else { 0 });
    }
}

impl Encode for [u8; 32] {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self);
    }
}

impl Encode for [u8; 64] {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self);
    }
}

impl Encode for [u8; 96] {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self);
    }
}

impl Encode for grey_types::Hash {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.0);
    }
}

impl Encode for grey_types::Ed25519PublicKey {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.0);
    }
}

impl Encode for grey_types::BandersnatchPublicKey {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.0);
    }
}

impl Encode for grey_types::BandersnatchSignature {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.0);
    }
}

impl Encode for grey_types::Ed25519Signature {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.0);
    }
}

/// Encode a variable-length sequence with length prefix.
impl<T: Encode> Encode for Vec<T> {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        encode_natural(self.len(), buf);
        for item in self {
            item.encode_to(buf);
        }
    }
}

/// Encode an optional value with a discriminator byte (eq C.5-C.7).
impl<T: Encode> Encode for Option<T> {
    fn encode_to(&self, buf: &mut Vec<u8>) {
        match self {
            None => buf.push(0),
            Some(val) => {
                buf.push(1);
                val.encode_to(buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_natural_small() {
        let mut buf = Vec::new();
        encode_natural(0, &mut buf);
        assert_eq!(buf, vec![0]);

        let mut buf = Vec::new();
        encode_natural(127, &mut buf);
        assert_eq!(buf, vec![127]);
    }

    #[test]
    fn test_encode_natural_large() {
        let mut buf = Vec::new();
        encode_natural(128, &mut buf);
        assert_eq!(buf, vec![0x80, 0x01]);

        let mut buf = Vec::new();
        encode_natural(300, &mut buf);
        assert_eq!(buf, vec![0xAC, 0x02]);
    }

    #[test]
    fn test_encode_u32_le() {
        let val: u32 = 0x12345678;
        let encoded = val.encode();
        assert_eq!(encoded, vec![0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_encode_hash() {
        let hash = grey_types::Hash([0xAB; 32]);
        let encoded = hash.encode();
        assert_eq!(encoded.len(), 32);
        assert!(encoded.iter().all(|&b| b == 0xAB));
    }
}
