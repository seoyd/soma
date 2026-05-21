mod support;

use soma_zero::{RemainingBlockerQueueV5Status, Sprint89CandleRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn remaining_blocker_queue_v5_keeps_order_and_primary_next_family_explicit() {
    let config = sprint::sprint89_config_from_example(
        "soma_remaining_blocker_queue_v5.toml",
        "remaining-queue-v5",
    );
    let report = Sprint89CandleRecoveryRunner::default()
        .run_remaining_blocker_queue_v5(&config)
        .expect("report");
    assert_eq!(
        report.queue_status,
        RemainingBlockerQueueV5Status::QueueAdvanced
    );
    assert_eq!(report.primary_next_family, "ExternalPrediction");
    assert_eq!(report.ordered_remaining_families.len(), 6);
    assert_eq!(
        report.completed_families,
        vec!["CandleExpansionOps".to_string()]
    );
}
