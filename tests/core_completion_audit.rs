mod common;

use soma_zero::{
    CoreCompletionAuditConfig, CoreCompletionAuditRunner, CoreCompletionRecommendation,
    CoreCompletionStatus, CoreSubsystem, SubsystemMaturity,
};

#[test]
fn core_completion_config_constructs_and_rejects_remote_paths() {
    let config = CoreCompletionAuditConfig::default();
    assert!(config.validate().is_ok());

    let remote = CoreCompletionAuditConfig::from_toml_str(
        r#"
audit_id = "remote"
output_root = "https://example.com/out"
"#,
    )
    .expect("config");
    assert!(!remote.validate_local_paths().is_empty());
}

#[test]
fn core_completion_report_loads_sample_artifacts_and_marks_expected_statuses() {
    let mut config = CoreCompletionAuditConfig::from_toml_path(&common::example_path(
        "soma_core_completion_audit.toml",
    ))
    .expect("config");
    config.output_root = common::sprint55_output_dir("core-completion")
        .display()
        .to_string();

    let (report, gaps) = CoreCompletionAuditRunner::default()
        .run(&config)
        .expect("audit");

    assert_eq!(
        report.core_completion_status,
        CoreCompletionStatus::CoreResearchOperatingSystemComplete
    );
    assert_eq!(
        report.final_recommendation,
        CoreCompletionRecommendation::CoreNeedsOutcomeLinkDepth
    );
    let runtime = report
        .maturity_matrix
        .rows
        .iter()
        .find(|row| row.subsystem == CoreSubsystem::RuntimeStateMachine)
        .expect("runtime row");
    assert_eq!(runtime.maturity, SubsystemMaturity::ResearchReady);
    let determinism = report
        .maturity_matrix
        .rows
        .iter()
        .find(|row| row.subsystem == CoreSubsystem::DeterminismGuard)
        .expect("determinism row");
    assert_eq!(determinism.maturity, SubsystemMaturity::ResearchReady);
    let risk = report
        .maturity_matrix
        .rows
        .iter()
        .find(|row| row.subsystem == CoreSubsystem::RiskInvariants)
        .expect("risk row");
    assert_eq!(risk.maturity, SubsystemMaturity::ResearchReady);
    let live = report
        .maturity_matrix
        .rows
        .iter()
        .find(|row| row.subsystem == CoreSubsystem::LiveSafety)
        .expect("live row");
    assert_eq!(live.maturity, SubsystemMaturity::ResearchReady);
    let tower = report
        .maturity_matrix
        .rows
        .iter()
        .find(|row| row.subsystem == CoreSubsystem::ControlTowerV1)
        .expect("tower row");
    assert_eq!(tower.maturity, SubsystemMaturity::PaperReady);
    let mamba = report
        .maturity_matrix
        .rows
        .iter()
        .find(|row| row.subsystem == CoreSubsystem::Mamba3Runtime)
        .expect("mamba row");
    assert!(matches!(
        mamba.maturity,
        SubsystemMaturity::Deferred | SubsystemMaturity::Forbidden
    ));
    assert!(
        gaps.gaps
            .iter()
            .any(|gap| gap.summary.contains("outcome links") || gap.gap_id.contains("outcome"))
    );
}

#[test]
fn core_completion_audit_is_deterministic() {
    let mut config = CoreCompletionAuditConfig::from_toml_path(&common::example_path(
        "soma_core_completion_audit.toml",
    ))
    .expect("config");
    config.output_root = common::sprint55_output_dir("core-completion-deterministic")
        .display()
        .to_string();

    let left = CoreCompletionAuditRunner::default()
        .run(&config)
        .expect("left")
        .0
        .to_json_string()
        .expect("left json");
    let right = CoreCompletionAuditRunner::default()
        .run(&config)
        .expect("right")
        .0
        .to_json_string()
        .expect("right json");
    assert_eq!(left, right);
}
