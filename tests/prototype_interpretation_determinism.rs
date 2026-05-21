use soma_zero::PrototypeComparisonInterpretationBundle;

#[path = "support/sprint69_support.rs"]
mod support;

#[test]
fn sprint81_bundle_is_deterministic() {
    let first: PrototypeComparisonInterpretationBundle =
        support::run_sprint81_bundle("soma_prototype_interpretation.toml", "determinism-a");
    let second: PrototypeComparisonInterpretationBundle =
        support::run_sprint81_bundle("soma_prototype_interpretation.toml", "determinism-b");
    assert_eq!(first, second);
}
