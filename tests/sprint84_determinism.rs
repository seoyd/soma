mod support;

use support::sprint69_support as sprint;

#[test]
fn sprint84_bundle_is_deterministic() {
    let first = sprint::run_sprint84_bundle(
        "soma_sprint84_test_cost_reduce.toml",
        "sprint84-determinism-a",
    );
    let second = sprint::run_sprint84_bundle(
        "soma_sprint84_test_cost_reduce.toml",
        "sprint84-determinism-b",
    );
    assert_eq!(first, second);
}
