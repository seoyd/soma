mod common;

use soma_zero::{
    CoreCheckConfig, CoreCheckRunner, CoreCheckedBenchmarkConfig, CoreCheckedBenchmarkRunner,
    CoreCheckedBenchmarkStatus, CoreReadinessStatus, RuntimeMode, build_core_check_gate_result,
};

#[test]
fn core_check_gate_allows_allowed_ready_status() {
    let report = CoreCheckRunner::default()
        .run(&CoreCheckConfig {
            official_evidence_ready: true,
            sequence_dataset_ready: true,
            ..CoreCheckConfig::default()
        })
        .expect("run core-check");

    let gate = build_core_check_gate_result(
        Some(&report),
        true,
        &[CoreReadinessStatus::ReadyForExternalModelPrototype],
    );

    assert!(gate.core_check_ran);
    assert!(gate.passed);
    assert_eq!(
        gate.core_status,
        Some(CoreReadinessStatus::ReadyForExternalModelPrototype)
    );
}

#[test]
fn core_check_gate_blocks_disallowed_status() {
    let mut report = CoreCheckRunner::default()
        .run(&CoreCheckConfig::default())
        .expect("run core-check");
    report.final_status = CoreReadinessStatus::NotReadyDueToRiskInvariantFailure;
    report.blockers = vec!["risk invariants failed".to_string()];

    let gate = build_core_check_gate_result(
        Some(&report),
        true,
        &[CoreReadinessStatus::ReadyForExternalModelPrototype],
    );

    assert!(!gate.passed);
    assert_eq!(
        gate.failed_reasons,
        vec!["risk invariants failed".to_string()]
    );
}

#[test]
fn failed_core_check_blocks_benchmark_outside_diagnostics_only() {
    let report = CoreCheckedBenchmarkRunner::default()
        .run(&CoreCheckedBenchmarkConfig {
            benchmark_id: "core-gate-blocked".to_string(),
            core_check_config: Some(CoreCheckConfig {
                runtime_mode: RuntimeMode::Research,
                ..CoreCheckConfig::default()
            }),
            allowed_core_statuses: vec![CoreReadinessStatus::NotReadyDueToContractDrift],
            output_root: common::output_dir("core-gate-blocked")
                .display()
                .to_string(),
            ..CoreCheckedBenchmarkConfig::default()
        })
        .expect("run blocked benchmark");

    assert_eq!(report.final_status, CoreCheckedBenchmarkStatus::CoreBlocked);
    assert!(!report.core_check_gate.passed);
}

#[test]
fn diagnostics_only_mode_emits_report_without_core_pass() {
    let report = CoreCheckedBenchmarkRunner::default()
        .run(&CoreCheckedBenchmarkConfig {
            benchmark_id: "core-gate-diagnostics".to_string(),
            core_check_config: Some(CoreCheckConfig {
                runtime_mode: RuntimeMode::DiagnosticsOnly,
                ..CoreCheckConfig::default()
            }),
            allowed_core_statuses: vec![CoreReadinessStatus::NotReadyDueToContractDrift],
            output_root: common::output_dir("core-gate-diagnostics")
                .display()
                .to_string(),
            ..CoreCheckedBenchmarkConfig::default()
        })
        .expect("run diagnostics benchmark");

    assert!(!report.core_check_gate.passed);
    assert_ne!(report.final_status, CoreCheckedBenchmarkStatus::CoreBlocked);
}
