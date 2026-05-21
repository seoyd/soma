use soma_zero::{
    ChairCalibrationRecommendation, ChairCalibrationReport, CommitteeDecisionQualityReport,
    CommitteeDecisionQualityStatus, CommitteeEvidenceQualityReport, CommitteeEvidenceQualityStatus,
    CommitteeV1NextRecommendation, CommitteeV1ReadinessStatus, PersonaConflictMatrix,
    PersonaConflictStatus, ReasonCode, RiskCalibrationRecommendation, RiskCalibrationReport,
    build_committee_v1_readiness_report,
};

fn evidence(
    status: CommitteeEvidenceQualityStatus,
    official_count: usize,
    scenario_count: usize,
) -> CommitteeEvidenceQualityReport {
    CommitteeEvidenceQualityReport {
        source_summary: "test".to_string(),
        official_count,
        crypto_only_count: 0,
        yfinance_research_count: usize::from(
            status == CommitteeEvidenceQualityStatus::ResearchOnlyEvidence,
        ),
        fixture_count: usize::from(status == CommitteeEvidenceQualityStatus::FixtureOnlyEvidence)
            * scenario_count,
        synthetic_test_count: 0,
        missing_provenance_count: 0,
        low_quality_count: 0,
        scenario_count,
        enough_for_design_review: status
            == CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable,
        quality_status: status,
        warnings: Vec::new(),
        reason_codes: vec![ReasonCode::CommitteeEvidenceQualityBuilt],
    }
}

fn decision(
    status: CommitteeDecisionQualityStatus,
    count: usize,
) -> CommitteeDecisionQualityReport {
    CommitteeDecisionQualityReport {
        decision_count: count,
        source_summary: "test".to_string(),
        final_action_counts: std::collections::BTreeMap::new(),
        chair_decision_counts: std::collections::BTreeMap::new(),
        persona_stance_counts: std::collections::BTreeMap::new(),
        no_trade_ratio: 0.2,
        approve_candidate_ratio: 0.4,
        reduce_size_ratio: 0.0,
        require_confirm_ratio: 0.0,
        risk_denial_ratio: 0.2,
        hard_veto_ratio: 0.2,
        emergency_stop_ratio: 0.0,
        cooldown_ratio: 0.0,
        groupthink_warning_ratio: 0.2,
        high_disagreement_ratio: 0.2,
        average_disagreement: 0.2,
        average_uncertainty: 0.2,
        average_weighted_score: 0.2,
        average_expected_edge_after_cost: 0.01,
        average_expected_drawdown: 0.02,
        data_quality_distribution: std::collections::BTreeMap::new(),
        evidence_quality_status: CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable,
        quality_status: status,
        reason_codes: vec![ReasonCode::CommitteeDecisionQualityBuilt],
    }
}

fn chair(final_recommendation: ChairCalibrationRecommendation) -> ChairCalibrationReport {
    ChairCalibrationReport {
        suggestions: Vec::new(),
        groupthink_suggestions: Vec::new(),
        disagreement_suggestions: Vec::new(),
        speaker_filter_suggestions: Vec::new(),
        cluster_penalty_suggestions: Vec::new(),
        contrarian_inclusion_suggestions: Vec::new(),
        final_recommendation,
        reason_codes: vec![ReasonCode::ChairCalibrationBuilt],
    }
}

fn risk(
    final_recommendation: RiskCalibrationRecommendation,
    overblocking: bool,
    underblocking: bool,
) -> RiskCalibrationReport {
    RiskCalibrationReport {
        suggestions: Vec::new(),
        overblocking_suspected: overblocking,
        underblocking_suspected: underblocking,
        final_recommendation,
        reason_codes: vec![ReasonCode::RiskCalibrationBuilt],
    }
}

fn conflict(
    status: PersonaConflictStatus,
    average_disagreement: f64,
    groupthink_frequency: f64,
) -> PersonaConflictMatrix {
    PersonaConflictMatrix {
        pairs: Vec::new(),
        most_aligned_pairs: Vec::new(),
        most_conflicted_pairs: Vec::new(),
        average_disagreement,
        groupthink_frequency,
        conflict_status: status,
        reason_codes: vec![ReasonCode::PersonaConflictMatrixBuilt],
    }
}

#[test]
fn fixture_and_research_only_are_not_ready() {
    let fixture = build_committee_v1_readiness_report(
        &evidence(CommitteeEvidenceQualityStatus::FixtureOnlyEvidence, 0, 5),
        &decision(CommitteeDecisionQualityStatus::FixtureOnly, 5),
        &chair(ChairCalibrationRecommendation::KeepChairV0),
        &risk(
            RiskCalibrationRecommendation::KeepRiskGovernor,
            false,
            false,
        ),
        &conflict(PersonaConflictStatus::HealthyDiversity, 0.2, 0.2),
    );
    assert_eq!(
        fixture.status,
        CommitteeV1ReadinessStatus::NotReadyFixtureOnly
    );

    let research = build_committee_v1_readiness_report(
        &evidence(CommitteeEvidenceQualityStatus::ResearchOnlyEvidence, 0, 5),
        &decision(CommitteeDecisionQualityStatus::ResearchOnly, 5),
        &chair(ChairCalibrationRecommendation::KeepChairV0),
        &risk(
            RiskCalibrationRecommendation::KeepRiskGovernor,
            false,
            false,
        ),
        &conflict(PersonaConflictStatus::HealthyDiversity, 0.2, 0.2),
    );
    assert_eq!(
        research.status,
        CommitteeV1ReadinessStatus::NotReadyResearchOnly
    );
}

