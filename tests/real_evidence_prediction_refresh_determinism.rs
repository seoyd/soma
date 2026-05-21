#[path = "support/sprint69_support.rs"]
mod support;

#[test]
fn sprint75_outputs_are_deterministic() {
    let first = support::run_sprint75_bundle(
        "soma_real_prediction_requirements.toml",
        "prediction-refresh-determinism-a",
    );
    let second = support::run_sprint75_bundle(
        "soma_real_prediction_requirements.toml",
        "prediction-refresh-determinism-b",
    );
    assert_eq!(first, second);
}
