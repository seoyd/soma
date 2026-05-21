mod support;

use soma_zero::{ResidualWorkspaceBlockerDrilldownV2Status, Sprint86ResidualGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn residual_blocker_drilldown_v2_identifies_family_and_recommended_suite() {
    let config = sprint::sprint86_config_from_example(
        "soma_residual_blocker_drilldown_v2.toml",
        "residual-blocker-drilldown-v2-test",
    );
    let report = Sprint86ResidualGateRecoveryRunner::default()
        .run_residual_blocker_drilldown_v2(&config)
        .expect("drilldown");
    assert_eq!(
        report.report_status,
        ResidualWorkspaceBlockerDrilldownV2Status::BlockersExplained
    );
    assert_eq!(
        report.recommended_suite_target.as_deref(),
        Some("tests/official_expansion_suite.rs")
    );
}
