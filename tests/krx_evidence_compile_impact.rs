mod support;

use soma_zero::{KrxEvidenceCompileImpactStatus, Sprint91KrxEvidenceRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn krx_compile_impact_defaults_to_sample_backed_and_records_blockers() {
    let config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_compile_impact.toml",
        "krx-compile-impact",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_compile_impact(&config)
        .expect("report");
    assert_eq!(
        report.impact_status,
        KrxEvidenceCompileImpactStatus::CompileImpactSampleBacked
    );
    assert_eq!(report.krx_family_delta, Some(1));
    assert!(report.blocked_targets.contains(&"KrxEvidence".to_string()));
}

#[test]
fn krx_compile_impact_requires_real_counts_for_measured_status() {
    let mut config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_compile_impact.toml",
        "krx-compile-impact-measured",
    );
    let path = sprint::write_support_json(
        "krx-compile-impact-measured",
        "krx_compile_impact_sample.json",
        &serde_json::json!({
            "target_count_before": 5,
            "target_count_after": 4,
            "compile_duration_before_ms": 1000,
            "compile_duration_after_ms": 900,
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
        .run_krx_evidence_compile_impact(&config)
        .expect("report");
    assert_eq!(
        report.impact_status,
        KrxEvidenceCompileImpactStatus::CompileImpactMeasured
    );
}

#[test]
fn krx_compile_impact_can_remain_still_blocked() {
    let mut config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_compile_impact.toml",
        "krx-compile-impact-blocked",
    );
    let path = sprint::write_support_json(
        "krx-compile-impact-blocked",
        "krx_compile_impact_sample.json",
        &serde_json::json!({
            "measured": false,
            "sample_backed": false,
            "blocked_targets": ["KrxEvidence"]
        }),
    );
    config
        .cargo_metadata_paths
        .retain(|value| !value.ends_with("krx_compile_impact_sample.json"));
    config.cargo_metadata_paths.push(path);
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_compile_impact(&config)
        .expect("report");
    assert_eq!(
        report.impact_status,
        KrxEvidenceCompileImpactStatus::CompileImpactStillBlocked
    );
}
