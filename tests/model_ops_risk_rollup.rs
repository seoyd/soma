mod common;
#[path = "support/sprint67_support.rs"]
mod sprint67_support;

use soma_zero::{ModelOpsRiskRollupStatus, ModelOpsRollupRunner};

#[test]
fn risk_rollup_maps_low_high_and_critical_models_conservatively() {
    let config =
        sprint67_support::rollup_config_from_example("soma_model_risk_rollup.toml", "risk-rollup");
    let report = ModelOpsRollupRunner::default()
        .run_model_risk_rollup(&config)
        .expect("run risk rollup");

    let low = report
        .items
        .iter()
        .find(|item| item.model_id == "ext-model-a" && item.model_version == "1.1.0")
        .expect("low risk item");
    assert_eq!(low.status, ModelOpsRiskRollupStatus::LowRiskResearch);

    let high = report
        .items
        .iter()
        .find(|item| item.model_id == "ext-model-a" && item.model_version == "1.0.0")
        .expect("high risk item");
    assert_eq!(high.status, ModelOpsRiskRollupStatus::HighRiskDiagnostic);

    let critical = report
        .items
        .iter()
        .find(|item| item.model_id == "ext-model-b" && item.model_version == "1.0.0")
        .expect("critical risk item");
    assert_eq!(
        critical.status,
        ModelOpsRiskRollupStatus::CriticalRiskDiagnostic
    );
    assert_eq!(critical.coverage_risk, "Critical");
    assert_eq!(critical.calibration_risk, "High");
    assert_eq!(critical.drift_risk, "High");
    assert_eq!(critical.artifact_completeness_risk, "High");
}
