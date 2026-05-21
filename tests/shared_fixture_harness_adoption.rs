mod support;

use soma_zero::{SharedFixtureHarnessAdoptionStatus, Sprint85WorkspaceGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn shared_fixture_harness_adoption_reports_migration_and_duplicates() {
    let config = sprint::sprint85_config_from_example(
        "soma_shared_fixture_adoption.toml",
        "shared-fixture-adoption-test",
    );
    let report = Sprint85WorkspaceGateRecoveryRunner::default()
        .run_shared_fixture_adoption(&config)
        .expect("adoption");
    assert_eq!(
        report.adoption_status,
        SharedFixtureHarnessAdoptionStatus::HarnessAdoptionReadyWithWarnings
    );
    assert_eq!(report.duplicate_setup_removed, 9);
    assert!(
        report
            .remaining_duplicate_setup
            .iter()
            .any(|name| name.ends_with("persona_readiness.rs"))
    );
}
