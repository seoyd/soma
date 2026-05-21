mod support;

use soma_zero::TimeoutCleanupConsistencyReportV1;
use support::sprint116_support::{read_fixture, run_sprint116};

#[test]
fn timeout_cleanup_consistency_v1_matches_expected() {
    let bundle = run_sprint116(
        "soma_timeout_cleanup_consistency_v1.toml",
        "timeout-cleanup-consistency-v1",
    );
    let expected: TimeoutCleanupConsistencyReportV1 =
        read_fixture("sprint116_data/timeout_cleanup_consistency_expected.json");
    assert_eq!(bundle.timeout_cleanup_consistency_report_v1, expected);
    assert!(
        !bundle
            .timeout_cleanup_consistency_report_v1
            .timeout_cleanup_is_pass
    );
}
