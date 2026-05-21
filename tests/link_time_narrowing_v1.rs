mod support;

use serde_json::Value;
use support::sprint114_support::{read_fixture, run_sprint114};

#[test]
fn link_time_narrowing_is_deterministic_and_split() {
    let bundle = run_sprint114("soma_link_time_narrowing_v1.toml", "link-time-narrowing-v1");
    let expected: Value = read_fixture("sprint114_data/link_time_narrowing_expected.json");
    let report = bundle.link_time_narrowing_report_v1;
    assert_eq!(report.status, expected["status"].as_str().unwrap());
    assert!(
        report
            .link_heavy_target_candidates
            .iter()
            .any(|t| t.contains("control_tower"))
    );
    assert!(!report.observed_evidence.is_empty());
    assert!(!report.inferred_evidence.is_empty());
}
