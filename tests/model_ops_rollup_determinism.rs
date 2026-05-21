mod common;
#[path = "support/sprint67_support.rs"]
mod sprint67_support;

use soma_zero::ModelOpsRollupRunner;

#[test]
fn rollup_bundle_and_panel_are_deterministic() {
    let config = sprint67_support::rollup_config_from_example(
        "soma_model_ops_rollup.toml",
        "rollup-determinism",
    );
    let runner = ModelOpsRollupRunner::default();
    let first = runner.run(&config).expect("first rollup");
    let second = runner.run(&config).expect("second rollup");
    assert_eq!(first, second);
}
