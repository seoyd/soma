use soma_zero::{
    CommitteeActionabilityReport, CommitteeActionabilityStatus, CommitteeAttributionReport,
    CommitteeAttributionStatus, CommitteeBenchmarkNextRecommendation,
    CommitteeBenchmarkReadinessStatus, CommitteeDecisionQualityReport,
    CommitteeDecisionQualityStatus, CommitteeMaterializationConfig,
    CommitteeScenarioMaterializationLevel, CommitteeScenarioRow, CommitteeScenarioSet,
    CommitteeScenarioSourceKind, EvidenceSourceKind, PersonaHorizon, ProviderMarket, ReasonCode,
    Regime, build_committee_benchmark_readiness_report,
};

fn set(
    source_kind: CommitteeScenarioSourceKind,
    evidence: EvidenceSourceKind,
    market: ProviderMarket,
    level: CommitteeScenarioMaterializationLevel,
    outcome: bool,
) -> CommitteeScenarioSet {
    CommitteeScenarioSet {
        scenario_id: "ready".to_string(),
        rows: vec![
            CommitteeScenarioRow {
                scenario_row_id: "row".to_string(),
                symbol: "AAPL".to_string(),
                timestamp_ms: 1,
                source_kind,
                evidence_source_kind: evidence,
                market,
                target_horizon: PersonaHorizon::Swing,
                feature_vector: None,
                regime: Regime::TrendUp,
                signal_summary: "test".to_string(),
                data_quality_score: 0.9,
                spread_bps: Some(5.0),
                expected_edge_after_cost: 0.01,
                expected_drawdown: 0.02,
                risk_snapshot_summary: None,
                provenance_summary: "official".to_string(),
                benchmark_status: Some("row-level".to_string()),
                baseline_signal_summary: None,
                external_prediction_summary: None,
                no_trade_counterfactual: None,
                risk_denial_counterfactual: None,
                outcome_reference: outcome.then(|| "outcome".to_string()),
                materialization_level: level,
                materialization_confidence: 0.9,
                reason_codes: vec![ReasonCode::CommitteeRowLevelMaterialized],
            };
            5
        ],
        source_summary: "Official".to_string(),
        row_count: 5,
        official_row_count: usize::from(evidence.readiness_eligible()) * 5,
        research_only_row_count: usize::from(evidence == EvidenceSourceKind::YFinanceResearch) * 5,
        fixture_row_count: usize::from(matches!(source_kind, CommitteeScenarioSourceKind::Fixture))
            * 5,
        skipped_row_count: 0,
        reason_codes: vec![ReasonCode::CommitteeMaterializationBuilt],
    }
}

fn decision(status: CommitteeDecisionQualityStatus) -> CommitteeDecisionQualityReport {
    CommitteeDecisionQualityReport {
        decision_count: 5,
        source_summary: "Official".to_string(),
        final_action_counts: std::collections::BTreeMap::new(),
        chair_decision_counts: std::collections::BTreeMap::new(),
        persona_stance_counts: std::collections::BTreeMap::new(),
        no_trade_ratio: 0.2,
        approve_candidate_ratio: 0.4,
        reduce_size_ratio: 0.0,
        require_confirm_ratio: 0.0,
        risk_denial_ratio: if status == CommitteeDecisionQualityStatus::RiskBlockedDominant {
            0.9
        } else {
            0.2
        },
        hard_veto_ratio: 0.2,
        emergency_stop_ratio: 0.0,
        cooldown_ratio: 0.0,
        groupthink_warning_ratio: if status == CommitteeDecisionQualityStatus::TooMuchGroupthink {
            0.8
        } else {
            0.2
        },
        high_disagreement_ratio: if status == CommitteeDecisionQualityStatus::TooMuchDisagreement {
            0.8
        } else {
            0.2
        },
        average_disagreement: 0.2,
        average_uncertainty: 0.2,
        average_weighted_score: 0.2,
        average_expected_edge_after_cost: 0.01,
        average_expected_drawdown: 0.02,
        data_quality_distribution: std::collections::BTreeMap::new(),
        evidence_quality_status:
            soma_zero::CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable,
        quality_status: status,
        reason_codes: vec![ReasonCode::CommitteeDecisionQualityBuilt],
    }
}

