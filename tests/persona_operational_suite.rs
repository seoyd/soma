use soma_zero::{
    CommitteeDiagnosticsAggregate, CommitteeDiagnosticsRecommendation, CommitteeDiagnosticsStatus,
    CommitteeEvaluationScaffold, CommitteeEvidenceQualityReport, CommitteeEvidenceQualityStatus,
    CommitteeReplayReport, EvidenceSourceKind, PersonaConflictMatrix, PersonaConflictStatus,
    PersonaHorizon, PersonaOperationalStatus, PersonaStance, PersonaVote, ReasonCode,
    RiskBridgeDiagnosticsReport, SixPersonaDesignRecommendation, build_status_report_from_votes,
    evaluate_six_persona_design_readiness, idle_trinity_operational_status_report,
};

fn aggregate(
    quality_status: CommitteeEvidenceQualityStatus,
    official_count: usize,
    scenario_count: usize,
    conflict_status: PersonaConflictStatus,
    average_disagreement: f64,
    groupthink_frequency: f64,
    final_status: CommitteeDiagnosticsStatus,
) -> CommitteeDiagnosticsAggregate {
    CommitteeDiagnosticsAggregate {
        replay_report: CommitteeReplayReport {
            replay_id: "replay".to_string(),
            records: Vec::new(),
            record_count: scenario_count,
            source_summary: "summary".to_string(),
            final_action_counts: std::collections::BTreeMap::new(),
            risk_denial_counts: std::collections::BTreeMap::new(),
            chair_decision_counts: std::collections::BTreeMap::new(),
            deterministic_fingerprint: "fp".to_string(),
            reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
        },
        chair_diagnostics: Vec::new(),
        risk_diagnostics: vec![RiskBridgeDiagnosticsReport {
            decision_id: "d1".to_string(),
            committee_final_decision: "ApproveCandidate".to_string(),
            risk_proposal_summary: "paper-only".to_string(),
            risk_governor_decision: "ApprovePaper".to_string(),
            final_action: "PaperApprove".to_string(),
            veto_applied: false,
            denial_reason_codes: Vec::new(),
            emergency_stop_triggered: false,
            cooldown_triggered: false,
            data_quality_block: false,
            negative_edge_block: false,
            invalid_prediction_block: false,
            schema_mismatch_block: false,
            diagnostic_status: soma_zero::RiskBridgeDiagnosticStatus::RiskPassed,
            reason_codes: vec![ReasonCode::RiskBridgeDiagnosticsBuilt],
        }],
        conflict_matrix: PersonaConflictMatrix {
            pairs: Vec::new(),
            most_aligned_pairs: Vec::new(),
            most_conflicted_pairs: Vec::new(),
            average_disagreement,
            groupthink_frequency,
            conflict_status,
            reason_codes: vec![ReasonCode::PersonaConflictMatrixBuilt],
        },
        evidence_quality_report: CommitteeEvidenceQualityReport {
            source_summary: "summary".to_string(),
            official_count,
            crypto_only_count: 0,
            yfinance_research_count: if quality_status
                == CommitteeEvidenceQualityStatus::ResearchOnlyEvidence
            {
                scenario_count
            } else {
                0
            },
            fixture_count: if quality_status == CommitteeEvidenceQualityStatus::FixtureOnlyEvidence
            {
                scenario_count
            } else {
                0
            },
            synthetic_test_count: 0,
            missing_provenance_count: 0,
            low_quality_count: 0,
            scenario_count,
            enough_for_design_review: quality_status
                == CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable
                && official_count >= 5,
            quality_status,
            warnings: Vec::new(),
            reason_codes: vec![ReasonCode::CommitteeEvidenceQualityBuilt],
        },
        evaluation_scaffold: CommitteeEvaluationScaffold {
            persona_metrics: Vec::new(),
            chair_metrics: Vec::new(),
            risk_metrics: Vec::new(),
            enough_samples: scenario_count >= 10,
            recommendation: soma_zero::CommitteeEvaluationRecommendation::KeepCurrentPersonas,
            reason_codes: vec![ReasonCode::CommitteeEvaluationScaffoldBuilt],
        },
        final_status,
        recommendation: CommitteeDiagnosticsRecommendation::KeepTrinity,
        reason_codes: vec![ReasonCode::CommitteeDiagnosticsBuilt],
    }
}

