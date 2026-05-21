mod support;

use support::sprint100_support::run_sprint100;
use support::sprint101_support::run_sprint101;

#[test]
fn sprint101_bundle_is_deterministic_and_summary_has_required_headings() {
    let _ = run_sprint100(
        "soma_sprint100_committee_closure.toml",
        "sprint101-determinism-sprint100-base",
    );
    let mut left = run_sprint101(
        "soma_sprint101_investor_archetype_ingest.toml",
        "sprint101-determinism-left",
    );
    let mut right = run_sprint101(
        "soma_sprint101_investor_archetype_ingest.toml",
        "sprint101-determinism-right",
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
        "## 14. 18 investor candidate registry",
        "## 30. Paper-only roster expansion gate",
        "## 34. Control Tower investor archetype panel",
        "## 46. Next gstack sprint recommendation",
    ] {
        assert!(summary.contains(heading), "missing heading {heading}");
    }
}
