mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn sprint110_bundle_is_deterministic() {
    let mut left = run_sprint110(
        "soma_sprint110_safe_consolidation_patch_v4.toml",
        "sprint110-determinism-left",
    );
    let mut right = run_sprint110(
        "soma_sprint110_safe_consolidation_patch_v4.toml",
        "sprint110-determinism-right",
    );
    left.storage_report.output_dir = "<normalized>".to_string();
    right.storage_report.output_dir = "<normalized>".to_string();
    assert_eq!(
        left.to_json_string().expect("left json"),
        right.to_json_string().expect("right json")
    );
    assert_eq!(
        left.final_summary
            .lines()
            .filter(|line| line.starts_with("## "))
            .count(),
        59
    );
    for heading in [
        "## 5. Sprint 109 external validation reconciliation",
        "## 31. Timeout cleanup verification v3",
        "## 35. Acceptance truth gate v11",
        "## 59. Next gstack sprint recommendation",
    ] {
        assert!(
            left.final_summary.contains(heading),
            "missing summary heading {heading}"
        );
    }
}
