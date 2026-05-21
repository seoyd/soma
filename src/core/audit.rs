use std::collections::BTreeMap;

use super::{AuditEvent, AuditEventType, ReasonCode};

pub fn stable_hash(input: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

pub fn build_audit_event(
    timestamp_ms: u64,
    event_type: AuditEventType,
    input_material: &str,
    decision_summary: impl Into<String>,
    reason_codes: Vec<ReasonCode>,
    numeric_snapshot: BTreeMap<String, f64>,
) -> AuditEvent {
    AuditEvent {
        timestamp_ms,
        event_type,
        input_hash: stable_hash(input_material),
        decision_summary: decision_summary.into(),
        reason_codes,
        numeric_snapshot,
    }
}
