mod support;

use soma_zero::{KrxEvidenceAuthBoundaryPreservationStatus, Sprint91KrxEvidenceRecoveryRunner};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn krx_evidence_auth_boundary_preserves_missing_auth_behavior() {
    let config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_auth_boundary_preservation.toml",
        "krx-auth-boundary",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_auth_boundary_preservation(&config)
        .expect("report");
    assert_eq!(
        report.auth_status,
        KrxEvidenceAuthBoundaryPreservationStatus::AuthBoundaryPreserved
    );
    assert!(report.missing_auth_behavior_preserved);
    assert!(report.env_var_name_only_preserved);
    assert!(report.secret_value_not_rendered);
    assert!(report.operator_action_preserved);
    assert!(report.auth_dry_run_preserved);
    assert!(report.live_collection_disabled_by_default);
    harness::assert_no_secret_like_values(&serde_json::to_string(&report).expect("json"));
}
