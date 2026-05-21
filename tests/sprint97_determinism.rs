mod support;

use support::sprint69_support as sprint;

#[test]
fn sprint97_bundle_is_deterministic_for_same_inputs() {
    let first = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "sprint97-determinism-a",
    );
    let second = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "sprint97-determinism-b",
    );
    assert_eq!(first, second);
}
