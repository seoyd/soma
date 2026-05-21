mod support;

use soma_zero::SccacheAvailabilityReportV1;
use support::sprint112_support::{read_fixture, run_sprint112, sprint112_config_from_example};

#[test]
fn sccache_availability_reports_cover_available_and_unavailable() {
    let bundle = run_sprint112("soma_sccache_availability_v1.toml", "sccache-availability");
    let expected: SccacheAvailabilityReportV1 =
        read_fixture("sprint112_data/sccache_availability_expected.json");
    assert_eq!(bundle.sccache_availability_report_v1, expected);
    let mut config =
        sprint112_config_from_example("soma_sccache_availability_v1.toml", "sccache-unavailable");
    config.sccache_paths = None;
    let unavailable = soma_zero::WorkspaceDiagnosticPilotV1Runner::default()
        .run(&config)
        .expect("run");
    assert!(matches!(
        unavailable
            .sccache_availability_report_v1
            .availability_status
            .as_str(),
        "SccacheProbeNotRun" | "SccacheUnavailable"
    ));
}
