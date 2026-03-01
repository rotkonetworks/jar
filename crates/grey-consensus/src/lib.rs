//! Safrole block production and GRANDPA finality (Sections 6, 19).
//!
//! Safrole is the block production mechanism combining:
//! - Ticket-based slot assignment (anonymous lottery)
//! - Fallback slot assignment (when tickets are insufficient)
//! - Epoch-based key rotation
//! - On-chain entropy accumulation

pub mod safrole;

/// Compute the epoch index for a given timeslot (eq 4.8).
pub fn epoch_index(timeslot: grey_types::Timeslot) -> u32 {
    timeslot / grey_types::constants::EPOCH_LENGTH
}

/// Compute the slot within an epoch for a given timeslot.
pub fn slot_in_epoch(timeslot: grey_types::Timeslot) -> u32 {
    timeslot % grey_types::constants::EPOCH_LENGTH
}

/// Check if a timeslot is within the ticket submission period (slot < Y).
pub fn is_ticket_submission_open(timeslot: grey_types::Timeslot) -> bool {
    slot_in_epoch(timeslot) < grey_types::constants::TICKET_SUBMISSION_END
}

/// Compute the rotation index for validator-core assignments.
pub fn rotation_index(timeslot: grey_types::Timeslot) -> u32 {
    timeslot / grey_types::constants::ROTATION_PERIOD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_index() {
        assert_eq!(epoch_index(0), 0);
        assert_eq!(epoch_index(599), 0);
        assert_eq!(epoch_index(600), 1);
        assert_eq!(epoch_index(1200), 2);
    }

    #[test]
    fn test_slot_in_epoch() {
        assert_eq!(slot_in_epoch(0), 0);
        assert_eq!(slot_in_epoch(599), 599);
        assert_eq!(slot_in_epoch(600), 0);
        assert_eq!(slot_in_epoch(601), 1);
    }

    #[test]
    fn test_ticket_submission() {
        assert!(is_ticket_submission_open(0));
        assert!(is_ticket_submission_open(499));
        assert!(!is_ticket_submission_open(500));
        assert!(!is_ticket_submission_open(599));
    }
}
