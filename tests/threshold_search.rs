use soma_zero::{
    BarrierHit, DatasetFrame, DatasetRow, DatasetSplitKind, FeatureName, FeatureValue,
    OptimizeMetric, PredictionFrame, PredictionRow, PredictionValidationResult, Regime,
    ThresholdSearchConfig, Timeframe, TripleBarrierOutcome, search_thresholds,
};

fn dataset(split_kind: DatasetSplitKind) -> DatasetFrame {
    DatasetFrame {
        feature_names: vec![FeatureName::Close],
        rows: vec![
            DatasetRow {
                row_id: "row-1".to_string(),
                symbol: "TH".to_string(),
                timestamp_ms: 1,
                timeframe: Timeframe::FiveMinute,
                fold_id: Some(0),
                split_kind,
                regime: Regime::TrendUp,
                data_quality_score: 1.0,
                feature_values: vec![FeatureValue::Value(1.0)],
                label_outcome: Some(TripleBarrierOutcome::Win),
                label_net_return_pct: Some(0.03),
                label_gross_return_pct: Some(0.035),
                label_bars_held: Some(2),
                label_first_hit: Some(BarrierHit::TakeProfit),
                reason_codes: vec![],
            },
            DatasetRow {
                row_id: "row-2".to_string(),
                symbol: "TH".to_string(),
                timestamp_ms: 2,
                timeframe: Timeframe::FiveMinute,
                fold_id: Some(0),
                split_kind,
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

fn frame() -> PredictionFrame {
    PredictionFrame {
        model_meta: soma_zero::ModelArtifactMeta {
            model_id: "ext-v0".to_string(),
            model_kind: soma_zero::ModelKind::ExternalPredictionFile,
            created_at_ms: Some(1),
            feature_schema_version: 1,
            feature_schema_hash: 1,
            training_window: None,
            validation_window: None,
            test_window: None,
            target_label_config: "triple".to_string(),
            cost_model_summary: "cost".to_string(),
            notes: None,
            reason_codes: vec![],
        },
        rows: vec![
            PredictionRow::new(
                "row-1",
                "TH",
                1,
                Timeframe::FiveMinute,
                Some(0),
                Some(DatasetSplitKind::Validation),
                "ext-v0",
                0.8,
                0.1,
                0.03,
                0.01,
                0.8,
                0.1,
                8,
            )
            .unwrap(),
            PredictionRow::new(
                "row-2",
                "TH",
                2,
                Timeframe::FiveMinute,
                Some(0),
                Some(DatasetSplitKind::Validation),
                "ext-v0",
                0.4,
                0.4,
                0.0,
                0.02,
                0.4,
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
fn threshold_search_is_deterministic_and_selects_best_validation_candidate() {
    let config = ThresholdSearchConfig {
        p_win_thresholds: vec![0.5, 0.7],
        p_stop_thresholds: vec![0.2, 0.5],
        confidence_thresholds: vec![0.5],
        no_trade_thresholds: vec![0.2, 0.5],
        min_expected_return_thresholds: vec![0.0, 0.02],
        max_drawdown_constraint: Some(0.2),
        min_sample_count: 1,
        optimize_metric: OptimizeMetric::NetReturn,
        validation_only: true,
    };
    let left = search_thresholds(0, &dataset(DatasetSplitKind::Validation), &frame(), &config);
    let right = search_thresholds(0, &dataset(DatasetSplitKind::Validation), &frame(), &config);

    assert_eq!(left, right);
    assert!(left.best_candidate.is_some());
}

#[test]
fn no_validation_rows_marks_search_research_only_and_too_few_samples_rejects() {
    let report = search_thresholds(
        0,
        &dataset(DatasetSplitKind::Test),
        &frame(),
        &ThresholdSearchConfig {
            p_win_thresholds: vec![0.9],
            p_stop_thresholds: vec![0.1],
            confidence_thresholds: vec![0.9],
            no_trade_thresholds: vec![0.1],
            min_expected_return_thresholds: vec![0.05],
            max_drawdown_constraint: None,
            min_sample_count: 3,
            optimize_metric: OptimizeMetric::NetReturn,
            validation_only: true,
        },
    );

    assert!(
        report
            .reason_codes
            .contains(&soma_zero::ReasonCode::ThresholdResearchOnly)
    );
    assert!(
        report
            .candidates
            .iter()
            .all(|candidate| !candidate.accepted)
    );
}
