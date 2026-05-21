mod support;

use support::sprint109_support::run_sprint109;

#[test]
fn sprint109_bundle_is_deterministic_and_summary_has_required_headings() {
    let mut left = run_sprint109(
        "soma_sprint109_safe_consolidation_patch_v3.toml",
        "sprint109-determinism-left",
    );
    let mut right = run_sprint109(
        "soma_sprint109_safe_consolidation_patch_v3.toml",
        "sprint109-determinism-right",
    );
    left.storage_report.output_dir = "<normalized>".to_string();
    right.storage_report.output_dir = "<normalized>".to_string();
    assert_eq!(
        left.to_json_string().expect("left json"),
        right.to_json_string().expect("right json")
    );
    for heading in [
        "## 1. Sprint summary",
        "## 27. Timeout cleanup verification v2",
        "## 31. Acceptance truth gate v10",
        "## 55. Next gstack sprint recommendation",
    ] {
        assert!(
            left.final_summary.contains(heading),
            "missing heading {heading}"
        );
    }
}
