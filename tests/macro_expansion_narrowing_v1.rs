mod support;

use serde_json::Value;
use support::sprint114_support::{read_fixture, run_sprint114};

#[test]
fn macro_expansion_narrowing_is_deterministic_and_split() {
    let bundle = run_sprint114(
        "soma_macro_expansion_narrowing_v1.toml",
        "macro-expansion-narrowing-v1",
    );
    let expected: Value = read_fixture("sprint114_data/macro_expansion_narrowing_expected.json");
    let report = bundle.macro_expansion_narrowing_report_v1;
    assert_eq!(report.status, expected["status"].as_str().unwrap());
    assert!(
        report
            .macro_heavy_target_candidates
            .contains(&"tests/workspace_timeout_root_cause.rs".to_string())
    );
    assert!(!report.observed_evidence.is_empty());
    assert!(!report.inferred_evidence.is_empty());
}
