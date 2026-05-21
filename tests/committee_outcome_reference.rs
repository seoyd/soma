use soma_zero::{
    CommitteeBaselineAction, CommitteeExternalReference, CommitteeOutcomeReference,
    CommitteeTripleBarrierLabel, EvidenceSourceKind,
};

fn reference(
    label: CommitteeTripleBarrierLabel,
    no_lookahead_safe: bool,
) -> CommitteeOutcomeReference {
    CommitteeOutcomeReference {
        outcome_id: "outcome".to_string(),
        decision_id: None,
        symbol: "AAPL".to_string(),
        timestamp_ms: 1,
        horizon_bars: 24,
        triple_barrier_label: label,
        net_return_pct: Some(0.05),
        max_favorable_excursion_pct: Some(0.06),
        max_adverse_excursion_pct: Some(-0.02),
        cost_bps: 5.0,
        slippage_bps: 2.0,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        no_lookahead_safe,
        reason_codes: vec![],
    }
}

#[test]
fn triple_barrier_labels_and_costs_are_supported() {
    assert!(reference(CommitteeTripleBarrierLabel::TakeProfit, true).benchmark_eligible());
    assert!(reference(CommitteeTripleBarrierLabel::StopLoss, true).benchmark_eligible());
    assert!(reference(CommitteeTripleBarrierLabel::TimeExpired, true).benchmark_eligible());
    assert!(
        reference(CommitteeTripleBarrierLabel::NoTradeCounterfactual, true)
            .no_trade_counterfactual()
    );
    assert!(
        reference(CommitteeTripleBarrierLabel::RiskDeniedCounterfactual, true)
            .risk_denial_counterfactual()
    );
    let adjusted = reference(CommitteeTripleBarrierLabel::TakeProfit, true)
        .cost_adjusted_return_pct()
        .expect("adjusted");
    assert!((adjusted - 0.0493).abs() < 1e-9);
}

#[test]
fn no_lookahead_unsafe_reference_is_not_benchmark_eligible() {
    assert!(!reference(CommitteeTripleBarrierLabel::TakeProfit, false).benchmark_eligible());
}

#[test]
fn baseline_and_external_reference_helpers_work() {
    assert_eq!(
        CommitteeBaselineAction::from_summary("ApproveCandidate"),
        CommitteeBaselineAction::Approve
    );
    assert_eq!(
        CommitteeExternalReference {
            external_action: Some("ReduceSize".to_string()),
            external_p_win: Some(0.55),
            external_confidence: Some(0.7),
            prediction_schema_valid: true,
            reason_codes: vec![],
        }
        .action_as_baseline_action(),
        CommitteeBaselineAction::ReduceSize
    );
}
