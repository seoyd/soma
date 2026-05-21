mod support;

use soma_zero::CommitteeRosterBalanceStatus;
use support::sprint99_support::run_sprint99;

#[test]
fn committee_roster_balance_covers_paper_roles() {
    let bundle = run_sprint99(
        "soma_committee_roster_balance.toml",
        "committee-roster-balance",
    );
    let report = bundle.committee_roster_balance_report;
    assert_eq!(
        report.roster_balance_status,
        CommitteeRosterBalanceStatus::RosterBalancedWithWarnings
    );
    assert_eq!(report.active_member_count, 6);
    assert_eq!(report.watchlist_member_count, 1);
    assert_eq!(report.diagnostic_member_count, 1);
    assert!(report.risk_defense_coverage);
    assert!(report.entry_scout_coverage);
    assert!(report.counterfactual_coverage);
}
