mod support;

use support::sprint101_support::run_sprint101;
use support::sprint102_support::run_sprint102;

#[test]
fn sprint102_bundle_is_deterministic_and_summary_has_required_headings() {
    let _ = run_sprint101(
        "soma_sprint101_investor_archetype_ingest.toml",
        "sprint102-determinism-sprint101-base",
    );
    let mut left = run_sprint102(
        "soma_sprint102_paper_rotation.toml",
        "sprint102-determinism-left",
    );
    let mut right = run_sprint102(
        "soma_sprint102_paper_rotation.toml",
        "sprint102-determinism-right",
    );
    left.storage_report.output_dir = "<normalized>".to_string();
    right.storage_report.output_dir = "<normalized>".to_string();
    assert_eq!(
        left.to_json_string().expect("left json"),
        right.to_json_string().expect("right json")
    );
    for heading in [
        "## 1. Sprint summary",
        "## 14. Paper-only member proposal run",
        "## 21. Risk Governor paper handoff",
        "## 33. Control Tower paper rotation panel",
        "## 48. Next gstack sprint recommendation",
    ] {
        assert!(
            left.final_summary.contains(heading),
            "missing heading {heading}"
        );
    }
}