#[test]
fn too_few_samples_groupthink_and_risk_instability_are_blocked() {
    let too_few = build_committee_v1_readiness_report(
        &evidence(
            CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable,
            5,
            2,
        ),
        &decision(CommitteeDecisionQualityStatus::InsufficientSamples, 2),
        &chair(ChairCalibrationRecommendation::KeepChairV0),
        &risk(
            RiskCalibrationRecommendation::KeepRiskGovernor,
            false,
            false,
        ),
        &conflict(PersonaConflictStatus::InsufficientSamples, 0.0, 0.0),
    );
    assert_eq!(
        too_few.status,
        CommitteeV1ReadinessStatus::NotReadyTooFewSamples
    );

    let groupthink = build_committee_v1_readiness_report(
        &evidence(
            CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable,
            5,
            5,
        ),
        &CommitteeDecisionQualityReport {
            groupthink_warning_ratio: 0.8,
            ..decision(CommitteeDecisionQualityStatus::TooMuchGroupthink, 5)
        },
        &chair(ChairCalibrationRecommendation::IncreaseContrarianProtection),
        &risk(
            RiskCalibrationRecommendation::KeepRiskGovernor,
            false,
            false,
        ),
        &conflict(PersonaConflictStatus::TooAligned, 0.1, 0.8),
    );
    assert_eq!(
        groupthink.status,
        CommitteeV1ReadinessStatus::NotReadyGroupthink
    );

    let risk_unstable = build_committee_v1_readiness_report(
        &evidence(
            CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable,
            5,
            5,
        ),
        &CommitteeDecisionQualityReport {
            emergency_stop_ratio: 0.5,
            ..decision(CommitteeDecisionQualityStatus::HealthyResearchMvp, 5)
        },
        &chair(ChairCalibrationRecommendation::KeepChairV0),
        &risk(RiskCalibrationRecommendation::TightenRiskRules, false, true),
        &conflict(PersonaConflictStatus::HealthyDiversity, 0.2, 0.2),
    );
    assert_eq!(
        risk_unstable.status,
        CommitteeV1ReadinessStatus::NotReadyRiskUnstable
    );
}

#[test]
fn healthy_official_paths_produce_ready_states_without_activation() {
    let benchmark = build_committee_v1_readiness_report(
        &evidence(
            CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable,
            5,
            5,
        ),
        &decision(CommitteeDecisionQualityStatus::HealthyResearchMvp, 5),
        &chair(ChairCalibrationRecommendation::KeepChairV0),
        &risk(
            RiskCalibrationRecommendation::KeepRiskGovernor,
            false,
            false,
        ),
        &conflict(PersonaConflictStatus::HealthyDiversity, 0.2, 0.2),
    );
    assert!(matches!(
        benchmark.next_recommendation,
        CommitteeV1NextRecommendation::RunCommitteeBenchmark
            | CommitteeV1NextRecommendation::SixPersonaDesignReviewOnly
    ));
    assert!(matches!(
        benchmark.status,
        CommitteeV1ReadinessStatus::ReadyForCommitteeBenchmark
            | CommitteeV1ReadinessStatus::ReadyForSixPersonaDesignReviewOnly
    ));
}

#[test]
fn readiness_is_deterministic() {
    let first = build_committee_v1_readiness_report(
        &evidence(
            CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable,
            5,
            5,
        ),
        &decision(CommitteeDecisionQualityStatus::HealthyResearchMvp, 5),
        &chair(ChairCalibrationRecommendation::KeepChairV0),
        &risk(
            RiskCalibrationRecommendation::KeepRiskGovernor,
            false,
            false,
        ),
        &conflict(PersonaConflictStatus::HealthyDiversity, 0.2, 0.2),
    );
    let second = build_committee_v1_readiness_report(
        &evidence(
            CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable,
            5,
            5,
        ),
        &decision(CommitteeDecisionQualityStatus::HealthyResearchMvp, 5),
        &chair(ChairCalibrationRecommendation::KeepChairV0),
        &risk(
            RiskCalibrationRecommendation::KeepRiskGovernor,
            false,
            false,
        ),
        &conflict(PersonaConflictStatus::HealthyDiversity, 0.2, 0.2),
    );
    assert_eq!(first.to_text(), second.to_text());
}
