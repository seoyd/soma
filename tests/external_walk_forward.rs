use soma_zero::{
    Candle, CandleSeries, ChairConfig, DatasetExportConfig, EvaluationMode,
    ExternalPredictionSignalConfig, ExternalPredictionSignalModel, FeatureSchema, GovernorConfig,
    ModelArtifactMeta, ModelKind, PredictionImportConfig, PredictionRow, Timeframe,
    WalkForwardConfig, WalkForwardEvaluator, prediction_frame_from_rows,
};

fn series() -> CandleSeries {
    CandleSeries {
        symbol: "EXT".to_string(),
        timeframe: Timeframe::FiveMinute,
        candles: (0..120)
            .map(|i| {
                let base = 100.0 + i as f64 * 0.3;
                Candle {
                    timestamp_ms: i as u64 * 300_000,
                    open: base,
                    high: base + 0.8,
                    low: base - 0.3,
                    close: base + 0.4,
                    volume: 1_200.0 + i as f64 * 25.0,
                    trade_value: Some((base + 0.4) * 1_200.0),
                    bid: Some(base + 0.38),
                    ask: Some(base + 0.42),
                    spread_bps: Some(2.0),
                }
            })
            .collect(),
    }
}

fn prediction_model(missing_last: bool) -> ExternalPredictionSignalModel {
    let evaluator = WalkForwardEvaluator::default();
    let config = WalkForwardConfig {
        train_window_bars: 40,
        validation_window_bars: Some(8),
        test_window_bars: 20,
        step_bars: 18,
        embargo_bars: 4,
        min_train_bars: 20,
        max_folds: Some(1),
        allow_partial_last_fold: false,
    };
    let split = evaluator.split(&series(), config);
    let dataset = evaluator.export_dataset(&series(), &split, &DatasetExportConfig::default());
    let schema = FeatureSchema::from_engine(&evaluator.feature_engine);
    let mut rows = dataset
        .rows
        .iter()
        .filter(|row| {
            matches!(
                row.split_kind,
                soma_zero::DatasetSplitKind::Validation | soma_zero::DatasetSplitKind::Test
            )
        })
        .map(|row| {
            PredictionRow::new(
                row.row_id.clone(),
                row.symbol.clone(),
                row.timestamp_ms,
                row.timeframe,
                row.fold_id,
                Some(row.split_kind),
                "ext-v0",
                0.95,
                0.05,
                0.1,
                0.01,
                0.99,
                0.0,
                20,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    if missing_last {
        rows.pop();
    }
    let meta = ModelArtifactMeta {
        model_id: "ext-v0".to_string(),
        model_kind: ModelKind::ExternalPredictionFile,
        created_at_ms: Some(1),
        feature_schema_version: schema.schema_version,
        feature_schema_hash: schema.checksum,
        training_window: None,
        validation_window: None,
        test_window: None,
        target_label_config: "triple".to_string(),
        cost_model_summary: "cost".to_string(),
        notes: None,
        reason_codes: vec![],
    };
    ExternalPredictionSignalModel {
        prediction_frame: prediction_frame_from_rows(
            meta,
            rows,
            &dataset,
            &schema,
            &PredictionImportConfig::default(),
        ),
        config: ExternalPredictionSignalConfig::default(),
    }
}

#[test]
fn walk_forward_evaluator_supports_external_prediction_mode_and_is_deterministic() {
    let mut evaluator = WalkForwardEvaluator::default();
    evaluator.external_signal_model = Some(prediction_model(false));
    let config = WalkForwardConfig {
        max_folds: Some(1),
        ..WalkForwardConfig::default()
    };

    let baseline = evaluator.evaluate_with_mode(&series(), config, EvaluationMode::BaselineSignal);
    let left = evaluator.evaluate_with_mode(&series(), config, EvaluationMode::ExternalPrediction);
    let right = evaluator.evaluate_with_mode(&series(), config, EvaluationMode::ExternalPrediction);

    assert_eq!(left, right);
    assert_eq!(
        baseline
            .folds
            .iter()
            .map(|fold| fold.fold_id)
            .collect::<Vec<_>>(),
        left.folds
            .iter()
            .map(|fold| fold.fold_id)
            .collect::<Vec<_>>()
    );
}

#[test]
fn missing_external_predictions_turn_conservative() {
    let mut evaluator = WalkForwardEvaluator::default();
    evaluator.external_signal_model = Some(prediction_model(true));
    let report = evaluator.evaluate_with_mode(
        &series(),
        WalkForwardConfig {
            max_folds: Some(1),
            ..WalkForwardConfig::default()
        },
        EvaluationMode::ExternalPrediction,
    );

    assert!(report.aggregate_metrics.decision_metrics.no_trade > 0);
}

#[test]
fn risk_governor_still_denies_risky_external_predictions() {
    let mut evaluator = WalkForwardEvaluator::default();
    evaluator.external_signal_model = Some(prediction_model(false));
    evaluator.chair.config = ChairConfig {
        strong_threshold: 0.0,
        weak_threshold: -1.0,
        ..ChairConfig::default()
    };
    evaluator.governor.config = GovernorConfig {
        min_expected_edge: 0.2,
        ..GovernorConfig::default()
    };
    let report = evaluator.evaluate_with_mode(
        &series(),
        WalkForwardConfig {
            max_folds: Some(1),
            ..WalkForwardConfig::default()
        },
        EvaluationMode::ExternalPrediction,
    );

    assert_eq!(report.aggregate_metrics.trade_metrics.total_trades, 0);
    assert!(report.aggregate_metrics.decision_metrics.no_trade > 0);
}
