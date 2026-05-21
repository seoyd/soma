mod support;

use soma_zero::{SevenBlockerQueueProgressStatusV5, Sprint89CandleRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn queue_progress_advances_to_external_prediction_when_candle_is_reduced() {
    let config = sprint::sprint89_config_from_example(
        "soma_seven_blocker_queue_progress_v5.toml",
        "queue-progress-v5",
    );
    let report = Sprint89CandleRecoveryRunner::default()
        .run_seven_blocker_queue_progress_v5(&config)
        .expect("report");
    assert_eq!(
        report.queue_status,
        SevenBlockerQueueProgressStatusV5::QueueAdvanced
    );
    assert_eq!(report.primary_next_family, "ExternalPrediction");
    assert_eq!(report.current_queue.len(), 6);
    assert_eq!(
        report.completed_families,
        vec!["CandleExpansionOps".to_string()]
    );
}
