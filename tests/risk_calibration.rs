use soma_zero::{
    CommitteeDecisionQualityReport, CommitteeDecisionQualityStatus, CommitteeEvidenceQualityReport,
    CommitteeEvidenceQualityStatus, ReasonCode, RiskBridgeDiagnosticStatus,
    RiskBridgeDiagnosticsReport, RiskCalibrationRecommendation, build_risk_calibration_report,
};

fn evidence(status: CommitteeEvidenceQualityStatus) -> CommitteeEvidenceQualityReport {
    CommitteeEvidenceQualityReport {
        source_summary: "test".to_string(),
        official_count: 5,
        crypto_only_count: 0,
        yfinance_research_count: 0,
        fixture_count: 0,
        synthetic_test_count: 0,
        missing_provenance_count: 0,
        low_quality_count: 0,
        scenario_count: 5,
        enough_for_design_review: true,
        quality_status: status,
        warnings: Vec::new(),
        reason_codes: vec![ReasonCode::CommitteeEvidenceQualityBuilt],
    }
}

fn decision_quality() -> CommitteeDecisionQualityReport {
    CommitteeDecisionQualityReport {
        decision_count: 5,
        source_summary: "official".to_string(),
        final_action_counts: std::collections::BTreeMap::new(),
        chair_decision_counts: std::collections::BTreeMap::new(),
        persona_stance_counts: std::collections::BTreeMap::new(),
        no_trade_ratio: 0.2,
        approve_candidate_ratio: 0.4,
        reduce_size_ratio: 0.0,
        require_confirm_ratio: 0.0,
        risk_denial_ratio: 0.8,
        hard_veto_ratio: 0.2,
        emergency_stop_ratio: 0.0,
        cooldown_ratio: 0.0,
        groupthink_warning_ratio: 0.0,
        high_disagreement_ratio: 0.0,
        average_disagreement: 0.2,
        average_uncertainty: 0.2,
        average_weighted_score: 0.2,
        average_expected_edge_after_cost: 0.01,
        average_expected_drawdown: 0.02,
        data_quality_distribution: std::collections::BTreeMap::new(),
        evidence_quality_status: CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable,
        quality_status: CommitteeDecisionQualityStatus::HealthyResearchMvp,
        reason_codes: vec![ReasonCode::CommitteeDecisionQualityBuilt],
    }
}

fn report(
    status: RiskBridgeDiagnosticStatus,
    veto: bool,
    reason_codes: Vec<ReasonCode>,
) -> RiskBridgeDiagnosticsReport {
    RiskBridgeDiagnosticsReport {
        decision_id: "d".to_string(),
        committee_final_decision: "ApproveCandidate".to_string(),
        risk_proposal_summary: "proposal".to_string(),
        risk_governor_decision: format!("{status:?}"),
        final_action: if veto { "FinalDenied" } else { "PaperApprove" }.to_string(),
        veto_applied: veto,
        denial_reason_codes: reason_codes.clone(),
        emergency_stop_triggered: status == RiskBridgeDiagnosticStatus::EmergencyStop,
        cooldown_triggered: status == RiskBridgeDiagnosticStatus::Cooldown,
        data_quality_block: reason_codes.contains(&ReasonCode::DataQualityGateBreached),
        negative_edge_block: reason_codes.contains(&ReasonCode::ExpectedEdgeBelowThreshold),
        invalid_prediction_block: false,
        schema_mismatch_block: false,
        diagnostic_status: status,
        reason_codes: vec![ReasonCode::RiskBridgeDiagnosticsBuilt],
    }
}

#[test]
fn valid_risk_denials_do_not_imply_risk_bug() {
    let report = build_risk_calibration_report(
        &vec![
            report(
                RiskBridgeDiagnosticStatus::RiskDeniedExpected,
                true,
                vec![ReasonCode::ExpectedEdgeBelowThreshold],
            );
            4
        ],
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
        &decision_quality(),
    );
    assert_eq!(
        report.final_recommendation,
        RiskCalibrationRecommendation::ImproveRiskDiagnostics
    );
}

#[test]
fn weak_evidence_yields_need_more_evidence() {
    let report = build_risk_calibration_report(
        &vec![
            report(
                RiskBridgeDiagnosticStatus::RiskDeniedUnexpected,
                true,
                vec![ReasonCode::DeniedByDefault],
            );
            2
        ],
        &evidence(CommitteeEvidenceQualityStatus::FixtureOnlyEvidence),
        &decision_quality(),
    );
    assert_eq!(
        report.final_recommendation,
        RiskCalibrationRecommendation::NeedMoreEvidence
    );
}

#[test]
fn soft_threshold_all_denials_mark_research_only_overblocking_review() {
    let report = build_risk_calibration_report(
        &vec![
            report(
                RiskBridgeDiagnosticStatus::RiskDeniedUnexpected,
                true,
                vec![ReasonCode::ConfidenceGateBreached],
            );
            5
        ],
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
        &decision_quality(),
    );
    assert!(report.overblocking_suspected);
    assert_eq!(
        report.final_recommendation,
        RiskCalibrationRecommendation::ResearchOnlyReviewForOverblocking
    );
    assert!(
        report
            .suggestions
            .iter()
            .all(|suggestion| !suggestion.apply_automatically)
    );
    assert!(
        report
            .suggestions
            .iter()
            .all(|suggestion| !suggestion.hard_veto_affected)
    );
}

#[test]
fn risk_calibration_is_deterministic() {
    let first = build_risk_calibration_report(
        &vec![
            report(
                RiskBridgeDiagnosticStatus::RiskDeniedUnexpected,
                true,
                vec![ReasonCode::ConfidenceGateBreached],
            );
            5
        ],
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
        &decision_quality(),
    );
    let second = build_risk_calibration_report(
        &vec![
            report(
                RiskBridgeDiagnosticStatus::RiskDeniedUnexpected,
                true,
                vec![ReasonCode::ConfidenceGateBreached],
            );
            5
        ],
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
        &decision_quality(),
    );
    assert_eq!(first.to_text(), second.to_text());
}
