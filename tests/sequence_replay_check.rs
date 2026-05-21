mod common;
#[path = "support/sprint62_support.rs"]
mod sprint62_support;

use soma_zero::{SequenceDatasetExportRunner, SequenceReplayStatus};

#[test]
fn replay_check_is_stable() {
    let config_path = sprint62_support::example_path("soma_sequence_dataset_replay_check.toml");
    let report = SequenceDatasetExportRunner::default()
        .run_replay_check(&config_path)
        .expect("run replay");
    assert_eq!(report.replay_status, SequenceReplayStatus::ReplayStable);
    assert!(report.fingerprints_match);
    assert!(report.row_order_match);
    assert!(report.split_match);
}