#[test]
fn readiness_blocks_fixture_research_crypto_and_weak_materialization() {
    let materialization = CommitteeMaterializationConfig::default();
    let actionability = CommitteeActionabilityReport {
        decision_count: 5,
        actionable_count: 1,
        paper_approve_count: 1,
        paper_reduce_size_count: 0,
        human_confirm_required_count: 0,
        final_no_trade_count: 2,
        final_denied_count: 2,
        research_only_count: 0,
        fixture_only_count: 0,
        official_actionable_count: 1,
        actionability_ratio: 0.2,
        official_actionability_ratio: 0.2,
        risk_block_ratio: 0.4,
        confirm_ratio: 0.0,
        actionability_status: CommitteeActionabilityStatus::ActionableResearch,
        reason_codes: vec![ReasonCode::CommitteeActionabilityBuilt],
    };
    let attribution = CommitteeAttributionReport {
        persona_contributions: Vec::new(),
        chair_contribution_summary: std::collections::BTreeMap::new(),
        risk_governor_contribution_summary: std::collections::BTreeMap::new(),
        source_contribution_summary: std::collections::BTreeMap::new(),
        high_influence_personas: Vec::new(),
        low_influence_personas: Vec::new(),
        overdominance_warnings: Vec::new(),
        underparticipation_warnings: Vec::new(),
        attribution_status: CommitteeAttributionStatus::Balanced,
        reason_codes: vec![ReasonCode::CommitteeAttributionBuilt],
    };
    let fixture = build_committee_benchmark_readiness_report(
        &set(
            CommitteeScenarioSourceKind::Fixture,
            EvidenceSourceKind::TestFixture,
            ProviderMarket::Crypto,
            CommitteeScenarioMaterializationLevel::Fixture,
            true,
        ),
        Some(&materialization),
        &decision(CommitteeDecisionQualityStatus::HealthyResearchMvp),
        &actionability,
        &attribution,
        5,
        5,
        3,
    );
    assert_eq!(
        fixture.status,
        CommitteeBenchmarkReadinessStatus::NotReadyFixtureOnly
    );

    let research = build_committee_benchmark_readiness_report(
        &set(
            CommitteeScenarioSourceKind::YahooResearchEvidenceReport,
            EvidenceSourceKind::YFinanceResearch,
            ProviderMarket::USEquity,
            CommitteeScenarioMaterializationLevel::EvidenceSummary,
            false,
        ),
        Some(&materialization),
        &decision(CommitteeDecisionQualityStatus::HealthyResearchMvp),
        &actionability,
        &attribution,
        5,
        5,
        3,
    );
    assert_eq!(
        research.status,
        CommitteeBenchmarkReadinessStatus::NotReadyResearchOnly
    );

    let crypto = build_committee_benchmark_readiness_report(
        &set(
            CommitteeScenarioSourceKind::OfficialBenchmarkReport,
            EvidenceSourceKind::OfficialApiCollected,
            ProviderMarket::Crypto,
            CommitteeScenarioMaterializationLevel::RowLevel,
            true,
        ),
        Some(&materialization),
        &decision(CommitteeDecisionQualityStatus::HealthyResearchMvp),
        &actionability,
        &attribution,
        5,
        5,
        3,
    );
    assert_eq!(
        crypto.status,
        CommitteeBenchmarkReadinessStatus::NotReadyCryptoOnly
    );

    let weak = build_committee_benchmark_readiness_report(
        &set(
            CommitteeScenarioSourceKind::OfficialBenchmarkReport,
            EvidenceSourceKind::OfficialApiCollected,
            ProviderMarket::USEquity,
            CommitteeScenarioMaterializationLevel::BenchmarkSummary,
            true,
        ),
        Some(&materialization),
        &decision(CommitteeDecisionQualityStatus::HealthyResearchMvp),
        &actionability,
        &attribution,
        5,
        5,
        3,
    );
    assert_eq!(
        weak.status,
        CommitteeBenchmarkReadinessStatus::NotReadyMaterializationWeak
    );
}

#[test]
fn controlled_official_row_level_can_be_ready() {
    let report = build_committee_benchmark_readiness_report(
        &set(
            CommitteeScenarioSourceKind::OfficialBenchmarkReport,
            EvidenceSourceKind::OfficialApiCollected,
            ProviderMarket::USEquity,
            CommitteeScenarioMaterializationLevel::RowLevel,
            true,
        ),
        Some(&CommitteeMaterializationConfig::default()),
        &decision(CommitteeDecisionQualityStatus::HealthyResearchMvp),
        &CommitteeActionabilityReport {
            decision_count: 5,
            actionable_count: 3,
            paper_approve_count: 2,
            paper_reduce_size_count: 1,
            human_confirm_required_count: 0,
            final_no_trade_count: 1,
            final_denied_count: 1,
            research_only_count: 0,
            fixture_only_count: 0,
            official_actionable_count: 3,
            actionability_ratio: 0.6,
            official_actionability_ratio: 0.6,
            risk_block_ratio: 0.2,
            confirm_ratio: 0.0,
            actionability_status: CommitteeActionabilityStatus::ActionableResearch,
            reason_codes: vec![ReasonCode::CommitteeActionabilityBuilt],
        },
        &CommitteeAttributionReport {
            persona_contributions: Vec::new(),
            chair_contribution_summary: std::collections::BTreeMap::new(),
            risk_governor_contribution_summary: std::collections::BTreeMap::new(),
            source_contribution_summary: std::collections::BTreeMap::new(),
            high_influence_personas: Vec::new(),
            low_influence_personas: Vec::new(),
            overdominance_warnings: Vec::new(),
            underparticipation_warnings: Vec::new(),
            attribution_status: CommitteeAttributionStatus::Balanced,
            reason_codes: vec![ReasonCode::CommitteeAttributionBuilt],
        },
        5,
        5,
        3,
    );
    assert!(matches!(
        report.status,
        CommitteeBenchmarkReadinessStatus::ReadyForCommitteeBenchmark
            | CommitteeBenchmarkReadinessStatus::ReadyForMoreOfficialEvidence
    ));
    assert!(matches!(
        report.next_recommendation,
        CommitteeBenchmarkNextRecommendation::KeepTrinity
            | CommitteeBenchmarkNextRecommendation::MoreOfficialCommitteeEvidence
    ));
}
