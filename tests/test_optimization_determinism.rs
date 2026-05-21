#[path = "support/sprint69_support.rs"]
mod support;

#[test]
fn sprint77_outputs_are_deterministic() {
    let first =
        support::run_sprint77_bundle("soma_repeated_workspace_timing.toml", "test-optimization-a");
    let second =
        support::run_sprint77_bundle("soma_repeated_workspace_timing.toml", "test-optimization-b");
    assert_eq!(first, second);
    let expected: serde_json::Value = support::read_json(support::example_path(
        "sprint77_data/expected_test_optimization_summary.json",
    ));
    assert_eq!(
        first.final_summary,
        expected
            .get("final_summary")
            .and_then(|value| value.as_str())
            .expect("summary")
    );
}
