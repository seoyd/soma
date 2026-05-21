mod support;

use soma_zero::{
    DashboardRendererSecretRedactionPreservationReport, DashboardRendererSecretRedactionStatus,
    Sprint94DashboardRendererRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn secret_redaction_matches_expected_fixture() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_secret_redaction_preservation(
            &sprint::sprint94_config_from_example(
                "soma_dashboard_renderer_secret_redaction_preservation.toml",
                "dashboard-renderer-secret-redaction",
            ),
        )
        .expect("report");
    let mut expected =
        harness::load_json_fixture::<DashboardRendererSecretRedactionPreservationReport>(
            sprint::example_path("sprint94_data/dashboard_renderer_secret_redaction_expected.json"),
        );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.redaction_status,
        DashboardRendererSecretRedactionStatus::SecretRedactionPreserved
    );
}
