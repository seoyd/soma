mod support;

use soma_zero::{KrxEvidenceAssertionMigrationStatus, Sprint91KrxEvidenceRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn krx_evidence_assertion_migration_counts_suite_assertions_and_keeps_reason() {
    let config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_assertion_migration.toml",
        "krx-assertion-migration",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_assertion_migration(&config)
        .expect("report");
    assert_eq!(report.assertions_found, 11);
    assert_eq!(report.assertions_migrated, 10);
    assert_eq!(report.assertions_remaining, 1);
    assert_eq!(
        report.migration_status,
        KrxEvidenceAssertionMigrationStatus::AssertionsMigratedWithWarnings
    );
    assert_eq!(report.high_risk_assertions_kept_separate.len(), 1);
}

#[test]
fn krx_evidence_assertion_migration_is_deterministic() {
    let config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_assertion_migration.toml",
        "krx-assertion-migration-deterministic",
    );
    let first = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_assertion_migration(&config)
        .expect("first");
    let second = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_assertion_migration(&config)
        .expect("second");
    assert_eq!(first, second);
}
