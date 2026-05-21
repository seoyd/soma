#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{RustToolchainModernizationRunner, SlowTestCategory, SlowTestInventoryStatus};

#[test]
fn slow_test_inventory_uses_sample_categories_and_actions() {
    let config = support::sprint76_config_from_example(
        "soma_slow_test_inventory.toml",
        "slow-test-inventory",
    );
    let inventory = RustToolchainModernizationRunner::default()
        .run_slow_test_inventory(&config)
        .expect("slow inventory");
    assert_eq!(inventory.slow_test_count, inventory.slow_tests.len());
    assert!(
        inventory
            .slow_tests
            .iter()
            .any(|record| record.category == SlowTestCategory::HeavyFixture)
    );
    assert!(
        inventory
            .slow_tests
            .iter()
            .any(|record| record.category == SlowTestCategory::CliSmoke)
    );
    assert!(
        inventory
            .slow_tests
            .iter()
            .any(|record| record.category == SlowTestCategory::ArtifactRendering)
    );
    assert!(matches!(
        inventory.inventory_status,
        SlowTestInventoryStatus::SlowTestsIdentified
    ));
    assert!(!inventory.recommended_actions.is_empty());
}

#[test]
fn slow_test_inventory_is_deterministic() {
    let config = support::sprint76_config_from_example(
        "soma_slow_test_inventory.toml",
        "slow-test-inventory-deterministic",
    );
    let runner = RustToolchainModernizationRunner::default();
    let first = runner
        .run_slow_test_inventory(&config)
        .expect("first inventory");
    let second = runner
        .run_slow_test_inventory(&config)
        .expect("second inventory");
    assert_eq!(first, second);
}
