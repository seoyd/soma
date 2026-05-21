use soma_zero::{
    ChairDecisionKind, ChairOutput, ExternalPredictionSignalConfig, ExternalPredictionSignalModel,
    GovernorConfig, MarketSnapshot, ModelArtifactMeta, ModelKind, PredictionFrame, PredictionRow,
    PredictionValidationResult, RiskDecisionKind, RiskGovernor, RiskSnapshot, Side, Timeframe,
    TradeProposal,
};

fn model(valid: bool) -> ExternalPredictionSignalModel {
    let row = PredictionRow::new(
        "row-1",
        "SIG",
        1,
        Timeframe::FiveMinute,
        Some(0),
        Some(soma_zero::DatasetSplitKind::Test),
        "ext-v0",
        0.8,
        0.1,
        0.03,
        0.01,
        0.9,
        0.05,
        8,
    )
    .expect("row");
    ExternalPredictionSignalModel {
        prediction_frame: PredictionFrame {
            model_meta: ModelArtifactMeta {
                model_id: "ext-v0".to_string(),
                model_kind: ModelKind::ExternalPredictionFile,
                created_at_ms: Some(1),
                feature_schema_version: 1,
                feature_schema_hash: 1,
                training_window: None,
                validation_window: None,
                test_window: None,
                target_label_config: "triple_barrier".to_string(),
                cost_model_summary: "cost".to_string(),
                notes: None,
                reason_codes: vec![],
            },
            rows: vec![row],
            schema_validation: PredictionValidationResult {
                valid,
                row_count: 1,
                missing_row_count: 0,
                extra_row_count: 0,
                schema_match: valid,
                feature_schema_hash_match: valid,
                invalid_probability_count: 0,
                nan_or_inf_count: 0,
                timestamp_mismatch_count: 0,
                reason_codes: vec![],
            },
            reason_codes: vec![],
        },
        config: ExternalPredictionSignalConfig::default(),
    }
}

#[test]
fn external_prediction_returns_signal_output_when_row_exists() {
    let signal = model(true).signal_for("SIG", 1, Timeframe::FiveMinute, Some(0), Some("row-1"));
    assert_eq!(signal.source, "external_prediction:ext-v0");
    assert_eq!(signal.p_win, 0.8);
}

#[test]
fn missing_or_invalid_predictions_become_conservative_no_trade() {
    let missing = model(true).signal_for("SIG", 2, Timeframe::FiveMinute, Some(0), Some("missing"));
    assert_eq!(missing.no_trade_probability, 1.0);
    assert_eq!(missing.source, "external_prediction_missing");

    let invalid = model(false).signal_for("SIG", 1, Timeframe::FiveMinute, Some(0), Some("row-1"));
    assert_eq!(invalid.no_trade_probability, 1.0);
    assert_eq!(invalid.source, "external_prediction_invalid_frame");
}

#[test]
fn same_prediction_input_is_deterministic() {
    let model = model(true);
    let left = model.signal_for("SIG", 1, Timeframe::FiveMinute, Some(0), Some("row-1"));
    let right = model.signal_for("SIG", 1, Timeframe::FiveMinute, Some(0), Some("row-1"));
    assert_eq!(left, right);
}

#[test]
fn external_signal_still_cannot_bypass_risk_governor() {
    let signal = model(true).signal_for("SIG", 1, Timeframe::FiveMinute, Some(0), Some("row-1"));
    let governor = RiskGovernor {
        config: GovernorConfig {
            min_expected_edge: 0.2,
            ..GovernorConfig::default()
        },
    };
    let decision = governor.evaluate(
        &MarketSnapshot {
            symbol: "SIG".to_string(),
            timestamp_ms: 1,
            price: 100.0,
            bid: 99.9,
            ask: 100.1,
            spread_bps: 2.0,
            volume: 1_000.0,
            trade_value: 100_000.0,
            volatility: 0.02,
            regime: soma_zero::Regime::TrendUp,
            data_quality_score: 1.0,
        },
        &RiskSnapshot {
            daily_pnl_pct: 0.0,
            consecutive_losses: 0,
            current_positions_count: 0,
            total_exposure_pct: 0.0,
            symbol_exposure_pct: 0.0,
            api_health_score: 1.0,
            data_quality_score: 1.0,
        },
        Some(&TradeProposal {
            symbol: "SIG".to_string(),
            side: Side::Long,
            quantity_hint: 1.0,
            entry_price_hint: 100.0,
            stop_loss: Some(99.0),
            take_profit: Some(110.0),
            max_slippage_bps: 2.0,
            expected_edge_after_cost: signal.expected_return,
            confidence: signal.confidence,
            source_chair_output: ChairOutput {
                selected_speakers: vec!["p1".to_string()],
                lead_speaker: "p1".to_string(),
                forced_contrarian: false,
                council_score: 1.0,
                disagreement_score: 0.0,
                groupthink_risk: 0.0,
                size_multiplier: 1.0,
                decision: ChairDecisionKind::ApproveCandidate,
                reason_codes: vec![],
            },
        }),
        1,
    );

    assert_eq!(decision.kind, RiskDecisionKind::Deny);
}
