mod support;

use serde_json::Value;
use support::sprint118_support::run_sprint118;

#[test]
fn sprint118_bundle_is_deterministic() {
    let mut first = run_sprint118(
        "soma_sprint118_timeout_reduction_queue.toml",
        "sprint118-determinism-a",
    );
    let mut second = run_sprint118(
        "soma_sprint118_timeout_reduction_queue.toml",
        "sprint118-determinism-b",
    );
    assert_eq!(first.storage_report.file_count, 51);
    first.storage_report.output_dir = "<normalized>".to_string();
    second.storage_report.output_dir = "<normalized>".to_string();
    let mut first_value: Value = serde_json::to_value(first).expect("first json");
    let mut second_value: Value = serde_json::to_value(second).expect("second json");
    first_value["storage_report"]["output_dir"] = Value::String("<normalized>".to_string());
    second_value["storage_report"]["output_dir"] = Value::String("<normalized>".to_string());
    assert_eq!(first_value, second_value);
    let summary = first_value["final_summary"].as_str().expect("summary");
    assert!(summary.contains("## 1. Sprint summary"));
    assert!(summary.contains("## 44. Acceptance truth gate v19"));
    assert!(summary.contains("## 67. Next gstack sprint recommendation"));
    assert_eq!(summary.matches("\n## ").count() + 1, 67);
}
