mod support;

use soma_zero::{DevDependencyFanoutStatus, Sprint87CompileGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn dev_dependency_fanout_detects_heavy_and_repeated_candidates() {
    let config = sprint::sprint87_config_from_example(
        "soma_dev_dependency_fanout.toml",
        "dev-dependency-fanout",
    );
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_dev_dependency_fanout(&config)
        .expect("fanout");
    assert!(report.dev_dependencies.contains(&"serde_json".to_string()));
    assert!(
        report
            .heavy_dev_dependency_candidates
            .contains(&"insta".to_string())
    );
    assert!(
        report
            .repeated_compile_candidates
            .contains(&"external_prediction".to_string())
    );
    assert_eq!(
        report.dependency_status,
        DevDependencyFanoutStatus::HeavyFanoutDetected
    );
}
