mod support;

use soma_zero::{
    SevenBlockerQueueProgressReportV10, SevenBlockerQueueProgressStatusV10,
    Sprint94DashboardRendererRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn queue_progress_matches_expected_fixture() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_seven_blocker_queue_progress_v10(&sprint::sprint94_config_from_example(
            "soma_seven_blocker_queue_progress_v10.toml",
            "queue-progress-v10",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<SevenBlockerQueueProgressReportV10>(
        sprint::example_path("sprint94_data/queue_progress_v10_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.queue_status,
        SevenBlockerQueueProgressStatusV10::QueueAdvanced
    );
}
