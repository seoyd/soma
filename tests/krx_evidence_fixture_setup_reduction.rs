mod support;

use soma_zero::{KrxEvidenceFixtureSetupReductionStatus, Sprint91KrxEvidenceRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn krx_evidence_fixture_setup_reduction_counts_expected_dedup() {
    let config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_fixture_setup_reduction.toml",
        "krx-fixture-setup",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_fixture_setup_reduction(&config)
        .expect("report");
    assert_eq!(
        report.reduction_status,
        KrxEvidenceFixtureSetupReductionStatus::FixtureSetupReduced
    );
    assert_eq!(report.duplicate_json_loads_removed, 3);
    assert_eq!(report.duplicate_csv_loads_removed, 1);
    assert_eq!(report.duplicate_toml_loads_removed, 1);
    assert_eq!(report.duplicate_output_dirs_removed, 2);
    assert_eq!(report.duplicate_auth_fixtures_removed, 1);
    assert_eq!(report.duplicate_endpoint_template_fixtures_removed, 1);
    assert!(report.shared_fixture_harness_used);
    assert!(report.deterministic_output_preserved);
}
