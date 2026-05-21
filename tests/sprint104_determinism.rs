mod support;

use support::sprint104_support::run_sprint104;

#[test]
fn sprint104_bundle_is_deterministic_and_summary_has_required_headings() {
    let mut left = run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "sprint104-determinism-left",
    );
    let mut right = run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "sprint104-determinism-right",
    );
    left.storage_report.output_dir = "<normalized>".to_string();
    right.storage_report.output_dir = "<normalized>".to_string();
    assert_eq!(
        left.to_json_string().expect("left json"),
        right.to_json_string().expect("right json")
    );
    for heading in [
        "## 1. Sprint summary",
        "## 17. Final verification gate",
        "## 28. Control Tower paper candidate lifecycle panel",
        "## 43. Next gstack sprint recommendation",
    ] {
        assert!(
            left.final_summary.contains(heading),
            "missing heading {heading}"
        );
    }
}
