mod support;

use soma_zero::{
    CommitteeCliSafetyAssertionMigrationStatus, Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn assertion_migration_represents_grouped_suites_and_keeps_high_risk_sentinel() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_assertion_migration(&sprint::sprint95_config_from_example(
            "soma_committee_cli_safety_assertion_migration.toml",
            "committee-cli-safety-assertion-migration",
        ))
        .expect("report");
    assert_eq!(
        report.migration_status,
        CommitteeCliSafetyAssertionMigrationStatus::AssertionsRepresentedWithIsolatedSentinel
    );
    assert!(report.assertions_represented >= 4);
    assert!(report.assertions_remaining_isolated >= 1);
    assert_eq!(report.assertions_remaining_uncovered, 0);
}
