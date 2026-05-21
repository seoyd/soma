use std::collections::BTreeMap;

use soma_zero::{
    ChairCalibrationRecommendation, ChairDiagnosticStatus, ChairDiagnosticsReport,
    CommitteeEvidenceQualityReport, CommitteeEvidenceQualityStatus, ReasonCode,
    build_chair_calibration_report,
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

fn report(
    status: ChairDiagnosticStatus,
    groupthink: f64,
    disagreement: f64,
) -> ChairDiagnosticsReport {
    ChairDiagnosticsReport {
        decision_id: "d".to_string(),
        speaker_traces: Vec::new(),
        selected_speakers: vec!["trend_breakout_fast".to_string()],
        filtered_speakers: vec!["defensive_value_risk".to_string()],
        cluster_counts: BTreeMap::new(),
        cluster_penalty_applied: true,
        contrarian_included: false,
        groupthink_risk: groupthink,
        disagreement_score: disagreement,
        uncertainty: 0.3,
        weighted_score: 0.2,
        final_decision: soma_zero::CommitteeDecision::ApproveCandidate,
        diagnostic_status: status,
        reason_codes: vec![ReasonCode::ChairDiagnosticsBuilt],
    }
}

#[test]
fn high_groupthink_suggests_stronger_contrarian_protection() {
    let report = build_chair_calibration_report(
        &vec![report(ChairDiagnosticStatus::GroupthinkRisk, 0.8, 0.2); 4],
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
    );
    assert_eq!(
        report.final_recommendation,
        ChairCalibrationRecommendation::IncreaseContrarianProtection
    );
    assert!(
        report
            .suggestions
            .iter()
            .all(|suggestion| !suggestion.apply_automatically)
    );
}

#[test]
fn excessive_disagreement_increases_no_trade_conservatism() {
    let report = build_chair_calibration_report(
        &vec![report(ChairDiagnosticStatus::ExcessiveDisagreement, 0.1, 0.8); 4],
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
    );
    assert_eq!(
        report.final_recommendation,
        ChairCalibrationRecommendation::IncreaseNoTradeConservatism
    );
}

#[test]
fn too_few_speakers_reduces_over_filtering() {
    let report = build_chair_calibration_report(
        &vec![report(ChairDiagnosticStatus::TooFewSpeakers, 0.1, 0.2); 4],
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
    );
    assert_eq!(
        report.final_recommendation,
        ChairCalibrationRecommendation::ReduceOverFiltering
    );
}

#[test]
fn weak_evidence_yields_need_more_evidence() {
    let report = build_chair_calibration_report(
        &vec![report(ChairDiagnosticStatus::Healthy, 0.1, 0.2); 2],
        &evidence(CommitteeEvidenceQualityStatus::FixtureOnlyEvidence),
    );
    assert_eq!(
        report.final_recommendation,
        ChairCalibrationRecommendation::NeedMoreEvidence
    );
}

#[test]
fn chair_calibration_is_deterministic() {
    let first = build_chair_calibration_report(
        &vec![report(ChairDiagnosticStatus::GroupthinkRisk, 0.8, 0.2); 4],
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
    );
    let second = build_chair_calibration_report(
        &vec![report(ChairDiagnosticStatus::GroupthinkRisk, 0.8, 0.2); 4],
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
    );
    assert_eq!(first.to_text(), second.to_text());
}
