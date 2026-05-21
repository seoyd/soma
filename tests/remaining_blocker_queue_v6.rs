mod support;

use soma_zero::{RemainingBlockerQueueV6Status, Sprint90ExternalPredictionRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn remaining_blocker_queue_v6_keeps_order_and_primary_next_family_explicit() {
    let config = sprint::sprint90_config_from_example(
        "soma_remaining_blocker_queue_v6.toml",
        "remaining-queue-v6",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_remaining_blocker_queue_v6(&config)
        .expect("report");
    assert_eq!(
        report.queue_status,
        RemainingBlockerQueueV6Status::QueueAdvanced
    );
    assert_eq!(report.primary_next_family, "KrxEvidence");
    assert_eq!(report.ordered_remaining_families.len(), 5);
    assert_eq!(
        report.completed_families,
        vec!["ExternalPrediction".to_string()]
    );
}
