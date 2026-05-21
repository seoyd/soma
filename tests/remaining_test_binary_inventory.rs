mod support;

use soma_zero::{
    RemainingTestBinaryFamily, RemainingTestBinaryInventoryStatus,
    Sprint85WorkspaceGateRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn remaining_test_binary_inventory_classifies_remaining_families() {
    let config = sprint::sprint85_config_from_example(
        "soma_remaining_test_binary_inventory.toml",
        "remaining-test-binary-inventory-test",
    );
    let report = Sprint85WorkspaceGateRecoveryRunner::default()
        .run_remaining_test_binary_inventory(&config)
        .expect("inventory");
    assert_eq!(
        report.inventory_status,
        RemainingTestBinaryInventoryStatus::RemainingInventoryReadyWithWarnings
    );
    assert_eq!(report.total_remaining_count, 16);
    assert_eq!(report.high_volume_count, 15);
    assert!(
        report
            .records
            .iter()
            .any(|record| record.family == RemainingTestBinaryFamily::SafetyGuard)
    );
}
