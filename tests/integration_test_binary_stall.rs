mod support;

use support::sprint111_support::run_sprint111;

#[test]
fn integration_test_binary_stall_identifies_high_fanout_families() {
    let bundle = run_sprint111(
        "soma_integration_test_binary_stall.toml",
        "integration-test-binary-stall",
    );
    assert_eq!(
        bundle.integration_test_binary_stall_report.stall_status,
        "IntegrationStallAttributed"
    );
    assert!(
        bundle
            .integration_test_binary_stall_report
            .high_fanout_integration_families
            .contains(&"fixture-fanout".to_string())
    );
    assert!(
        bundle
            .integration_test_binary_stall_report
            .already_retired_targets_excluded
    );
}
