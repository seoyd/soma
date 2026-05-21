use std::path::{Path, PathBuf};

use soma_zero::{
    AssetClass, BacktestSimulator, BaselineSignalModel, CandleCsvConfig, CandleCsvLoader,
    FeatureEngine, MarketVenue, ReasonCode, RegimeClassifier, SymbolRegistry, SymbolSpec,
    WalkForwardConfig, WalkForwardEvaluator,
};

#[test]
fn valid_local_csv_to_feature_engine_works() {
    let loaded = load_valid_fixture();
    let frame = FeatureEngine::default().build_frame(&loaded.series);
    assert_eq!(frame.rows.len(), loaded.series.len());
}

#[test]
fn valid_local_csv_to_walk_forward_evaluator_works() {
    let loaded = load_valid_fixture();
    let report = WalkForwardEvaluator::default().evaluate(
        &loaded.series,
        WalkForwardConfig {
            train_window_bars: 5,
            validation_window_bars: Some(2),
            test_window_bars: 3,
            step_bars: 2,
            embargo_bars: 0,
            min_train_bars: 5,
            max_folds: Some(2),
            allow_partial_last_fold: false,
        },
    );
    assert!(!report.folds.is_empty());
}

#[test]
fn low_quality_local_csv_yields_deny_or_no_trade_heavy_behavior() {
    let loaded = CandleCsvLoader::default()
        .load_from_path(
            &fixture_path("generic_ohlcv_gaps.csv"),
            &CandleCsvConfig::default(),
        )
        .expect("gap fixture");
    let result = BacktestSimulator::default().run(&loaded.series);
    assert!(result.denied_trades + result.no_trades >= result.executed_trades);
}

#[test]
fn no_real_broker_path_exists_for_loaded_data() {
    let loaded = load_valid_fixture();
    let result = BacktestSimulator::default().run(&loaded.series);
    assert!(
        result
            .reason_codes
            .contains(&ReasonCode::PaperExecutionOnly)
    );
}

#[test]
fn no_live_network_api_call_exists() {
    let error = CandleCsvLoader::default()
        .load_from_path(
            Path::new("https://example.com/data.csv"),
            &CandleCsvConfig::default(),
        )
        .expect_err("local only");
    assert_eq!(error.reason_codes, vec![ReasonCode::LocalFileOnly]);
}

#[test]
fn no_runtime_llm_path_exists() {
    let loaded = load_valid_fixture();
    let feature_engine = FeatureEngine::default();
    let regime_classifier = RegimeClassifier::default();
    let baseline = BaselineSignalModel::default();
    let features = feature_engine.build_at(&loaded.series, 6);
    let regime =
        regime_classifier.classify(&features, loaded.series.lookback_window(6, 6).unwrap());
    let signal = baseline.evaluate(
        &features,
        &regime,
        &BacktestSimulator::default().config.cost_model,
    );
    assert_eq!(signal.source, "baseline_rule_v0");
    assert!(!signal.source.to_ascii_lowercase().contains("llm"));
}

#[test]
fn same_fixture_input_produces_same_report() {
    let first =
        WalkForwardEvaluator::default().evaluate(&load_valid_fixture().series, pipeline_config());
    let second =
        WalkForwardEvaluator::default().evaluate(&load_valid_fixture().series, pipeline_config());
    assert_eq!(first.aggregate_metrics, second.aggregate_metrics);
}

fn load_valid_fixture() -> soma_zero::LoadedCandleData {
    let mut registry = SymbolRegistry::default();
    registry
        .register_symbol(SymbolSpec::new(
            "btc-usdt",
            MarketVenue::Binance,
            AssetClass::Crypto,
        ))
        .expect("register symbol");
    let loader = CandleCsvLoader {
        registry,
        ..CandleCsvLoader::default()
    };
    let config = CandleCsvConfig {
        symbol: "btc-usdt".to_string(),
        ..CandleCsvConfig::default()
    };
    loader
        .load_from_path(&fixture_path("generic_ohlcv_valid.csv"), &config)
        .expect("valid fixture")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("market_data")
        .join(name)
}

fn pipeline_config() -> WalkForwardConfig {
    WalkForwardConfig {
        train_window_bars: 5,
        validation_window_bars: Some(2),
        test_window_bars: 3,
        step_bars: 2,
        embargo_bars: 0,
        min_train_bars: 5,
        max_folds: Some(2),
        allow_partial_last_fold: false,
    }
}
