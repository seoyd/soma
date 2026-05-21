mod support;

use support::sprint99_support::run_sprint99;

#[test]
fn sprint99_bundle_is_deterministic_and_summary_has_required_headings() {
    let mut left = run_sprint99(
        "soma_sprint99_committee_quality_harden.toml",
        "sprint99-determinism-left",
    );
    let mut right = run_sprint99(
        "soma_sprint99_committee_quality_harden.toml",
        "sprint99-determinism-right",
    );
    left.storage_report.output_dir = "<normalized>".to_string();
    right.storage_report.output_dir = "<normalized>".to_string();
    assert_eq!(
        left.to_json_string().expect("left json"),
        right.to_json_string().expect("right json")
    );
    let summary = left.final_summary;
    for heading in [
        "## 1. Sprint summary",
        "## 5. Committee member proposal quality",
        "## 18. Paper-only decision replay",
        "## 22. Workspace acceptance truth closure",
        "## 24. Safety coverage preservation v15",
        "## 41. Next gstack sprint recommendation",
    ] {
        assert!(summary.contains(heading), "missing heading {heading}");
    }
}
