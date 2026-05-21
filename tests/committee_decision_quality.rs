use std::collections::BTreeMap;

use soma_zero::{
    ChairDiagnosticStatus, ChairDiagnosticsReport, CommitteeDecision,
    CommitteeDecisionQualityStatus, CommitteeEvidenceQualityReport, CommitteeEvidenceQualityStatus,
    CommitteeFinalAction, CommitteeReplayRecord, CommitteeReplayReport, CommitteeScenarioRow,
    CommitteeScenarioSourceKind, PersonaConflictMatrix, PersonaConflictPair, PersonaConflictStatus,
    PersonaHorizon, PersonaStance, PersonaVote, ProviderMarket, ReasonCode, Regime,
    RiskBridgeDiagnosticStatus, RiskBridgeDiagnosticsReport,
    build_committee_decision_quality_report,
};

fn replay_record(
    action: CommitteeFinalAction,
    decision: CommitteeDecision,
    stance: PersonaStance,
    score: f64,
    quality: f64,
) -> CommitteeReplayRecord {
    CommitteeReplayRecord {
        scenario_row: CommitteeScenarioRow {
            scenario_row_id: format!("row-{score:.2}"),
            symbol: "AAPL".to_string(),
            timestamp_ms: 1,
            source_kind: CommitteeScenarioSourceKind::OfficialBenchmarkReport,
            evidence_source_kind: soma_zero::EvidenceSourceKind::OfficialApiCollected,
            market: ProviderMarket::USEquity,
            target_horizon: PersonaHorizon::Swing,
            feature_vector: None,
            regime: Regime::TrendUp,
            signal_summary: "test".to_string(),
            data_quality_score: quality,
            spread_bps: Some(5.0),
            expected_edge_after_cost: 0.01,
            expected_drawdown: 0.02,
            risk_snapshot_summary: None,
            provenance_summary: "official".to_string(),
            benchmark_status: Some("official".to_string()),
            baseline_signal_summary: None,
            external_prediction_summary: None,
            no_trade_counterfactual: None,
            risk_denial_counterfactual: None,
            outcome_reference: None,
            materialization_level:
                soma_zero::CommitteeScenarioMaterializationLevel::BenchmarkSummary,
            materialization_confidence: 0.7,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        persona_votes: vec![PersonaVote {
            persona_id: "trend_breakout_fast".to_string(),
            stance,
            conviction: 0.8,
            voice_power: 0.7,
            horizon: PersonaHorizon::Swing,
            source_kind: soma_zero::EvidenceSourceKind::OfficialApiCollected,
            regime_fit: 0.8,
            data_quality_fit: 0.9,
            risk_fit: 0.7,
            expected_edge_fit: 0.8,
            doctrine_violations: Vec::new(),
            reason_codes: vec![ReasonCode::PersonaVoteBuilt],
        }],
        chair_decision_record: soma_zero::CommitteeDecisionRecord {
            decision_id: "decision".to_string(),
            symbol: "AAPL".to_string(),
            timestamp_ms: 1,
            selected_speakers: vec!["trend_breakout_fast".to_string()],
            all_votes: Vec::new(),
            weighted_score: score,
            disagreement_score: if score < 0.0 { 0.8 } else { 0.1 },
            groupthink_risk: if score > 0.8 { 0.8 } else { 0.1 },
            uncertainty: 0.3,
            final_decision: decision,
            chair_reason_codes: vec![ReasonCode::ChairV0Built],
            source_kind: soma_zero::EvidenceSourceKind::OfficialApiCollected,
            regime: Regime::TrendUp,
            core_fingerprint: None,
            reason_codes: vec![ReasonCode::ChairV0Built],
        },
        risk_bridge_outcome: soma_zero::CommitteeOutcome {
            committee_record: soma_zero::CommitteeDecisionRecord {
                decision_id: "decision".to_string(),
                symbol: "AAPL".to_string(),
                timestamp_ms: 1,
                selected_speakers: vec!["trend_breakout_fast".to_string()],
                all_votes: Vec::new(),
                weighted_score: score,
                disagreement_score: 0.1,
                groupthink_risk: 0.1,
                uncertainty: 0.3,
                final_decision: decision,
                chair_reason_codes: vec![ReasonCode::ChairV0Built],
                source_kind: soma_zero::EvidenceSourceKind::OfficialApiCollected,
                regime: Regime::TrendUp,
                core_fingerprint: None,
                reason_codes: vec![ReasonCode::ChairV0Built],
            },
            risk_decision: soma_zero::RiskDecision {
                kind: if action == CommitteeFinalAction::FinalDenied {
                    soma_zero::RiskDecisionKind::Deny
                } else {
                    soma_zero::RiskDecisionKind::ApprovePaper
                },
                approved_order_plan: None,
                reason_codes: vec![ReasonCode::DeniedByDefault],
                audit_id: "audit".to_string(),
            },
            final_action: action,
            reason_codes: vec![ReasonCode::CommitteeRiskBridgeBuilt],
        },
        final_action: action,
        replay_fingerprint: format!("fp-{score:.2}"),
        reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
    }
}

fn evidence(status: CommitteeEvidenceQualityStatus) -> CommitteeEvidenceQualityReport {
    CommitteeEvidenceQualityReport {
        source_summary: "test".to_string(),
        official_count: 5,
        crypto_only_count: 0,
        yfinance_research_count: usize::from(
            status == CommitteeEvidenceQualityStatus::ResearchOnlyEvidence,
        ),
        fixture_count: usize::from(status == CommitteeEvidenceQualityStatus::FixtureOnlyEvidence),
        synthetic_test_count: 0,
        missing_provenance_count: 0,
        low_quality_count: 0,
        scenario_count: 5,
        enough_for_design_review: status
            == CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable,
        quality_status: status,
        warnings: Vec::new(),
        reason_codes: vec![ReasonCode::CommitteeEvidenceQualityBuilt],
    }
}

#[test]
fn decision_quality_ratios_are_computed() {
    let records = vec![
        replay_record(
            CommitteeFinalAction::FinalNoTrade,
            CommitteeDecision::NoTrade,
            PersonaStance::NoTrade,
            0.1,
            0.81,
        ),
        replay_record(
            CommitteeFinalAction::FinalDenied,
            CommitteeDecision::Vetoed,
            PersonaStance::Veto,
            0.9,
            0.91,
        ),
        replay_record(
            CommitteeFinalAction::PaperApprove,
            CommitteeDecision::ApproveCandidate,
            PersonaStance::Approve,
            0.4,
            0.95,
        ),
    ];
    let replay = CommitteeReplayReport {
        replay_id: "quality".to_string(),
        records,
        record_count: 3,
        source_summary: "official".to_string(),
        final_action_counts: BTreeMap::from([
            ("FinalDenied".to_string(), 1),
            ("FinalNoTrade".to_string(), 1),
            ("PaperApprove".to_string(), 1),
        ]),
        risk_denial_counts: BTreeMap::from([("DeniedByDefault".to_string(), 1)]),
        chair_decision_counts: BTreeMap::from([
            ("ApproveCandidate".to_string(), 1),
            ("NoTrade".to_string(), 1),
            ("Vetoed".to_string(), 1),
        ]),
        deterministic_fingerprint: "fp".to_string(),
        reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
    };
    let chair = vec![
        ChairDiagnosticsReport {
            decision_id: "1".to_string(),
            speaker_traces: Vec::new(),
            selected_speakers: vec!["trend_breakout_fast".to_string()],
            filtered_speakers: Vec::new(),
            cluster_counts: BTreeMap::new(),
            cluster_penalty_applied: false,
            contrarian_included: false,
            groupthink_risk: 0.7,
            disagreement_score: 0.1,
            uncertainty: 0.2,
            weighted_score: 0.1,
            final_decision: CommitteeDecision::NoTrade,
            diagnostic_status: ChairDiagnosticStatus::GroupthinkRisk,
            reason_codes: vec![ReasonCode::ChairDiagnosticsBuilt],
        },
        ChairDiagnosticsReport {
            decision_id: "2".to_string(),
            speaker_traces: Vec::new(),
            selected_speakers: vec!["trend_breakout_fast".to_string()],
            filtered_speakers: Vec::new(),
            cluster_counts: BTreeMap::new(),
            cluster_penalty_applied: false,
            contrarian_included: false,
            groupthink_risk: 0.1,
            disagreement_score: 0.8,
            uncertainty: 0.4,
            weighted_score: 0.2,
            final_decision: CommitteeDecision::Vetoed,
            diagnostic_status: ChairDiagnosticStatus::ExcessiveDisagreement,
            reason_codes: vec![ReasonCode::ChairDiagnosticsBuilt],
        },
        ChairDiagnosticsReport {
            decision_id: "3".to_string(),
            speaker_traces: Vec::new(),
            selected_speakers: vec!["trend_breakout_fast".to_string()],
            filtered_speakers: Vec::new(),
            cluster_counts: BTreeMap::new(),
            cluster_penalty_applied: false,
            contrarian_included: false,
            groupthink_risk: 0.1,
            disagreement_score: 0.2,
            uncertainty: 0.3,
            weighted_score: 0.5,
            final_decision: CommitteeDecision::ApproveCandidate,
            diagnostic_status: ChairDiagnosticStatus::Healthy,
            reason_codes: vec![ReasonCode::ChairDiagnosticsBuilt],
        },
    ];
    let risk = vec![
        RiskBridgeDiagnosticsReport {
            decision_id: "1".to_string(),
            committee_final_decision: "NoTrade".to_string(),
            risk_proposal_summary: "none".to_string(),
            risk_governor_decision: "Deny".to_string(),
            final_action: "FinalNoTrade".to_string(),
            veto_applied: true,
            denial_reason_codes: vec![ReasonCode::DeniedByDefault],
            emergency_stop_triggered: false,
            cooldown_triggered: false,
            data_quality_block: false,
            negative_edge_block: false,
            invalid_prediction_block: false,
            schema_mismatch_block: false,
            diagnostic_status: RiskBridgeDiagnosticStatus::RiskDeniedUnexpected,
            reason_codes: vec![ReasonCode::RiskBridgeDiagnosticsBuilt],
        },
        RiskBridgeDiagnosticsReport {
            decision_id: "2".to_string(),
            committee_final_decision: "Vetoed".to_string(),
            risk_proposal_summary: "none".to_string(),
            risk_governor_decision: "Cooldown".to_string(),
            final_action: "FinalDenied".to_string(),
            veto_applied: true,
            denial_reason_codes: vec![ReasonCode::DeniedByDefault],
            emergency_stop_triggered: false,
            cooldown_triggered: true,
            data_quality_block: false,
            negative_edge_block: false,
            invalid_prediction_block: false,
            schema_mismatch_block: false,
            diagnostic_status: RiskBridgeDiagnosticStatus::Cooldown,
            reason_codes: vec![ReasonCode::RiskBridgeDiagnosticsBuilt],
        },
        RiskBridgeDiagnosticsReport {
            decision_id: "3".to_string(),
            committee_final_decision: "ApproveCandidate".to_string(),
            risk_proposal_summary: "proposal".to_string(),
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
            diagnostic_status: RiskBridgeDiagnosticStatus::RiskPassed,
            reason_codes: vec![ReasonCode::RiskBridgeDiagnosticsBuilt],
        },
    ];
    let conflict = PersonaConflictMatrix {
        pairs: vec![PersonaConflictPair {
            persona_a: "a".to_string(),
            persona_b: "b".to_string(),
            same_stance_count: 1,
            opposite_stance_count: 1,
            disagreement_rate: 0.5,
            average_conviction_delta: 0.5,
            high_conflict_count: 0,
            reason_codes: vec![ReasonCode::PersonaConflictMatrixBuilt],
        }],
        most_aligned_pairs: Vec::new(),
        most_conflicted_pairs: Vec::new(),
        average_disagreement: 0.5,
        groupthink_frequency: 0.2,
        conflict_status: PersonaConflictStatus::HealthyDiversity,
        reason_codes: vec![ReasonCode::PersonaConflictMatrixBuilt],
    };
    let report = build_committee_decision_quality_report(
        &replay,
        &chair,
        &risk,
        &conflict,
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
    );
    assert_eq!(report.no_trade_ratio, 1.0 / 3.0);
    assert_eq!(report.risk_denial_ratio, 1.0 / 3.0);
    assert_eq!(report.hard_veto_ratio, 1.0 / 3.0);
    assert_eq!(report.cooldown_ratio, 1.0 / 3.0);
    assert_eq!(report.groupthink_warning_ratio, 1.0 / 3.0);
    assert_eq!(report.high_disagreement_ratio, 1.0 / 3.0);
}

#[test]
fn all_no_trade_and_fixture_quality_are_classified_conservatively() {
    let replay = CommitteeReplayReport {
        replay_id: "quality-no-trade".to_string(),
        records: vec![
            replay_record(
                CommitteeFinalAction::FinalNoTrade,
                CommitteeDecision::NoTrade,
                PersonaStance::NoTrade,
                0.1,
                0.85,
            );
            3
        ],
        record_count: 3,
        source_summary: "fixture".to_string(),
        final_action_counts: BTreeMap::from([("FinalNoTrade".to_string(), 3)]),
        risk_denial_counts: BTreeMap::new(),
        chair_decision_counts: BTreeMap::from([("NoTrade".to_string(), 3)]),
        deterministic_fingerprint: "fp".to_string(),
        reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
    };
    let chair = vec![
        ChairDiagnosticsReport {
            decision_id: "1".to_string(),
            speaker_traces: Vec::new(),
            selected_speakers: Vec::new(),
            filtered_speakers: Vec::new(),
            cluster_counts: BTreeMap::new(),
            cluster_penalty_applied: false,
            contrarian_included: false,
            groupthink_risk: 0.1,
            disagreement_score: 0.1,
            uncertainty: 0.2,
            weighted_score: 0.1,
            final_decision: CommitteeDecision::NoTrade,
            diagnostic_status: ChairDiagnosticStatus::Healthy,
            reason_codes: vec![ReasonCode::ChairDiagnosticsBuilt],
        };
        3
    ];
    let risk = vec![
        RiskBridgeDiagnosticsReport {
            decision_id: "1".to_string(),
            committee_final_decision: "NoTrade".to_string(),
            risk_proposal_summary: "none".to_string(),
            risk_governor_decision: "Deny".to_string(),
            final_action: "FinalNoTrade".to_string(),
            veto_applied: true,
            denial_reason_codes: vec![ReasonCode::DeniedByDefault],
            emergency_stop_triggered: false,
            cooldown_triggered: false,
            data_quality_block: false,
            negative_edge_block: false,
            invalid_prediction_block: false,
            schema_mismatch_block: false,
            diagnostic_status: RiskBridgeDiagnosticStatus::RiskDeniedUnexpected,
            reason_codes: vec![ReasonCode::RiskBridgeDiagnosticsBuilt],
        };
        3
    ];
    let conflict = PersonaConflictMatrix {
        pairs: Vec::new(),
        most_aligned_pairs: Vec::new(),
        most_conflicted_pairs: Vec::new(),
        average_disagreement: 0.0,
        groupthink_frequency: 0.0,
        conflict_status: PersonaConflictStatus::HealthyDiversity,
        reason_codes: vec![ReasonCode::PersonaConflictMatrixBuilt],
    };
    let all_no_trade = build_committee_decision_quality_report(
        &replay,
        &chair,
        &risk,
        &conflict,
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
    );
    assert_eq!(
        all_no_trade.quality_status,
        CommitteeDecisionQualityStatus::AllNoTrade
    );
    let fixture = build_committee_decision_quality_report(
        &replay,
        &chair,
        &risk,
        &conflict,
        &evidence(CommitteeEvidenceQualityStatus::FixtureOnlyEvidence),
    );
    assert_eq!(
        fixture.quality_status,
        CommitteeDecisionQualityStatus::FixtureOnly
    );
}

#[test]
fn decision_quality_is_deterministic() {
    let replay = CommitteeReplayReport {
        replay_id: "det".to_string(),
        records: vec![
            replay_record(
                CommitteeFinalAction::PaperApprove,
                CommitteeDecision::ApproveCandidate,
                PersonaStance::Approve,
                0.2,
                0.9,
            );
            3
        ],
        record_count: 3,
        source_summary: "official".to_string(),
        final_action_counts: BTreeMap::from([("PaperApprove".to_string(), 3)]),
        risk_denial_counts: BTreeMap::new(),
        chair_decision_counts: BTreeMap::from([("ApproveCandidate".to_string(), 3)]),
        deterministic_fingerprint: "fp".to_string(),
        reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
    };
    let chair = vec![
        ChairDiagnosticsReport {
            decision_id: "1".to_string(),
            speaker_traces: Vec::new(),
            selected_speakers: Vec::new(),
            filtered_speakers: Vec::new(),
            cluster_counts: BTreeMap::new(),
            cluster_penalty_applied: false,
            contrarian_included: false,
            groupthink_risk: 0.1,
            disagreement_score: 0.1,
            uncertainty: 0.2,
            weighted_score: 0.3,
            final_decision: CommitteeDecision::ApproveCandidate,
            diagnostic_status: ChairDiagnosticStatus::Healthy,
            reason_codes: vec![ReasonCode::ChairDiagnosticsBuilt],
        };
        3
    ];
    let risk = vec![
        RiskBridgeDiagnosticsReport {
            decision_id: "1".to_string(),
            committee_final_decision: "ApproveCandidate".to_string(),
            risk_proposal_summary: "proposal".to_string(),
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
            diagnostic_status: RiskBridgeDiagnosticStatus::RiskPassed,
            reason_codes: vec![ReasonCode::RiskBridgeDiagnosticsBuilt],
        };
        3
    ];
    let conflict = PersonaConflictMatrix {
        pairs: Vec::new(),
        most_aligned_pairs: Vec::new(),
        most_conflicted_pairs: Vec::new(),
        average_disagreement: 0.1,
        groupthink_frequency: 0.1,
        conflict_status: PersonaConflictStatus::HealthyDiversity,
        reason_codes: vec![ReasonCode::PersonaConflictMatrixBuilt],
    };
    let first = build_committee_decision_quality_report(
        &replay,
        &chair,
        &risk,
        &conflict,
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
    );
    let second = build_committee_decision_quality_report(
        &replay,
        &chair,
        &risk,
        &conflict,
        &evidence(CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable),
    );
    assert_eq!(first.to_text(), second.to_text());
}
