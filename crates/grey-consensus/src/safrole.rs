//! Safrole consensus mechanism (Section 6 of the Gray Paper).
//!
//! Key operations:
//! - Outside-in sequencer Z for ordering tickets (eq 6.25)
//! - Fallback key sequence F (eq 6.26)
//! - Seal-key series generation (eq 6.24)

use grey_types::header::Ticket;

/// Outside-in sequencer Z (eq 6.25).
///
/// Reorders a sequence [s₀, s₁, ..., s_{n-1}] as [s₀, s_{n-1}, s₁, s_{n-2}, ...].
pub fn outside_in_sequence<T: Clone>(items: &[T]) -> Vec<T> {
    let n = items.len();
    let mut result = Vec::with_capacity(n);
    let mut lo = 0;
    let mut hi = n.wrapping_sub(1);

    for i in 0..n {
        if i % 2 == 0 {
            result.push(items[lo].clone());
            lo += 1;
        } else {
            result.push(items[hi].clone());
            hi = hi.wrapping_sub(1);
        }
    }

    result
}

/// Merge new tickets into the ticket accumulator, keeping only the lowest E entries (eq 6.34).
pub fn merge_tickets(
    existing: &[Ticket],
    new_tickets: &[Ticket],
    max_size: usize,
) -> Vec<Ticket> {
    let mut all: Vec<Ticket> = existing.to_vec();
    all.extend(new_tickets.iter().cloned());

    // Sort by ticket identifier (ascending)
    all.sort_by(|a, b| a.id.0.cmp(&b.id.0));

    // Keep only the lowest max_size entries
    all.truncate(max_size);
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outside_in_even() {
        let items = vec![0, 1, 2, 3, 4, 5];
        let result = outside_in_sequence(&items);
        assert_eq!(result, vec![0, 5, 1, 4, 2, 3]);
    }

    #[test]
    fn test_outside_in_odd() {
        let items = vec![0, 1, 2, 3, 4];
        let result = outside_in_sequence(&items);
        assert_eq!(result, vec![0, 4, 1, 3, 2]);
    }

    #[test]
    fn test_outside_in_empty() {
        let items: Vec<i32> = vec![];
        let result = outside_in_sequence(&items);
        assert!(result.is_empty());
    }

    #[test]
    fn test_outside_in_single() {
        let items = vec![42];
        let result = outside_in_sequence(&items);
        assert_eq!(result, vec![42]);
    }
}
