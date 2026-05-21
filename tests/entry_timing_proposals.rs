mod support;

use soma_zero::{EntryTimingProposal, EntryTimingProposalStatus, EntryTimingWindow, Timeframe};
use support::sprint98_support::run_sprint98;

#[test]
fn entry_timing_proposals_cover_required_windows_and_conditions() {
    let bundle = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "entry-timing-proposals",
    );
    let windows = bundle
        .entry_timing_proposals
        .iter()
        .map(|proposal| proposal.entry_window)
        .collect::<Vec<_>>();
    for window in [
        EntryTimingWindow::NextCandle,
        EntryTimingWindow::PullbackConfirmation,
        EntryTimingWindow::VolatilityCooldown,
        EntryTimingWindow::NoEntry,
    ] {
        assert!(windows.contains(&window), "missing {window:?}");
    }
    let immediate = EntryTimingProposal {
        timing_id: "immediate".to_string(),
        member_id: "trend-scout".to_string(),
        symbol: "TEST".to_string(),
        timeframe: Timeframe::OneDay,
        entry_window: EntryTimingWindow::ImmediatePaperOnly,
        earliest_entry_timestamp: Some(1),
        latest_entry_timestamp: Some(2),
        confirmation_conditions: vec!["paper open".to_string()],
        cancellation_conditions: vec!["risk deny".to_string()],
        required_risk_checks: vec!["paper only".to_string()],
        timing_status: EntryTimingProposalStatus::EntryTimingReady,
        reason_codes: vec![],
    };
    assert_eq!(
        immediate.entry_window,
        EntryTimingWindow::ImmediatePaperOnly
    );
    assert!(
        bundle
            .entry_timing_proposals
            .iter()
            .all(|proposal| !proposal.confirmation_conditions.is_empty())
    );
    assert!(
        bundle
            .entry_timing_proposals
            .iter()
            .all(|proposal| !proposal.cancellation_conditions.is_empty())
    );
}
