mod support;

use serde_json::Value;
use support::sprint114_support::{read_fixture, run_sprint114};

#[test]
fn integration_fanout_narrowing_preserves_observed_vs_inferred() {
    let bundle = run_sprint114(
        "soma_integration_fanout_narrowing_v1.toml",
        "integration-fanout-narrowing-v1",
    );
    let expected: Value = read_fixture("sprint114_data/integration_fanout_narrowing_expected.json");
    let report = bundle.integration_fanout_narrowing_report_v1;
    assert_eq!(report.status, expected["status"].as_str().unwrap());
    assert!(!report.observed_evidence.is_empty());
    assert!(!report.inferred_evidence.is_empty());
    assert_ne!(report.observed_evidence, report.inferred_evidence);
}
