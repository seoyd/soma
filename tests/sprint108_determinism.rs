mod support;

use support::sprint108_support::run_sprint108;

#[test]
fn sprint108_bundle_is_deterministic_and_summary_has_required_headings() {
    let mut left = run_sprint108(
        "soma_sprint108_safe_consolidation_patch_v2.toml",
        "sprint108-determinism-left",
    );
    let mut right = run_sprint108(
        "soma_sprint108_safe_consolidation_patch_v2.toml",
        "sprint108-determinism-right",
    );
    left.storage_report.output_dir = "<normalized>".to_string();
    right.storage_report.output_dir = "<normalized>".to_string();
    assert_eq!(
        left.to_json_string().expect("left json"),
        right.to_json_string().expect("right json")
    );
    for heading in [
        "## 1. Sprint summary",
        "## 25. Timeout cleanup verification",
        "## 29. Acceptance truth gate v9",
        "## 52. Next gstack sprint recommendation",
    ] {
        assert!(
            left.final_summary.contains(heading),
            "missing heading {heading}"
        );
    }
}
