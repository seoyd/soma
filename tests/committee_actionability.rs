use soma_zero::{
    CommitteeActionabilityStatus, CommitteeFinalAction, CommitteeReplayReport,
    CommitteeScenarioSet, ReasonCode, build_committee_actionability_report,
};

#[test]
fn actionability_counts_are_computed() {
    let set = CommitteeScenarioSet {
        scenario_id: "act".to_string(),
        rows: vec![],
        source_summary: "Official".to_string(),
        row_count: 5,
        official_row_count: 5,
        research_only_row_count: 0,
        fixture_row_count: 0,
        skipped_row_count: 0,
        reason_codes: vec![ReasonCode::CommitteeMaterializationBuilt],
    };
    let report = build_committee_actionability_report(
        &set,
        &CommitteeReplayReport {
            replay_id: "act".to_string(),
            records: vec![],
            record_count: 5,
            source_summary: "Official".to_string(),
            final_action_counts: std::collections::BTreeMap::from([
                (format!("{:?}", CommitteeFinalAction::PaperApprove), 2),
                (format!("{:?}", CommitteeFinalAction::FinalDenied), 3),
            ]),
            risk_denial_counts: std::collections::BTreeMap::new(),
            chair_decision_counts: std::collections::BTreeMap::new(),
            deterministic_fingerprint: "fp".to_string(),
            reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
        },
    );
    assert_eq!(report.decision_count, 5);
    assert_eq!(
        report.actionability_status,
        CommitteeActionabilityStatus::ActionableResearch
    );
}
