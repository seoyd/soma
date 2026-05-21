use soma_zero::{
    BarrierHit, CostModel, ReasonCode, TripleBarrierOutcome, TripleBarrierResult,
    evaluate_no_trade_counterfactual,
};

fn result(outcome: TripleBarrierOutcome, net_return_pct: f64) -> TripleBarrierResult {
    TripleBarrierResult {
        outcome,
        first_hit: match outcome {
            TripleBarrierOutcome::Win => BarrierHit::TakeProfit,
            TripleBarrierOutcome::Loss => BarrierHit::StopLoss,
            TripleBarrierOutcome::Neutral => BarrierHit::TimeExpired,
            TripleBarrierOutcome::NoData => BarrierHit::NoData,
        },
        entry_index: 0,
        exit_index: 1,
        entry_price: 100.0,
        exit_price: 99.0,
        gross_return_pct: net_return_pct,
        net_return_pct,
        max_favorable_excursion_pct: 0.02,
        max_adverse_excursion_pct: 0.01,
        bars_held: 1,
        reason_codes: vec![],
    }
}

#[test]
fn round_trip_cost_is_positive() {
    let cost = CostModel {
        fee_bps: 2.0,
        slippage_bps: 3.0,
        spread_bps: Some(1.0),
        min_cost_bps: None,
    };
    assert!(cost.estimate_round_trip_cost_bps() > 0.0);
}

#[test]
fn negative_expected_edge_after_cost_is_detected() {
    let cost = CostModel {
        fee_bps: 5.0,
        slippage_bps: 5.0,
        spread_bps: Some(4.0),
        min_cost_bps: None,
    };
    assert!(cost.expected_edge_after_cost(0.001) < 0.0);
}

#[test]
fn no_trade_gets_positive_avoided_loss_score() {
    let evaluation = evaluate_no_trade_counterfactual(
        Some(&result(TripleBarrierOutcome::Loss, -0.02)),
        false,
        soma_zero::NoTradeScoreConfig::default(),
    );
    assert!(evaluation.avoided_loss_score > 0.0);
    assert_eq!(evaluation.missed_gain_penalty, 0.0);
    assert!(
        evaluation
            .reason_codes
            .contains(&ReasonCode::PositiveSilenceValue)
    );
}

#[test]
fn no_trade_gets_small_missed_gain_penalty() {
    let evaluation = evaluate_no_trade_counterfactual(
        Some(&result(TripleBarrierOutcome::Win, 0.03)),
        false,
        soma_zero::NoTradeScoreConfig::default(),
    );
    assert!(evaluation.avoided_loss_score == 0.0);
    assert!(evaluation.missed_gain_penalty < 0.0);
    assert!(evaluation.missed_gain_penalty > -0.01);
    assert!(
        evaluation
            .reason_codes
            .contains(&ReasonCode::NegativeSilenceValue)
    );
}

#[test]
fn neutral_hypothetical_stays_near_zero() {
    let evaluation = evaluate_no_trade_counterfactual(
        Some(&result(TripleBarrierOutcome::Neutral, 0.0)),
        false,
        soma_zero::NoTradeScoreConfig::default(),
    );
    assert_eq!(evaluation.avoided_loss_score, 0.0);
    assert_eq!(evaluation.missed_gain_penalty, 0.0);
}

#[test]
fn risk_denial_avoiding_loss_gets_defensive_attribution() {
    let evaluation = evaluate_no_trade_counterfactual(
        Some(&result(TripleBarrierOutcome::Loss, -0.04)),
        true,
        soma_zero::NoTradeScoreConfig::default(),
    );
    assert!(evaluation.avoided_loss_score > 0.0);
    assert!(
        evaluation
            .reason_codes
            .contains(&ReasonCode::DefensiveAttribution)
    );
}

#[test]
fn risk_denial_missing_gain_records_opportunity_cost_only() {
    let evaluation = evaluate_no_trade_counterfactual(
        Some(&result(TripleBarrierOutcome::Win, 0.04)),
        true,
        soma_zero::NoTradeScoreConfig::default(),
    );
    assert!(evaluation.avoided_loss_score == 0.0);
    assert!(evaluation.missed_gain_penalty < 0.0);
    assert!(
        evaluation
            .reason_codes
            .contains(&ReasonCode::OpportunityCostRecorded)
    );
}
