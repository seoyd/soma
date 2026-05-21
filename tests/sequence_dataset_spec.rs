use std::collections::BTreeMap;
use std::fs;

use soma_zero::{
    BarrierHit, Candle, CandleSeries, DatasetExportConfig, DatasetFrame, DatasetRow,
    DatasetSplitKind, FeatureEngine, FeatureName, FeatureValue, ReasonCode, Regime,
    SequenceDatasetConfig, SequenceDatasetSpec, StorageBudget, Timeframe, TripleBarrierOutcome,
    build_sequence_row_refs, prior_window_features_unchanged,
};

mod common;

fn dataset_frame() -> DatasetFrame {
    DatasetFrame {
        feature_names: vec![FeatureName::Close, FeatureName::Volume],
        rows: (0..6)
            .map(|index| DatasetRow {
                row_id: format!("row-{index}"),
                symbol: "BTC-USDT".to_string(),
                timestamp_ms: index as u64 * 60_000,
                timeframe: Timeframe::OneMinute,
                fold_id: Some(0),
                split_kind: DatasetSplitKind::Train,
                regime: Regime::Range,
                data_quality_score: 1.0,
                feature_values: vec![
                    FeatureValue::Value(100.0 + index as f64),
                    FeatureValue::Value(1_000.0 + index as f64 * 10.0),
                ],
                label_outcome: Some(TripleBarrierOutcome::Win),
                label_net_return_pct: Some(0.01),
                label_gross_return_pct: Some(0.012),
                label_bars_held: Some(2),
                label_first_hit: Some(BarrierHit::TimeExpired),
                reason_codes: vec![],
            })
            .collect(),
        metadata: BTreeMap::new(),
    }
}

fn valid_config() -> SequenceDatasetConfig {
    SequenceDatasetConfig {
        window_size: 3,
        stride: 2,
        horizon_bars: 2,
        max_windows: 64,
        max_bytes: 1_024,
        storage_budget: StorageBudget {
            max_total_bytes: 1_024,
            ..StorageBudget::default()
        },
        ..SequenceDatasetConfig::default()
    }
}

fn series() -> CandleSeries {
    CandleSeries {
        symbol: "BTC-USDT".to_string(),
        timeframe: Timeframe::OneMinute,
        candles: (0..40)
            .map(|i| {
                let base = 100.0 + i as f64 * 0.2;
                Candle {
                    timestamp_ms: i as u64 * 60_000,
                    open: base,
                    high: base + 0.8,
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

#[test]
fn sequence_config_validation_rejects_zero_window_or_stride() {
    let invalid = SequenceDatasetConfig {
        window_size: 0,
        stride: 0,
        horizon_bars: 0,
        ..SequenceDatasetConfig::default()
    };

    assert!(
        invalid
            .validate()
            .contains(&ReasonCode::SequenceDatasetInvalid)
    );
}

#[test]
fn sequence_spec_estimates_windows_bytes_and_hash_deterministically() {
    let frame = dataset_frame();
    let config = valid_config();
    let left = SequenceDatasetSpec::from_dataset_frame(&frame, &config);
    let right = SequenceDatasetSpec::from_dataset_frame(&frame, &config);

    assert_eq!(left, right);
    assert_eq!(left.estimated_windows, 2);
    assert_eq!(left.estimated_bytes, 224);
    assert!(left.no_lookahead_safe);
    assert!(left.storage_budget_ok);
    assert_ne!(left.config.feature_schema_hash, 0);
}

#[test]
fn sequence_csv_parsing_uses_real_dataset_headers() {
    let frame = dataset_frame();
    let config = valid_config();
    let output_dir = common::output_dir("sequence-spec-csv");
    let csv_path = output_dir.join("dataset.csv");
    fs::write(
        &csv_path,
        frame.to_csv_string(&DatasetExportConfig::default()),
    )
    .expect("write dataset csv");

    let spec =
        SequenceDatasetSpec::from_dataset_csv_path(&csv_path, &config).expect("load sequence spec");

    assert_eq!(spec.estimated_windows, 2);
    assert_eq!(spec.estimated_bytes, 224);
    assert!(spec.no_lookahead_safe);
    assert_ne!(spec.config.feature_schema_hash, 0);
}

#[test]
fn sequence_storage_budget_overflow_is_reason_coded() {
    let frame = dataset_frame();
    let config = SequenceDatasetConfig {
        max_bytes: 100,
        storage_budget: StorageBudget {
            max_total_bytes: 100,
            ..StorageBudget::default()
        },
        ..valid_config()
    };

    let spec = SequenceDatasetSpec::from_dataset_frame(&frame, &config);

    assert!(!spec.storage_budget_ok);
    assert!(
        spec.reason_codes
            .contains(&ReasonCode::SequenceStorageBudgetExceeded)
    );
}

#[test]
fn sequence_row_refs_align_label_index_after_window_end() {
    let refs = build_sequence_row_refs(&dataset_frame(), &valid_config());

    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].start_index, 0);
    assert_eq!(refs[0].end_index, 2);
    assert_eq!(refs[0].label_index, 2);
    assert_eq!(refs[1].start_index, 2);
    assert_eq!(refs[1].end_index, 4);
    assert_eq!(refs[1].label_index, 4);
    assert_ne!(refs[0].feature_schema_hash, 0);
}

#[test]
fn future_candle_mutation_does_not_change_prior_window_features() {
    let base = series();
    let mut mutated = series();
    mutated.candles[25].close *= 1.5;
    mutated.candles[25].high *= 1.6;

    assert!(prior_window_features_unchanged(
        &FeatureEngine::default(),
        &base,
        &mutated,
        15,
        5,
    ));
}
