mod support;

use soma_zero::NextestAvailabilityReportV1;
use support::sprint112_support::{read_fixture, run_sprint112, sprint112_config_from_example};

#[test]
fn nextest_reports_cover_available_unavailable_partition_and_slow_targets() {
    let bundle = run_sprint112("soma_nextest_availability_v1.toml", "nextest-availability");
    let expected: NextestAvailabilityReportV1 =
        read_fixture("sprint112_data/nextest_availability_expected.json");
    assert_eq!(bundle.nextest_availability_report_v1, expected);
    assert_eq!(
        bundle.nextest_availability_report_v1.availability_status,
        "NextestAvailable"
    );
    assert_eq!(
        bundle.nextest_pilot_execution_report_v1.execution_status,
        "NextestNotRun"
    );
    assert!(
        bundle
            .nextest_target_partition_report_v1
            .safety_partition_present
    );
    assert!(
        bundle
            .nextest_target_partition_report_v1
            .sentinel_partition_isolated
    );
    assert!(
        !bundle
            .workspace_diagnostic_evidence_matrix_v1
            .supports_acceptance
    );
    assert!(
        !bundle
            .nextest_slow_target_attribution_report_v1
            .slow_tests
            .is_empty()
    );

    let mut config =
        sprint112_config_from_example("soma_nextest_availability_v1.toml", "nextest-unavailable");
    config.nextest_paths = None;
    let unavailable = soma_zero::WorkspaceDiagnosticPilotV1Runner::default()
        .run(&config)
        .expect("run");
    assert!(matches!(
        unavailable
            .nextest_availability_report_v1
            .availability_status
            .as_str(),
        "NextestProbeNotRun" | "NextestUnavailable"
    ));
}
