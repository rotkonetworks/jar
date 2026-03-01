//! Fisher-Yates shuffle function F (Appendix F of the Gray Paper).

use grey_types::Hash;

/// Fisher-Yates shuffle of a sequence using a sequence of random naturals (eq F.1).
///
/// F: (⟦T⟧_l, ⟦N⟧_l) → ⟦T⟧_l
pub fn fisher_yates_shuffle<T: Clone>(sequence: &mut [T], entropy: &[u32]) {
    let n = sequence.len();
    if n <= 1 {
        return;
    }
    for i in (1..n).rev() {
        let j = entropy[n - 1 - i] as usize % (i + 1);
        sequence.swap(i, j);
    }
}

/// Generate a sequence of random u32 values from a hash (eq F.2).
///
/// Q_l: H → ⟦N_{2^32}⟧_l
pub fn random_sequence_from_hash(hash: &Hash, length: usize) -> Vec<u32> {
    let mut result = Vec::with_capacity(length);
    for i in 0..length {
        let chunk_index = i / 8;
        let within_chunk = (i % 8) * 4;

        // Hash the original hash concatenated with the chunk index
        let mut input = Vec::with_capacity(36);
        input.extend_from_slice(&hash.0);
        input.extend_from_slice(&(chunk_index as u32).to_le_bytes());

        let derived = crate::blake2b_256(&input);
        let start = within_chunk % 32;
        let value = u32::from_le_bytes([
            derived.0[start],
            derived.0[start + 1],
            derived.0[start + 2],
            derived.0[start + 3],
        ]);
        result.push(value);
    }
    result
}

/// Shuffle a sequence using a hash as entropy source (eq F.3).
///
/// F: (⟦T⟧_l, H) → ⟦T⟧_l
pub fn shuffle_with_hash<T: Clone>(sequence: &mut [T], hash: &Hash) {
    let entropy = random_sequence_from_hash(hash, sequence.len());
    fisher_yates_shuffle(sequence, &entropy);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shuffle_deterministic() {
        let hash = Hash([1u8; 32]);
        let mut seq1 = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut seq2 = seq1.clone();

        shuffle_with_hash(&mut seq1, &hash);
        shuffle_with_hash(&mut seq2, &hash);

        assert_eq!(seq1, seq2);
    }

    #[test]
    fn test_shuffle_preserves_elements() {
        let hash = Hash([42u8; 32]);
        let mut seq = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        shuffle_with_hash(&mut seq, &hash);

        let mut sorted = seq.clone();
        sorted.sort();
        assert_eq!(sorted, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_shuffle_empty() {
        let hash = Hash([0u8; 32]);
        let mut seq: Vec<u32> = vec![];
        shuffle_with_hash(&mut seq, &hash);
        assert!(seq.is_empty());
    }
}
