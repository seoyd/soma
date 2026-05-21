#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::HeavyFixtureInventoryRecommendation;

#[test]
fn heavy_fixture_inventory_detects_duplicates_and_repeated_loads() {
    let bundle = support::run_sprint76_bundle(
        "soma_rust_toolchain_modernize.toml",
        "heavy-fixture-inventory",
    );
    let inventory = bundle.heavy_fixture_inventory;
    assert!(inventory.total_fixture_bytes > 0);
    assert!(
        inventory
            .duplicate_fixture_candidates
            .contains(&"workspace_acceptance_sample.json".to_string())
    );
    assert!(
        inventory
            .repeated_load_candidates
            .contains(&"workspace_acceptance_sample.json".to_string())
    );
    assert_eq!(
        inventory.recommendation,
        HeavyFixtureInventoryRecommendation::DeduplicateFixture
    );
}