fn votes() -> [PersonaVote; 3] {
    [
        PersonaVote {
            persona_id: "trend_breakout_fast".to_string(),
            stance: PersonaStance::Approve,
            conviction: 0.7,
            voice_power: 0.8,
            horizon: PersonaHorizon::Intraday,
            source_kind: EvidenceSourceKind::OfficialApiCollected,
            regime_fit: 1.0,
            data_quality_fit: 1.0,
            risk_fit: 1.0,
            expected_edge_fit: 1.0,
            doctrine_violations: vec![],
            reason_codes: vec![],
        },
        PersonaVote {
            persona_id: "defensive_value_risk".to_string(),
            stance: PersonaStance::Abstain,
            conviction: 0.4,
            voice_power: 0.5,
            horizon: PersonaHorizon::Swing,
            source_kind: EvidenceSourceKind::OfficialApiCollected,
            regime_fit: 0.8,
            data_quality_fit: 0.9,
            risk_fit: 0.8,
            expected_edge_fit: 0.6,
            doctrine_violations: vec![],
            reason_codes: vec![],
        },
        PersonaVote {
            persona_id: "cycle_regime_guard".to_string(),
            stance: PersonaStance::Veto,
            conviction: 0.9,
            voice_power: 0.6,
            horizon: PersonaHorizon::Swing,
            source_kind: EvidenceSourceKind::OfficialApiCollected,
            regime_fit: 0.7,
            data_quality_fit: 0.7,
            risk_fit: 0.7,
            expected_edge_fit: 0.4,
            doctrine_violations: vec!["hard-stop".to_string()],
            reason_codes: vec![],
        },
    ]
}

#[test]
fn persona_operational_status_captures_done_abstain_and_veto() {
    let report = build_status_report_from_votes("candidate", "AAPL", &votes());
    assert_eq!(report.active_count, 3);
    assert!(
        report
            .persona_views
            .iter()
            .any(|view| view.status == PersonaOperationalStatus::Done)
    );
    assert!(
        report
            .persona_views
            .iter()
            .any(|view| view.status == PersonaOperationalStatus::Abstained)
    );
    assert!(
        report
            .persona_views
            .iter()
            .any(|view| view.status == PersonaOperationalStatus::Vetoed)
    );
    assert_eq!(idle_trinity_operational_status_report().active_count, 3);
}

#[test]
fn fixture_only_and_research_only_cannot_pass() {
    let fixture = evaluate_six_persona_design_readiness(
        &aggregate(
            CommitteeEvidenceQualityStatus::FixtureOnlyEvidence,
            0,
            10,
            PersonaConflictStatus::HealthyDiversity,
            0.3,
            0.2,
            CommitteeDiagnosticsStatus::EvidenceTooWeak,
        ),
        &soma_zero::SixPersonaDesignReadinessConfig::default(),
    );
    let research = evaluate_six_persona_design_readiness(
        &aggregate(
            CommitteeEvidenceQualityStatus::ResearchOnlyEvidence,
            0,
            10,
            PersonaConflictStatus::HealthyDiversity,
            0.3,
            0.2,
            CommitteeDiagnosticsStatus::ResearchOnly,
        ),
        &soma_zero::SixPersonaDesignReadinessConfig::default(),
    );
    assert!(!fixture.ready_for_design_review);
    assert!(!research.ready_for_design_review);
}

#[test]
fn official_sufficient_controlled_case_stays_design_review_only() {
    let report = evaluate_six_persona_design_readiness(
        &aggregate(
            CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable,
            6,
            10,
            PersonaConflictStatus::HealthyDiversity,
            0.30,
            0.20,
            CommitteeDiagnosticsStatus::DiagnosticsHealthy,
        ),
        &soma_zero::SixPersonaDesignReadinessConfig::default(),
    );
    assert!(report.ready_for_design_review);
    assert_eq!(
        report.recommendation,
        SixPersonaDesignRecommendation::SixPersonaDesignReviewOnly
    );
}

#[test]
fn persona_operational_report_stays_paper_only_and_deterministic() {
    let first = build_status_report_from_votes("candidate", "AAPL", &votes());
    let second = build_status_report_from_votes("candidate", "AAPL", &votes());
    let first_json = serde_json::to_string(&first).expect("first");
    let second_json = serde_json::to_string(&second).expect("second");
    assert_eq!(first_json, second_json);
    assert!(!first_json.to_ascii_lowercase().contains("runtime_llm"));
    assert!(!first_json.to_ascii_lowercase().contains("order_id"));
    assert!(!first_json.to_ascii_lowercase().contains("account_id"));
}
