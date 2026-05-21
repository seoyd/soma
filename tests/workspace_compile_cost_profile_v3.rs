mod support;

use serde_json::Value;
use support::sprint106_support::run_sprint106;

#[test]
fn compile_cost_profile_records_counts_and_cost_centers() {
    let bundle = run_sprint106(
        "soma_workspace_compile_cost_profile_v3.toml",
        "workspace_compile_cost_profile_v3",
    );
    let report = bundle.workspace_compile_cost_profile_v3;
    assert!(report.target_count.unwrap_or_default() > 0);
    assert!(report.integration_test_target_count.unwrap_or_default() > 0);
    assert!(!report.suspected_cost_centers.is_empty());
    let value = serde_json::to_value(report).expect("json");
    assert!(matches!(value, Value::Object(_)));
}
