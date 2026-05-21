use soma_zero::{
    BarrierHit, DatasetFrame, DatasetRow, DatasetSplitKind, FeatureName, FeatureValue,
    ModelArtifactMeta, ModelKind, PredictionFrame, PredictionRow, PredictionValidationResult,
    ReasonCode, Regime, Timeframe, TripleBarrierOutcome, build_calibration_report,
};

fn dataset_rows() -> DatasetFrame {
    DatasetFrame {
        feature_names: vec![FeatureName::Close],
        rows: vec![
            DatasetRow {
                row_id: "row-1".to_string(),
                symbol: "CAL".to_string(),
                timestamp_ms: 1,
                timeframe: Timeframe::FiveMinute,
                fold_id: Some(0),
                split_kind: DatasetSplitKind::Test,
                regime: Regime::TrendUp,
                data_quality_score: 1.0,
                feature_values: vec![FeatureValue::Value(1.0)],
                label_outcome: Some(TripleBarrierOutcome::Win),
                label_net_return_pct: Some(0.02),
                label_gross_return_pct: Some(0.03),
                label_bars_held: Some(2),
                label_first_hit: Some(BarrierHit::TakeProfit),
                reason_codes: vec![],
            },
            DatasetRow {
                row_id: "row-2".to_string(),
                symbol: "CAL".to_string(),
                timestamp_ms: 2,
                timeframe: Timeframe::FiveMinute,
                fold_id: Some(0),
                split_kind: DatasetSplitKind::Test,
                regime: Regime::TrendDown,
                data_quality_score: 1.0,
                feature_values: vec![FeatureValue::Value(1.0)],
                label_outcome: Some(TripleBarrierOutcome::Loss),
                label_net_return_pct: Some(-0.01),
                label_gross_return_pct: Some(-0.008),
                label_bars_held: Some(2),
                label_first_hit: Some(BarrierHit::StopLoss),
                reason_codes: vec![],
            },
        ],
        metadata: Default::default(),
    }
}

fn prediction_frame() -> PredictionFrame {
    PredictionFrame {
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
        rows: vec![
            PredictionRow::new(
                "row-1",
                "CAL",
                1,
                Timeframe::FiveMinute,
                Some(0),
                Some(DatasetSplitKind::Test),
                "ext-v0",
                0.8,
                0.1,
                0.02,
                0.01,
                0.8,
                0.1,
                8,
            )
            .unwrap(),
            PredictionRow::new(
                "row-2",
                "CAL",
                2,
                Timeframe::FiveMinute,
                Some(0),
                Some(DatasetSplitKind::Test),
                "ext-v0",
                0.2,
                0.6,
                -0.01,
                0.02,
                0.6,
                0.4,
                8,
            )
            .unwrap(),
        ],
        schema_validation: PredictionValidationResult {
            valid: true,
            row_count: 2,
            missing_row_count: 0,
            extra_row_count: 0,
            schema_match: true,
            feature_schema_hash_match: true,
            invalid_probability_count: 0,
            nan_or_inf_count: 0,
            timestamp_mismatch_count: 0,
            reason_codes: vec![],
        },
        reason_codes: vec![],
    }
}

#[test]
fn calibration_report_is_deterministic_and_correct() {
    let left = build_calibration_report(&prediction_frame(), &dataset_rows(), Some(0));
    let right = build_calibration_report(&prediction_frame(), &dataset_rows(), Some(0));

    assert_eq!(left, right);
    assert_eq!(left.total_count, 2);
    assert!((left.brier_score - 0.04).abs() < 1e-9);
    assert_eq!(left.bins.len(), 5);
}

#[test]
fn empty_calibration_report_emits_reason_code() {
    let mut dataset = dataset_rows();
    dataset.rows.clear();
    let report = build_calibration_report(&prediction_frame(), &dataset, Some(0));
    assert_eq!(report.total_count, 0);
    assert!(report.reason_codes.contains(&ReasonCode::CalibrationEmpty));
}
