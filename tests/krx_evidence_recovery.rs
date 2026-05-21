mod support;

use soma_zero::{KrxEvidenceRecoveryStatus, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn krx_evidence_recovery_preserves_market_data_boundary() {
    let config =
        sprint::sprint88_config_from_example("soma_krx_evidence_recovery.toml", "krx-recovery");
    let report = Sprint88SevenBlockerRecoveryRunner::default()
        .run_krx_evidence_recovery(&config)
        .expect("report");
    assert!(report.missing_auth_covered);
    assert!(report.endpoint_template_covered);
    assert!(report.official_reference_fallback_covered);
    assert!(report.market_data_only_covered);
    assert!(report.no_order_account_path_covered);
    assert!(report.deterministic_status_covered);
    assert_eq!(
        report.recovery_status,
        KrxEvidenceRecoveryStatus::KrxEvidenceReduced
    );
}
