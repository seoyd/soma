mod support;

use soma_zero::{Sprint87CompileGateRecoveryRunner, WorkspaceFeatureUnificationStatus};
use support::sprint69_support as sprint;

#[test]
fn workspace_feature_unification_records_repeated_and_unsafe_candidates() {
    let config = sprint::sprint87_config_from_example(
        "soma_feature_unification_audit.toml",
        "feature-unification-audit",
    );
    let first = Sprint87CompileGateRecoveryRunner::default()
        .run_feature_unification_audit(&config)
        .expect("first");
    let second = Sprint87CompileGateRecoveryRunner::default()
        .run_feature_unification_audit(&config)
        .expect("second");
    assert!(
        first
            .repeated_feature_variants
            .contains(&"default+test-fixtures".to_string())
    );
    assert!(
        first
            .unsafe_unification_candidates
            .contains(&"do not gate workspace safety suite behind optional feature".to_string())
    );
    assert_eq!(
        first.report_status,
        WorkspaceFeatureUnificationStatus::FeatureUnificationReadyWithWarnings
    );
    assert_eq!(first, second);
}
