mod support;

use soma_zero::{Sprint87CompileGateRecoveryRunner, WorkspaceCompileBlockerDrilldownV3Status};
use support::sprint69_support as sprint;

#[test]
fn workspace_compile_blocker_drilldown_v3_lists_family_files_and_crates() {
    let config = sprint::sprint87_config_from_example(
        "soma_compile_blocker_drilldown_v3.toml",
        "compile-blocker-drilldown",
    );
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_compile_blocker_drilldown_v3(&config)
        .expect("drilldown");
    assert_eq!(
        report.report_status,
        WorkspaceCompileBlockerDrilldownV3Status::BlockersExplained
    );
    assert_eq!(report.blocker_family, "CandleExpansionOps");
    assert!(
        report
            .suspected_files
            .iter()
            .any(|file| file.ends_with("official_candle_expansion_runner.rs"))
    );
    assert!(
        report
            .suspected_crates
            .contains(&"candle_expansion_ops".to_string())
    );
    assert!(
        report
            .suspected_dev_dependencies
            .contains(&"serde_json".to_string())
    );
    assert_eq!(
        report.recommended_suite_target.as_deref(),
        Some("tests/candle_expansion_ops_suite.rs")
    );
}
