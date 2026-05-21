mod common;
#[path = "support/sprint69_support.rs"]
mod sprint69_support;

use std::fs;

#[test]
fn control_tower_trace_coverage_panel_and_fragments_are_static() {
    let bundle = sprint69_support::run_coverage(
        "soma_control_tower_trace_coverage.toml",
        "trace-coverage-panel",
    );
    assert_eq!(
        bundle.control_tower_trace_coverage_panel.coverage_status,
        "CoverageReady"
    );
    assert!(
        bundle
            .control_tower_trace_coverage_panel
            .missing_target_summary
            .contains("TargetsResolved")
    );
    assert_eq!(
        bundle
            .control_tower_trace_coverage_panel
            .per_model_scores
            .len(),
        4
    );

    let fragments = bundle.static_fragments.expect("static fragments");
    let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint69-tests")
        .join("trace-coverage-panel")
        .join("sprint69-control-tower-trace-coverage");
    for required in [
        "trace_coverage_overview",
        "missing_comparison_targets",
        "downgrade_evidence_audit",
        "snapshot_diff_integrity",
        "per_model_trace_scores",
    ] {
        let relative = fragments.get(required).expect("fragment path");
        let absolute = base.join(relative);
        let html = fs::read_to_string(absolute).expect("read fragment");
        for forbidden in [
            "<script",
            "<form",
            "KIS_APP_KEY",
            "KIS_APP_SECRET",
            "order",
            "account",
        ] {
            assert!(
                !html.contains(forbidden),
                "unexpected fragment content: {forbidden}"
            );
        }
    }
}
