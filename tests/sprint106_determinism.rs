mod support;

use support::sprint106_support::run_sprint106;

#[test]
fn sprint106_bundle_is_deterministic_and_summary_has_required_headings() {
    let mut left = run_sprint106(
        "soma_sprint106_workspace_acceptance_recover.toml",
        "sprint106-determinism-left",
    );
    let mut right = run_sprint106(
        "soma_sprint106_workspace_acceptance_recover.toml",
        "sprint106-determinism-right",
    );
    left.storage_report.output_dir = "<normalized>".to_string();
    right.storage_report.output_dir = "<normalized>".to_string();
    assert_eq!(
        left.to_json_string().expect("left json"),
        right.to_json_string().expect("right json")
    );
    for heading in [
        "## 1. Sprint summary",
        "## 24. Acceptance truth gate v7",
        "## 29. Control Tower workspace acceptance recovery panel v7",
        "## 43. Next gstack sprint recommendation",
    ] {
        assert!(
            left.final_summary.contains(heading),
            "missing heading {heading}"
        );
    }
}
