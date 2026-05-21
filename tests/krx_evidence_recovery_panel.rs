mod support;

use soma_zero::{MeasuredTargetDeltaV7Status, Sprint91KrxEvidenceRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn krx_evidence_recovery_panel_is_read_only_and_explicit() {
    let config = sprint::sprint91_config_from_example(
        "soma_control_tower_krx_evidence_recovery.toml",
        "krx-panel",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_control_tower_krx_evidence_recovery(&config)
        .expect("report");
    assert_eq!(report.primary_next_family, "KrxEvidence");
    assert_eq!(
        report.measured_delta_status,
        MeasuredTargetDeltaV7Status::SampleBackedOnly
    );
    assert_eq!(
        report.runtime_deferred_status,
        "RuntimeDeferredResearchOnlyPaperOnly"
    );
    for warning in [
        "no train button",
        "no runtime button",
        "no live button",
        "no order/account controls",
        "no browser execution",
    ] {
        assert!(
            report.warnings.iter().any(|value| value.contains(warning)),
            "missing {warning}"
        );
    }
}
