//! Validator activity statistics sub-transition (Section 13, eq 13.3-13.5).
//!
//! Updates per-validator performance records based on block activity.

use grey_types::config::Config;
use grey_types::header::Extrinsic;
use grey_types::state::{ValidatorRecord, ValidatorStatistics};
use grey_types::Timeslot;

/// Apply the statistics sub-transition.
///
/// Updates the validator statistics based on the block's author and extrinsic content.
/// On epoch boundaries, rotates current stats to last and resets current.
pub fn update_statistics(
    config: &Config,
    stats: &mut ValidatorStatistics,
    prior_timeslot: Timeslot,
    new_timeslot: Timeslot,
    author_index: u16,
    extrinsic: &Extrinsic,
) {
    let old_epoch = prior_timeslot / config.epoch_length;
    let new_epoch = new_timeslot / config.epoch_length;

    let num_validators = stats.current.len();

    // Epoch transition: rotate statistics (eq 13.4)
    if new_epoch > old_epoch {
        stats.last = stats.current.clone();
        stats.current = vec![ValidatorRecord::default(); num_validators];
    }

    let author = author_index as usize;
    if author < stats.current.len() {
        // Block author: increment blocks_produced (eq 13.5)
        stats.current[author].blocks_produced += 1;

        // Tickets introduced
        stats.current[author].tickets_introduced += extrinsic.tickets.len() as u32;

        // Preimages introduced
        stats.current[author].preimages_introduced += extrinsic.preimages.len() as u32;

        // Preimage total bytes
        let preimage_bytes: u64 = extrinsic
            .preimages
            .iter()
            .map(|(_, data)| data.len() as u64)
            .sum();
        stats.current[author].preimage_bytes += preimage_bytes;
    }

    // Assurances: each validator that submitted an assurance
    for assurance in &extrinsic.assurances {
        let idx = assurance.validator_index as usize;
        if idx < stats.current.len() {
            stats.current[idx].assurances_made += 1;
        }
    }

    // Guarantees: each validator that guaranteed a report
    for guarantee in &extrinsic.guarantees {
        for (validator_idx, _sig) in &guarantee.credentials {
            let idx = *validator_idx as usize;
            if idx < stats.current.len() {
                stats.current[idx].reports_guaranteed += 1;
            }
        }
    }
}
