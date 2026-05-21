use soma_zero::{
    Candle, CandleSeries, DatasetExportConfig, FeatureSchema, ModelArtifactMeta, ModelKind,
    PredictionImportConfig, PredictionRow, ReasonCode, Timeframe, WalkForwardConfig,
    WalkForwardEvaluator, prediction_frame_from_csv_string, prediction_frame_from_rows,
    prediction_frame_to_csv_string,
};

fn series() -> CandleSeries {
    CandleSeries {
        symbol: "PRED".to_string(),
        timeframe: Timeframe::FiveMinute,
        candles: (0..90)
            .map(|i| {
                let base = 100.0 + i as f64 * 0.2;
                Candle {
                    timestamp_ms: i as u64 * 300_000,
                    open: base,
                    high: base + 0.7,
                    low: base - 0.4,
                    close: base + 0.3,
                    volume: 1_000.0 + i as f64 * 20.0,
                    trade_value: Some((base + 0.3) * 1_000.0),
                    bid: Some(base + 0.28),
                    ask: Some(base + 0.32),
                    spread_bps: Some(2.0),
                }
            })
            .collect(),
    }
}

fn dataset_context() -> (soma_zero::DatasetFrame, FeatureSchema, ModelArtifactMeta) {
    let evaluator = WalkForwardEvaluator::default();
    let split = evaluator.split(
        &series(),
        WalkForwardConfig {
            train_window_bars: 30,
            validation_window_bars: Some(10),
            test_window_bars: 20,
            step_bars: 20,
            embargo_bars: 4,
            min_train_bars: 20,
            max_folds: Some(1),
            allow_partial_last_fold: false,
        },
    );
    let dataset = evaluator.export_dataset(&series(), &split, &DatasetExportConfig::default());
    let schema = FeatureSchema::from_engine(&evaluator.feature_engine);
    let meta = ModelArtifactMeta {
        model_id: "ext-v0".to_string(),
        model_kind: ModelKind::ExternalPredictionFile,
        created_at_ms: Some(1),
        feature_schema_version: schema.schema_version,
        feature_schema_hash: schema.checksum,
        training_window: None,
        validation_window: None,
        test_window: None,
        target_label_config: "triple_barrier_v0".to_string(),
        cost_model_summary: "cost_v0".to_string(),
        notes: None,
        reason_codes: vec![],
    };
    (dataset, schema, meta)
}

fn aligned_rows(dataset: &soma_zero::DatasetFrame) -> Vec<PredictionRow> {
    dataset
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
                0.7,
                0.2,
                0.02,
                0.01,
                0.8,
                0.1,
                8,
            )
            .expect("valid row")
        })
        .collect()
}

#[test]
fn prediction_row_construction_and_validation_work() {
    assert!(
        PredictionRow::new(
            "row-1",
            "PRED",
            1,
            Timeframe::FiveMinute,
            Some(0),
            Some(soma_zero::DatasetSplitKind::Test),
            "ext-v0",
            0.7,
            0.2,
            0.02,
            0.01,
            0.8,
            0.1,
            8
        )
        .is_ok()
    );

    let error = PredictionRow::new(
        "row-1",
        "PRED",
        1,
        Timeframe::FiveMinute,
        Some(0),
        Some(soma_zero::DatasetSplitKind::Test),
        "ext-v0",
        1.2,
        0.2,
        0.02,
        0.01,
        0.8,
        0.1,
        8,
    )
    .expect_err("invalid p_win");
    assert!(error.contains(&ReasonCode::InvalidProbability));
}

#[test]
fn prediction_csv_round_trips_deterministically() {
    let (dataset, schema, meta) = dataset_context();
    let frame = prediction_frame_from_rows(
        meta.clone(),
        aligned_rows(&dataset),
        &dataset,
        &schema,
        &PredictionImportConfig::default(),
    );

    let csv = prediction_frame_to_csv_string(&frame);
    let imported = prediction_frame_from_csv_string(
        &csv,
        meta,
        &dataset,
        &schema,
        &PredictionImportConfig::default(),
    );

    assert_eq!(
        prediction_frame_to_csv_string(&frame),
        prediction_frame_to_csv_string(&imported)
    );
    assert!(imported.schema_validation.valid);
}

#[test]
fn missing_required_column_and_alignment_issues_are_reported() {
    let (dataset, schema, meta) = dataset_context();
    let csv = "row_id,symbol,timestamp_ms,timeframe,fold_id,split_kind,model_id,p_stop,expected_return,expected_drawdown,confidence,no_trade_probability,horizon_bars,reason_codes\n";
    let frame = prediction_frame_from_csv_string(
        csv,
        meta.clone(),
        &dataset,
        &schema,
        &PredictionImportConfig::default(),
    );
    assert!(!frame.schema_validation.valid);
    assert!(
        frame
            .schema_validation
            .reason_codes
            .contains(&ReasonCode::MissingRequiredColumn)
    );

    let mut rows = aligned_rows(&dataset);
    rows.pop();
    rows.push(
        PredictionRow::new(
            "row-extra",
            "PRED",
            999,
            Timeframe::FiveMinute,
            Some(0),
            Some(soma_zero::DatasetSplitKind::Test),
            "ext-v0",
            0.7,
            0.2,
            0.02,
            0.01,
            0.8,
            0.1,
            8,
        )
        .expect("extra row"),
    );
    let misaligned = prediction_frame_from_rows(
        meta,
        rows,
        &dataset,
        &schema,
        &PredictionImportConfig::default(),
    );
    assert!(!misaligned.schema_validation.valid);
    assert!(misaligned.schema_validation.missing_row_count > 0);
    assert!(misaligned.schema_validation.extra_row_count > 0);
}

#[test]
fn strict_schema_mismatch_blocks_prediction_frame() {
    let (dataset, schema, mut meta) = dataset_context();
    meta.feature_schema_hash += 1;
    let frame = prediction_frame_from_rows(
        meta,
        aligned_rows(&dataset),
        &dataset,
        &schema,
        &PredictionImportConfig::default(),
    );

    assert!(!frame.schema_validation.valid);
    assert!(
        frame
            .schema_validation
            .reason_codes
            .contains(&ReasonCode::PredictionSchemaMismatch)
    );
}
