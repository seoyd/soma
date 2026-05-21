mod support;

use soma_zero::{MeasuredTargetDeltaV7Status, Sprint91KrxEvidenceRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn measured_target_delta_defaults_to_sample_backed() {
    let config = sprint::sprint91_config_from_example(
        "soma_measured_target_delta_v7.toml",
        "krx-measured-delta-default",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_measured_target_delta_v7(&config)
        .expect("report");
    assert_eq!(
        report.delta_status,
        MeasuredTargetDeltaV7Status::SampleBackedOnly
    );
    assert_eq!(report.krx_family_delta, Some(1));
}

#[test]
fn measured_target_delta_requires_real_counts_for_measured_state() {
    let mut config = sprint::sprint91_config_from_example(
        "soma_measured_target_delta_v7.toml",
        "krx-measured-delta-measured",
    );
    let path = sprint::write_support_json(
        "krx-measured-delta-measured",
        "krx_compile_impact_sample.json",
        &serde_json::json!({
            "target_count_before": 5,
            "target_count_after": 4,
            "measured": true,
            "sample_backed": false,
            "blocked_targets": ["DashboardRenderer"]
        }),
    );
    config
        .cargo_metadata_paths
        .retain(|value| !value.ends_with("krx_compile_impact_sample.json"));
    config.cargo_metadata_paths.push(path);
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_measured_target_delta_v7(&config)
        .expect("report");
    assert_eq!(
        report.delta_status,
        MeasuredTargetDeltaV7Status::MeasuredTargetDeltaReadyWithWarnings
    );
    assert!(report.measured);
}
