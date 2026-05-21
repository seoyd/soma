mod support;

use support::sprint100_support::run_sprint100;

#[test]
fn sprint100_bundle_is_deterministic_and_summary_has_required_headings() {
    let mut left = run_sprint100(
        "soma_sprint100_committee_closure.toml",
        "sprint100-determinism-left",
    );
    let mut right = run_sprint100(
        "soma_sprint100_committee_closure.toml",
        "sprint100-determinism-right",
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
        "## 5. Proposal quality warning closure",
        "## 16. Chairman rulebook approval gate",
        "## 27. Committee paper readiness gate",
        "## 32. Control Tower AI committee closure panel",
        "## 47. Next gstack sprint recommendation",
    ] {
        assert!(summary.contains(heading), "missing heading {heading}");
    }
}
