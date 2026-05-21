mod support;

use soma_zero::{SevenBlockerQueueProgressStatusV6, Sprint90ExternalPredictionRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn queue_progress_advances_to_krx_evidence_when_external_prediction_is_reduced() {
    let config = sprint::sprint90_config_from_example(
        "soma_seven_blocker_queue_progress_v6.toml",
        "queue-progress-v6",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_seven_blocker_queue_progress_v6(&config)
        .expect("report");
    assert_eq!(
        report.queue_status,
        SevenBlockerQueueProgressStatusV6::QueueAdvanced
    );
    assert_eq!(report.primary_next_family, "KrxEvidence");
    assert_eq!(report.current_queue.len(), 5);
    assert_eq!(
        report.completed_families,
        vec!["ExternalPrediction".to_string()]
    );
}
