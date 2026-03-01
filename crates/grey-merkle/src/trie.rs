//! Binary Patricia Merkle Trie (Appendix D.2).
//!
//! 64-byte nodes, either branches or leaves.
//! Branch: 1-bit discriminator + two child hashes (255 + 256 bits).
//! Leaf: embedded-value or regular (with value hash).

use grey_types::Hash;

/// A node in the binary Patricia Merkle Trie.
#[derive(Clone, Debug)]
pub enum TrieNode {
    /// Empty sub-trie, identified by H₀.
    Empty,

    /// Branch node: left and right child hashes.
    Branch {
        left: Hash,
        right: Hash,
    },

    /// Leaf node with embedded value (≤ 32 bytes).
    EmbeddedLeaf {
        key: [u8; 31],
        value: Vec<u8>,
    },

    /// Leaf node with hashed value (> 32 bytes).
    HashedLeaf {
        key: [u8; 31],
        value_hash: Hash,
    },
}

impl TrieNode {
    /// Encode this node as 64 bytes (eq D.3-D.5).
    pub fn encode(&self) -> [u8; 64] {
        let mut node = [0u8; 64];
        match self {
            TrieNode::Empty => {} // All zeros = H₀

            TrieNode::Branch { left, right } => {
                // First bit = 0 (branch)
                // Remaining 255 bits of left, then 256 bits of right
                // left: bits 1..256 → bytes 0..31 (skipping first bit)
                // right: bits 256..512 → bytes 32..64
                node[0] = 0; // First bit = 0
                // Left child: use last 255 bits (skip MSB of first byte)
                node[0] |= left.0[0] & 0x7F; // 7 bits from left[0]
                node[1..32].copy_from_slice(&left.0[1..32]);
                node[32..64].copy_from_slice(&right.0);
            }

            TrieNode::EmbeddedLeaf { key, value } => {
                // First bit = 1 (leaf), second bit = 1 (embedded)
                // Remaining 6 bits = value length
                let len = value.len().min(32) as u8;
                node[0] = 0x80 | 0x40 | (len & 0x3F);
                node[1..32].copy_from_slice(key);
                node[32..32 + value.len().min(32)].copy_from_slice(&value[..value.len().min(32)]);
            }

            TrieNode::HashedLeaf { key, value_hash } => {
                // First bit = 1 (leaf), second bit = 0 (regular)
                // Remaining 6 bits = 0
                node[0] = 0x80;
                node[1..32].copy_from_slice(key);
                node[32..64].copy_from_slice(&value_hash.0);
            }
        }
        node
    }

    /// Compute the hash (identity) of this node.
    pub fn hash(&self) -> Hash {
        match self {
            TrieNode::Empty => Hash::ZERO,
            _ => grey_crypto::blake2b_256(&self.encode()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_node_is_zero_hash() {
        assert_eq!(TrieNode::Empty.hash(), Hash::ZERO);
    }

    #[test]
    fn test_embedded_leaf_encoding() {
        let node = TrieNode::EmbeddedLeaf {
            key: [0xAB; 31],
            value: vec![1, 2, 3],
        };
        let encoded = node.encode();
        // First byte: 0x80 | 0x40 | 3 = 0xC3
        assert_eq!(encoded[0], 0xC3);
        assert_eq!(&encoded[1..32], &[0xAB; 31]);
        assert_eq!(&encoded[32..35], &[1, 2, 3]);
    }

    #[test]
    fn test_hashed_leaf_encoding() {
        let node = TrieNode::HashedLeaf {
            key: [0xCD; 31],
            value_hash: Hash([0xFF; 32]),
        };
        let encoded = node.encode();
        assert_eq!(encoded[0], 0x80);
        assert_eq!(&encoded[1..32], &[0xCD; 31]);
        assert_eq!(&encoded[32..64], &[0xFF; 32]);
    }
}
