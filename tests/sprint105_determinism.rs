mod support;

use support::sprint105_support::run_sprint105;

#[test]
fn sprint105_bundle_is_deterministic_and_summary_has_required_headings() {
    let mut left = run_sprint105(
        "soma_sprint105_verification_patch_close.toml",
        "sprint105-determinism-left",
    );
    let mut right = run_sprint105(
        "soma_sprint105_verification_patch_close.toml",
        "sprint105-determinism-right",
    );
    left.storage_report.output_dir = "<normalized>".to_string();
    right.storage_report.output_dir = "<normalized>".to_string();
    assert_eq!(
        left.to_json_string().expect("left json"),
        right.to_json_string().expect("right json")
    );
    for heading in [
        "## 1. Sprint summary",
        "## 13. Final verification gate v2",
        "## 31. Control Tower paper lifecycle closure panel",
        "## 45. Next gstack sprint recommendation",
    ] {
        assert!(
            left.final_summary.contains(heading),
            "missing heading {heading}"
        );
    }
}
