mod support;

use soma_zero::{
    KrxNonPrimaryProofReport, KrxNonPrimaryProofStatus, RealWorkspaceTimeoutAttributionConfig,
    Sprint93TimeoutAttributionRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_krx_non_primary_proof.toml", name)
}

#[test]
fn krx_non_primary_proof_matches_expected_fixture() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_krx_non_primary_proof(&config("krx-non-primary-proof"))
        .expect("report");
    let mut expected = harness::load_json_fixture::<KrxNonPrimaryProofReport>(
        sprint::example_path("sprint93_data/krx_non_primary_proof_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.proof_status,
        KrxNonPrimaryProofStatus::KrxProvenNonPrimary
    );
}
