use soma_zero::{
    ExpansionReadinessDecision, NextStepRecommendation, Sprint14DecisionRecord,
    Sprint14DecisionRouter, Sprint14EvidenceInput, Sprint14Track,
};

#[test]
fn sprint14_decision_record_can_be_constructed() {
    let record = Sprint14DecisionRecord {
        selected_track: Sprint14Track::NeedMoreExperiments,
        reason: "test".to_string(),
        evidence_inputs: Sprint14EvidenceInput {
            source_study_id: Some("study".to_string()),
            source_report_path: Some("report.json".to_string()),
            sprint13_next_step: Some(NextStepRecommendation::NeedMoreExperiments),
            dominant_dimension: None,
            dataset_count: Some(2),
            usable_dataset_count: Some(2),
            total_outcome_records: Some(0),
            regime_coverage_count: Some(0),
            comparable_variant_count: Some(0),
            average_data_quality_score: Some(0.87),
            baseline_failed_runs: Some(1),
            expansion_readiness_decision: Some(ExpansionReadinessDecision::NeedMoreExperiments),
            warnings: vec![],
            blockers: vec![],
        },
        rejected_tracks: vec![],
        blockers: vec![],
        warnings: vec![],
        reason_codes: vec![],
    };
    assert_eq!(record.selected_track, Sprint14Track::NeedMoreExperiments);
}

#[test]
fn decision_router_selects_need_more_experiments_when_input_is_missing() {
    let record = Sprint14DecisionRouter::default().decide(None);
    assert_eq!(record.selected_track, Sprint14Track::NeedMoreExperiments);
}

#[test]
fn decision_router_selects_safety_critical_track_first_when_tracks_conflict() {
    let record = Sprint14DecisionRouter::default().decide(Some(&Sprint14EvidenceInput {
        source_study_id: Some("study".to_string()),
        source_report_path: None,
        sprint13_next_step: Some(NextStepRecommendation::TightenRiskGates),
        dominant_dimension: None,
        dataset_count: Some(2),
        usable_dataset_count: Some(2),
        total_outcome_records: Some(4),
        regime_coverage_count: Some(0),
        comparable_variant_count: Some(1),
        average_data_quality_score: Some(0.70),
        baseline_failed_runs: Some(1),
        expansion_readiness_decision: Some(ExpansionReadinessDecision::NeedMoreExperiments),
        warnings: vec!["thin evidence".to_string()],
        blockers: vec![],
    }));
    assert_eq!(record.selected_track, Sprint14Track::ImproveDataFirst);
}

#[test]
fn rejected_tracks_are_recorded() {
    let record = Sprint14DecisionRouter::default().decide(Some(&Sprint14EvidenceInput {
        source_study_id: Some("study".to_string()),
        source_report_path: None,
        sprint13_next_step: Some(NextStepRecommendation::TightenRiskGates),
        dominant_dimension: None,
        dataset_count: Some(2),
        usable_dataset_count: Some(2),
        total_outcome_records: Some(0),
        regime_coverage_count: Some(0),
        comparable_variant_count: Some(0),
        average_data_quality_score: Some(0.87),
        baseline_failed_runs: Some(1),
        expansion_readiness_decision: Some(ExpansionReadinessDecision::NeedMoreExperiments),
        warnings: vec![],
        blockers: vec![],
    }));
    assert!(record.rejected_tracks.iter().any(|track| {
        track.track == Sprint14Track::ImproveRiskGovernorFirst
            || track.track == Sprint14Track::NeedMoreExperiments
    }));
}
