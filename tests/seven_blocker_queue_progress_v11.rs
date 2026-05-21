mod support;

use soma_zero::{
    SevenBlockerQueueProgressReportV11, SevenBlockerQueueProgressStatusV11,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn queue_progress_advances_to_baseline_signal() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_seven_blocker_queue_progress_v11(&sprint::sprint95_config_from_example(
            "soma_seven_blocker_queue_progress_v11.toml",
            "seven-blocker-queue-progress-v11",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<SevenBlockerQueueProgressReportV11>(
        sprint::example_path("sprint95_data/queue_progress_v11_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.queue_status,
        SevenBlockerQueueProgressStatusV11::QueueAdvanced
    );
    assert_eq!(report.primary_next_family, "BaselineSignal");
}
