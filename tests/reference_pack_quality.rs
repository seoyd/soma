mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    CandleAlignmentOverallStatus, CandleAlignmentReport, CommitteeReferencePackConfig,
    GeneratedCommitteeReference, GeneratedCommitteeReferencePack, GeneratedReferenceKind,
    GeneratedReferenceSource, GeneratedReferenceStatus, ReferencePackQualityStatus,
    build_reference_pack_quality_report,
};

fn pack_with_counts(
    name: &str,
    outcome: usize,
    baseline: usize,
    no_trade: usize,
    risk_denied: usize,
    diagnostic: usize,
    fixture: bool,
) -> GeneratedCommitteeReferencePack {
    let mut row = official_committee_support::scenario_row(name, 0, "AAPL", 1_700_000_000_000);
    if fixture {
        row.source_kind = soma_zero::CommitteeScenarioSourceKind::Fixture;
        row.evidence_source_kind = soma_zero::EvidenceSourceKind::TestFixture;
    }
    let scenario_rows = vec![row.clone()];
    let mut references = Vec::new();
    for index in 0..outcome {
        references.push(GeneratedCommitteeReference {
            reference_id: format!("o-{index}"),
            scenario_row_id: row.scenario_row_id.clone(),
            reference_kind: GeneratedReferenceKind::TripleBarrierOutcome,
            status: GeneratedReferenceStatus::Generated,
            outcome_reference: Some(soma_zero::CommitteeOutcomeReference {
                outcome_id: format!("o-{index}"),
                decision_id: None,
                symbol: row.symbol.clone(),
                timestamp_ms: row.timestamp_ms,
                horizon_bars: 24,
                triple_barrier_label: soma_zero::CommitteeTripleBarrierLabel::TakeProfit,
                net_return_pct: Some(0.01),
                max_favorable_excursion_pct: Some(0.01),
                max_adverse_excursion_pct: Some(0.0),
                cost_bps: 5.0,
                slippage_bps: 2.0,
                source_kind: row.evidence_source_kind,
                no_lookahead_safe: true,
                reason_codes: vec![],
            }),
            baseline_reference: None,
            external_reference: None,
            no_trade_counterfactual: None,
            risk_denied_counterfactual: None,
            generated_from: GeneratedReferenceSource::LocalCandleSeries,
            official_readiness_eligible: !fixture,
            diagnostic_only: diagnostic > 0,
            reason_codes: vec![],
        });
    }
    for index in 0..baseline {
        references.push(GeneratedCommitteeReference {
            reference_id: format!("b-{index}"),
            scenario_row_id: row.scenario_row_id.clone(),
            reference_kind: GeneratedReferenceKind::BaselineAction,
            status: GeneratedReferenceStatus::Generated,
            outcome_reference: None,
            baseline_reference: Some(soma_zero::CommitteeBaselineReference {
                baseline_action: soma_zero::CommitteeBaselineAction::Approve,
                baseline_confidence: Some(0.7),
                baseline_expected_edge: Some(0.01),
                baseline_reason_codes: vec![],
                reason_codes: vec![],
            }),
            external_reference: None,
            no_trade_counterfactual: None,
            risk_denied_counterfactual: None,
            generated_from: GeneratedReferenceSource::DeterministicBaselinePolicy,
            official_readiness_eligible: false,
            diagnostic_only: diagnostic > 0,
            reason_codes: vec![],
        });
    }
    for index in 0..no_trade {
        references.push(GeneratedCommitteeReference {
            reference_id: format!("n-{index}"),
            scenario_row_id: row.scenario_row_id.clone(),
            reference_kind: GeneratedReferenceKind::NoTradeCounterfactual,
            status: GeneratedReferenceStatus::Generated,
            outcome_reference: None,
            baseline_reference: None,
            external_reference: None,
            no_trade_counterfactual: Some(soma_zero::CommitteeCounterfactualRecord {
                counterfactual_id: format!("n-{index}"),
                scenario_row_id: row.scenario_row_id.clone(),
                counterfactual_type: soma_zero::CommitteeCounterfactualType::NoTrade,
                build_status: soma_zero::CounterfactualBuildStatus::Built,
                triple_barrier_label: Some(
                    soma_zero::CommitteeTripleBarrierLabel::NoTradeCounterfactual,
                ),
                net_return_pct: Some(0.0),
                avoided_loss_value: None,
                missed_gain_value: None,
                max_favorable_excursion_pct: Some(0.0),
                max_adverse_excursion_pct: Some(0.0),
                cost_bps: 5.0,
                slippage_bps: 2.0,
                no_lookahead_safe: true,
                diagnostic_only: diagnostic > 0,
                reason_codes: vec![],
            }),
            risk_denied_counterfactual: None,
            generated_from: GeneratedReferenceSource::LocalCandleSeries,
            official_readiness_eligible: !fixture,
            diagnostic_only: diagnostic > 0,
            reason_codes: vec![],
        });
    }
    for index in 0..risk_denied {
        references.push(GeneratedCommitteeReference {
            reference_id: format!("r-{index}"),
            scenario_row_id: row.scenario_row_id.clone(),
            reference_kind: GeneratedReferenceKind::RiskDeniedCounterfactual,
            status: GeneratedReferenceStatus::Generated,
            outcome_reference: None,
            baseline_reference: None,
            external_reference: None,
            no_trade_counterfactual: None,
            risk_denied_counterfactual: Some(soma_zero::CommitteeCounterfactualRecord {
                counterfactual_id: format!("r-{index}"),
                scenario_row_id: row.scenario_row_id.clone(),
                counterfactual_type: soma_zero::CommitteeCounterfactualType::RiskDenied,
                build_status: soma_zero::CounterfactualBuildStatus::Built,
                triple_barrier_label: Some(
                    soma_zero::CommitteeTripleBarrierLabel::RiskDeniedCounterfactual,
                ),
                net_return_pct: Some(0.0),
                avoided_loss_value: None,
                missed_gain_value: None,
                max_favorable_excursion_pct: Some(0.0),
                max_adverse_excursion_pct: Some(0.0),
                cost_bps: 5.0,
                slippage_bps: 2.0,
                no_lookahead_safe: true,
                diagnostic_only: diagnostic > 0,
                reason_codes: vec![],
            }),
            generated_from: GeneratedReferenceSource::LocalCandleSeries,
            official_readiness_eligible: !fixture,
            diagnostic_only: diagnostic > 0,
            reason_codes: vec![],
        });
    }
    GeneratedCommitteeReferencePack::new(
        name.to_string(),
        scenario_rows,
        references,
        CandleAlignmentReport {
            records: vec![],
            matched_count: 1,
            unmatched_count: 0,
            exact_match_count: 1,
            tolerance_match_count: 0,
            missing_series_count: 0,
            missing_timestamp_count: 0,
            wrong_symbol_count: 0,
            insufficient_future_bars_count: 0,
            no_lookahead_rejected_count: 0,
            alignment_status: CandleAlignmentOverallStatus::HealthyAlignment,
            reason_codes: vec![],
        },
        vec![],
    )
}

