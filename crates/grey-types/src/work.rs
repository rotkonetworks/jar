//! Work-related types: reports, digests, packages (Sections 11, 14).

use crate::{CoreIndex, Gas, Hash, ServiceId, Timeslot};
use std::collections::BTreeMap;

/// Work report R (eq 11.2).
#[derive(Clone, Debug)]
pub struct WorkReport {
    /// s: Availability specification.
    pub availability: AvailabilitySpec,

    /// c: Refinement context.
    pub context: RefinementContext,

    /// c (core): Core index on which the work was done.
    pub core_index: CoreIndex,

    /// a: Authorizer hash.
    pub authorizer_hash: Hash,

    /// t: Authorizer trace (opaque blob).
    pub authorizer_trace: Vec<u8>,

    /// l: Segment-root lookup dictionary.
    pub segment_root_lookup: BTreeMap<Hash, Hash>,

    /// g: Gas consumed during Is-Authorized invocation.
    pub auth_gas_used: Gas,

    /// d: Work digests.
    pub digests: Vec<WorkDigest>,
}

/// Availability specification Y (eq 11.5).
#[derive(Clone, Debug)]
pub struct AvailabilitySpec {
    /// p: Work-package hash.
    pub package_hash: Hash,

    /// l: Auditable work bundle length.
    pub bundle_length: u32,

    /// u: Erasure root.
    pub erasure_root: Hash,

    /// e: Segment root.
    pub segment_root: Hash,

    /// n: Segment count.
    pub segment_count: u32,
}

/// Refinement context C (eq 11.4).
#[derive(Clone, Debug)]
pub struct RefinementContext {
    /// a: Anchor header hash.
    pub anchor: Hash,

    /// s: Anchor posterior state root.
    pub state_root: Hash,

    /// b: Anchor accumulation output log super-peak.
    pub beefy_root: Hash,

    /// l: Lookup-anchor header hash.
    pub lookup_anchor: Hash,

    /// t: Lookup-anchor timeslot.
    pub lookup_anchor_timeslot: Timeslot,

    /// p: Prerequisite work-package hashes.
    pub prerequisites: Vec<Hash>,
}

/// Work digest D (eq 11.6).
#[derive(Clone, Debug)]
pub struct WorkDigest {
    /// s: Service index.
    pub service_id: ServiceId,

    /// c: Code hash of the service at time of reporting.
    pub code_hash: Hash,

    /// y: Hash of the payload in the work item.
    pub payload_hash: Hash,

    /// g: Gas limit for accumulation of this item.
    pub gas_limit: Gas,

    /// l: Work result — either output blob or error.
    pub result: WorkResult,

    /// u: Actual gas used during refinement.
    pub gas_used: Gas,

    /// i: Number of segments imported.
    pub imports_count: u32,

    /// x: Number of extrinsics used.
    pub extrinsics_count: u32,

    /// z: Total size of extrinsics in octets.
    pub extrinsics_size: u32,

    /// e: Number of segments exported.
    pub exports_count: u32,
}

/// Work result: either a successful output blob or an error (eq 11.7).
#[derive(Clone, Debug)]
pub enum WorkResult {
    /// Successful refinement output.
    Ok(Vec<u8>),
    /// Out of gas (∞).
    OutOfGas,
    /// Panic (☇).
    Panic,
    /// Invalid export count (⊚).
    InvalidExportCount,
    /// Digest too large (⊖).
    DigestTooLarge,
    /// Service code not available (BAD).
    CodeNotAvailable,
    /// Service code too large (BIG).
    CodeTooLarge,
}

/// Work package P (eq 14.2).
#[derive(Clone, Debug)]
pub struct WorkPackage {
    /// h: Authorization hash.
    pub authorization_hash: Hash,

    /// u: Authorization code (blob).
    pub authorization_code: Vec<u8>,

    /// c: Authorization configuration.
    pub authorization_config: Vec<u8>,

    /// j: Prerequisite work-package hashes.
    pub prerequisites: Vec<Hash>,

    /// f: Authorization token.
    pub auth_token: Vec<u8>,

    /// w: Work items.
    pub items: Vec<WorkItem>,
}

/// Work item W (eq 14.3).
#[derive(Clone, Debug)]
pub struct WorkItem {
    /// s: Service index.
    pub service_id: ServiceId,

    /// c: Code hash.
    pub code_hash: Hash,

    /// g: Gas limit for refinement.
    pub gas_limit: Gas,

    /// a: Gas limit for accumulation.
    pub accumulate_gas_limit: Gas,

    /// e: Number of exports.
    pub exports_count: u16,

    /// y: Payload.
    pub payload: Vec<u8>,

    /// i: Import segments.
    pub imports: Vec<ImportSegment>,

    /// x: Extrinsics (hash, index) pairs.
    pub extrinsics: Vec<(Hash, u32)>,
}

/// An import segment reference.
#[derive(Clone, Debug)]
pub struct ImportSegment {
    pub hash: Hash,
    pub index: u32,
}
