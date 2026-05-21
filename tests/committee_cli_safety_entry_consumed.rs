mod support;

use soma_zero::{CommitteeCliSafetyEntryConsumedStatus, Sprint95CommitteeCliSafetyRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn committee_cli_safety_entry_is_consumed_when_isolation_is_explicit() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_entry_consumed(&sprint::sprint95_config_from_example(
            "soma_committee_cli_safety_entry_consumed.toml",
            "committee-cli-safety-entry-consumed",
        ))
        .expect("report");
    assert_eq!(
        report.consumed_status,
        CommitteeCliSafetyEntryConsumedStatus::EntryConsumedForCommitteeCliSafety
    );
    assert!(report.entry_consumed);
}