#[test]
fn reference_pack_quality_reports_conservative_statuses_and_is_deterministic() {
    let config = CommitteeReferencePackConfig::default();
    assert_eq!(
        build_reference_pack_quality_report(&config, &pack_with_counts("q1", 0, 1, 1, 1, 0, false))
            .quality_status,
        ReferencePackQualityStatus::NeedMoreOutcomeReferences
    );
    assert_eq!(
        build_reference_pack_quality_report(&config, &pack_with_counts("q2", 1, 0, 1, 1, 0, false))
            .quality_status,
        ReferencePackQualityStatus::NeedMoreBaselineReferences
    );
    assert_eq!(
        build_reference_pack_quality_report(&config, &pack_with_counts("q3", 1, 1, 0, 1, 0, false))
            .quality_status,
        ReferencePackQualityStatus::NeedMoreNoTradeCounterfactuals
    );
    assert_eq!(
        build_reference_pack_quality_report(&config, &pack_with_counts("q4", 1, 1, 1, 0, 0, false))
            .quality_status,
        ReferencePackQualityStatus::NeedMoreRiskDeniedCounterfactuals
    );

    let mut diagnostic_pack = pack_with_counts("q5", 1, 1, 1, 1, 4, false);
    diagnostic_pack.diagnostic_only_count = 4;
    assert_eq!(
        build_reference_pack_quality_report(&config, &diagnostic_pack).quality_status,
        ReferencePackQualityStatus::TooManyDiagnosticOnlyReferences
    );

    let first = build_reference_pack_quality_report(
        &CommitteeReferencePackConfig {
            allow_controlled_fixture_references: true,
            ..CommitteeReferencePackConfig::default()
        },
        &pack_with_counts("q6", 1, 1, 1, 1, 0, true),
    );
    let second = build_reference_pack_quality_report(
        &CommitteeReferencePackConfig {
            allow_controlled_fixture_references: true,
            ..CommitteeReferencePackConfig::default()
        },
        &pack_with_counts("q6", 1, 1, 1, 1, 0, true),
    );
    assert_eq!(first, second);
    assert_eq!(
        first.quality_status,
        ReferencePackQualityStatus::HealthyReferencePack
    );
}
