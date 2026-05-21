mod common;

use soma_zero::{
    CoreCompletionAuditConfig, CoreCompletionAuditRunner, CoreSubsystem, SubsystemMaturity,
};

#[test]
fn subsystem_matrix_counts_and_forbidden_surfaces_are_computed() {
    let mut config = CoreCompletionAuditConfig::from_toml_path(&common::example_path(
        "soma_core_completion_audit.toml",
    ))
    .expect("config");
    config.output_root = common::sprint55_output_dir("core-matrix")
        .display()
        .to_string();

    let report = CoreCompletionAuditRunner::default()
        .run(&config)
        .expect("audit")
        .0;
    let matrix = &report.maturity_matrix;

    assert!(matrix.research_ready_count >= 10);
    assert!(matrix.blocked_count >= 2);
    assert!(matrix.deferred_count >= 1);
    assert!(matrix.forbidden_count >= 2);
    assert_eq!(
        matrix
            .rows
            .iter()
            .find(|row| row.subsystem == CoreSubsystem::LiveTradingSurface)
            .expect("live trading row")
            .maturity,
        SubsystemMaturity::Forbidden
    );
    assert_eq!(
        matrix
            .rows
            .iter()
            .find(|row| row.subsystem == CoreSubsystem::KISOrderAccountSurface)
            .expect("kis order/account row")
            .maturity,
        SubsystemMaturity::Forbidden
    );

    let left = serde_json::to_string(matrix).expect("left json");
    let right = serde_json::to_string(matrix).expect("right json");
    assert_eq!(left, right);
}
