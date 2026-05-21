mod support;

use support::sprint106_support::run_sprint106;

#[test]
fn integration_target_cost_ranking_is_deterministic_and_lists_missing_timing() {
    let bundle = run_sprint106(
        "soma_integration_target_cost_ranking.toml",
        "integration_target_cost_ranking",
    );
    let report = bundle.integration_target_cost_ranking_report;
    assert!(!report.ranked_targets.is_empty());
    assert!(!report.targets_missing_timing.is_empty());
    assert!(
        report
            .ranked_targets
            .windows(2)
            .all(|window| window[0].score >= window[1].score)
    );
}
