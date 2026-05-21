mod support;

use soma_zero::{MeasuredTargetDeltaV6Status, Sprint90ExternalPredictionRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn external_prediction_recovery_panel_is_read_only_and_explicit() {
    let config = sprint::sprint90_config_from_example(
        "soma_control_tower_external_prediction_recovery.toml",
        "external-panel",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_control_tower_external_prediction_recovery(&config)
        .expect("report");
    assert_eq!(report.primary_next_family, "KrxEvidence");
    assert_eq!(
        report.measured_delta_status,
        MeasuredTargetDeltaV6Status::SampleBackedOnly
    );
    assert_eq!(
        report.runtime_deferred_status,
        "RuntimeDeferredResearchOnlyPaperOnly"
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|value| value.contains("no train button"))
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|value| value.contains("no runtime button"))
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|value| value.contains("no live button"))
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|value| value.contains("no order/account controls"))
    );
}
