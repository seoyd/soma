mod common;
#[path = "support/sprint68_support.rs"]
mod sprint68_support;

#[test]
fn sprint68_trace_bundle_is_deterministic() {
    let left = sprint68_support::run_trace("soma_model_ops_trace.toml", "trace-determinism-a");
    let right = sprint68_support::run_trace("soma_model_ops_trace.toml", "trace-determinism-b");
    let left_json = serde_json::to_string_pretty(&left).expect("serialize left bundle");
    let right_json = serde_json::to_string_pretty(&right).expect("serialize right bundle");
    assert_eq!(left_json, right_json);
}
