#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use encoding_rs::EUC_KR;
use soma_zero::{
    AssetClass, CandleCsvFormat, CandleCsvLoader, ChairConfig, CostModel, DataProvenance,
    DataValidationConfig, DatasetBundleConfig, DatasetEntry, DatasetExportConfig,
    DatasetOutputFormat, DatasetSplitKind, EvidenceSourceKind, ExperimentConfig,
    ExperimentMatrixConfig, ExperimentMode, ExperimentVariant, ExperimentVariantOverrides,
    FeatureConfig, FeatureSchema, GovernorConfig, LocalDataOnboardingConfig, MarketVenue,
    ModelArtifactMeta, ModelKind, NoTradeScoreConfig, PredictionRow, PreflightReport,
    PreflightValidator, RealEvidenceClosureConfig, RegimeClassifierConfig, ResearchCampaignConfig,
    Timeframe, TripleBarrierConfig, WalkForwardConfig, WalkForwardEvaluator,
    prediction_frame_from_rows, prediction_frame_to_csv_string,
};

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("market_data")
        .join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint10-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    if let Err(err) = fs::create_dir_all(&path) {
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    }
    path
}

pub fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

pub fn sprint52_data_path(name: &str) -> PathBuf {
    example_path("sprint52_data").join(name)
}

pub fn sprint52_output_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint52-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    if let Err(err) = fs::create_dir_all(&path) {
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    }
    path
}

pub fn sprint53_data_path(name: &str) -> PathBuf {
    example_path("sprint53_data").join(name)
}

pub fn sprint53_output_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint53-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    if let Err(err) = fs::create_dir_all(&path) {
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    }
    path
}

pub fn sprint55_data_path(name: &str) -> PathBuf {
    example_path("sprint55_data").join(name)
}

pub fn sprint55_output_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint55-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    if let Err(err) = fs::create_dir_all(&path) {
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    }
    path
}

pub fn sprint57_data_path(name: &str) -> PathBuf {
    example_path("sprint57_data").join(name)
}

pub fn sprint57_output_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint57-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    if let Err(err) = fs::create_dir_all(&path) {
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    }
    path
}

pub fn baseline_config(name: &str, fixture: &str) -> ExperimentConfig {
    let mut config = ExperimentConfig::baseline_only(
        name,
        "BTC-USDT",
        fixture_path(fixture).display().to_string(),
        Timeframe::OneMinute,
        output_dir(name).display().to_string(),
    );
    config.walk_forward_config.train_window_bars = 5;
    config.walk_forward_config.validation_window_bars = Some(2);
    config.walk_forward_config.test_window_bars = 3;
    config.walk_forward_config.step_bars = 2;
    config.walk_forward_config.embargo_bars = 0;
    config.walk_forward_config.min_train_bars = 5;
    config.walk_forward_config.max_folds = Some(2);
    config.feature_config.min_required_bars = 5;
    config.triple_barrier_config.horizon_bars = 2;
    config
}

pub fn dataset_config(name: &str, fixture: &str) -> ExperimentConfig {
    let mut config = baseline_config(name, fixture);
    config.mode = ExperimentMode::DatasetExportOnly;
    config
}

pub fn dataset_entry(dataset_id: &str, fixture: &str, enabled: bool) -> DatasetEntry {
    DatasetEntry {
        dataset_id: dataset_id.to_string(),
        symbol: "BTC-USDT".to_string(),
        data_path: fixture_path(fixture).display().to_string(),
        csv_format: CandleCsvFormat::GenericOhlcv,
        timeframe: Timeframe::OneMinute,
        resample_to: Some(Timeframe::FiveMinute),
        venue: MarketVenue::Generic,
        asset_class: AssetClass::Crypto,
        provenance: None,
        enabled,
        tags: vec!["fixture".to_string()],
        expected_min_rows: Some(5),
        notes: None,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

pub fn real_local_test_entry(dataset_id: &str, fixture: &str) -> DatasetEntry {
    let mut entry = dataset_entry(dataset_id, fixture, true);
    entry.resample_to = None;
    entry.provenance = Some(DataProvenance {
        source_kind: EvidenceSourceKind::RealLocal,
        source_label: format!("test-{dataset_id}"),
        provider_label: None,
        upstream_label: None,
        local_path: Some(fixture_path(fixture).display().to_string()),
        generated_by: None,
        user_supplied: true,
        downloaded_by_soma: false,
        remote_url_present: false,
        official_provider: Some(false),
        affiliated_or_endorsed: Some(false),
        intended_use: Some("controlled test local data".to_string()),
        readiness_eligible: Some(true),
        benchmark_eligible: Some(true),
        license_note: None,
        notes: Some("controlled test override".to_string()),
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    });
    entry.tags.push("test-real-local".to_string());
    entry
}

pub fn synthetic_entry(dataset_id: &str, fixture: &str) -> DatasetEntry {
    let mut entry = dataset_entry(dataset_id, fixture, true);
    entry.resample_to = None;
    entry.provenance = Some(DataProvenance {
        source_kind: EvidenceSourceKind::SyntheticFixture,
        source_label: format!("synthetic-{dataset_id}"),
        provider_label: None,
        upstream_label: None,
        local_path: Some(fixture_path(fixture).display().to_string()),
        generated_by: Some("fixture".to_string()),
        user_supplied: false,
        downloaded_by_soma: false,
        remote_url_present: false,
        official_provider: Some(false),
        affiliated_or_endorsed: Some(false),
        intended_use: Some("synthetic fixture".to_string()),
        readiness_eligible: Some(false),
        benchmark_eligible: Some(true),
        license_note: None,
        notes: Some("synthetic fixture".to_string()),
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    });
    entry.tags.push("synthetic".to_string());
    entry
}

pub fn real_evidence_config(name: &str, entries: Vec<DatasetEntry>) -> RealEvidenceClosureConfig {
    RealEvidenceClosureConfig {
        closure_id: name.to_string(),
        dataset_bundle_config_path: None,
        real_dataset_entries: entries,
        source_sprint15_report_path: Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("soma_evidence_closure")
                .join("sprint15_evidence_closure")
                .join("evidence_closure_report.json")
                .display()
                .to_string(),
        ),
        output_root: output_dir(&format!("{name}-real")).display().to_string(),
        evidence_store_path: output_dir(&format!("{name}-real-store"))
            .display()
            .to_string(),
        min_real_local_datasets: 1,
        min_real_local_outcome_records: 20,
        min_real_local_comparable_variants: 2,
        allow_synthetic_for_pipeline_smoke: true,
        allow_synthetic_for_readiness: false,
        continue_on_failure: true,
        strict_data_quality: true,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

pub fn onboarding_config(name: &str, fixture: &str) -> LocalDataOnboardingConfig {
    LocalDataOnboardingConfig {
        onboarding_id: name.to_string(),
        input_path: fixture_path(fixture).display().to_string(),
        output_root: output_dir(&format!("{name}-onboarding"))
            .display()
            .to_string(),
        symbol: Some("BTC-USDT".to_string()),
        venue: Some(MarketVenue::Generic),
        asset_class: Some(AssetClass::Crypto),
        timeframe: Some(Timeframe::OneMinute),
        csv_format_hint: None,
        custom_column_map: None,
        source_kind: None,
        user_supplied: true,
        source_label: None,
        strict: true,
        allow_format_autodetect: true,
        allow_sort_repair: false,
        allow_duplicate_drop: false,
        min_rows_for_preflight: 5,
        target_min_outcomes: 1,
        target_min_comparable_variants: 1,
        target_min_usable_datasets: 1,
        walk_forward_config: Some(WalkForwardConfig {
            train_window_bars: 5,
            validation_window_bars: Some(2),
            test_window_bars: 3,
            step_bars: 2,
            embargo_bars: 0,
            min_train_bars: 5,
            max_folds: Some(2),
            allow_partial_last_fold: false,
        }),
        triple_barrier_config: Some(TripleBarrierConfig {
            take_profit_pct: 0.02,
            stop_loss_pct: 0.01,
            horizon_bars: 2,
            fee_bps: 2.0,
            slippage_bps: 2.0,
            side: soma_zero::Side::Long,
            use_high_low_intrabar: true,
        }),
        cost_model: Some(CostModel {
            fee_bps: 2.0,
            slippage_bps: 2.0,
            spread_bps: Some(2.0),
            min_cost_bps: None,
        }),
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

pub fn run_preflight(config: &LocalDataOnboardingConfig) -> PreflightReport {
    PreflightValidator::default().run(config)
}

pub fn write_temp_csv(name: &str, contents: &str) -> PathBuf {
    let path = output_dir(&format!("{name}-tmp")).join(format!("{name}.csv"));
    fs::write(&path, contents).expect("write temp csv");
    path
}

pub fn write_cp949_temp_csv(name: &str, contents: &str) -> PathBuf {
    let path = output_dir(&format!("{name}-tmp")).join(format!("{name}.csv"));
    let (encoded, _, _) = EUC_KR.encode(contents);
    fs::write(&path, encoded.as_ref()).expect("write cp949 temp csv");
    path
}

pub fn ensure_sprint15_report() {
    let report_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("soma_evidence_closure")
        .join("sprint15_evidence_closure")
        .join("evidence_closure_report.json");
    if report_path.exists() {
        return;
    }
    let status = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "evidence-close",
            "--config",
            "examples/soma_evidence_closure.toml",
        ])
        .status()
        .expect("run sprint15 evidence-close");
    assert!(status.success());
}

pub fn baseline_variant(variant_id: &str, enabled: bool) -> ExperimentVariant {
    ExperimentVariant {
        variant_id: variant_id.to_string(),
        mode: ExperimentMode::BaselineOnly,
        overrides: ExperimentVariantOverrides {
            timeframe: Some(Timeframe::OneMinute),
            resample_to: Some(Timeframe::FiveMinute),
            ..ExperimentVariantOverrides::default()
        },
        enabled,
        tags: vec!["baseline".to_string()],
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

pub fn compare_variant(variant_id: &str, prediction_csv_path: String) -> ExperimentVariant {
    ExperimentVariant {
        variant_id: variant_id.to_string(),
        mode: ExperimentMode::TrainAndCompare,
        overrides: ExperimentVariantOverrides {
            timeframe: Some(Timeframe::OneMinute),
            resample_to: Some(Timeframe::FiveMinute),
            prediction_csv_path: Some(prediction_csv_path),
            run_python_training: Some(false),
            ..ExperimentVariantOverrides::default()
        },
        enabled: true,
        tags: vec!["compare".to_string()],
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

pub fn batch_matrix(
    name: &str,
    entries: Vec<DatasetEntry>,
    variants: Vec<ExperimentVariant>,
) -> ExperimentMatrixConfig {
    ExperimentMatrixConfig {
        matrix_id: name.to_string(),
        dataset_bundle: DatasetBundleConfig {
            bundle_id: format!("{name}-bundle"),
            entries,
            default_data_validation_config: DataValidationConfig {
                expected_step_ms: Some(60_000),
                max_gap_count: 1_000_000,
                ..DataValidationConfig::default()
            },
            default_feature_config: FeatureConfig {
                min_required_bars: 5,
                ..FeatureConfig::default()
            },
            default_regime_config: RegimeClassifierConfig::default(),
            default_chair_config: ChairConfig::default(),
            default_walk_forward_config: WalkForwardConfig {
                train_window_bars: 5,
                validation_window_bars: Some(2),
                test_window_bars: 3,
                step_bars: 2,
                embargo_bars: 0,
                min_train_bars: 5,
                max_folds: Some(2),
                allow_partial_last_fold: false,
            },
            default_triple_barrier_config: TripleBarrierConfig {
                take_profit_pct: 0.02,
                stop_loss_pct: 0.01,
                horizon_bars: 2,
                fee_bps: 2.0,
                slippage_bps: 2.0,
                side: soma_zero::Side::Long,
                use_high_low_intrabar: true,
            },
            default_cost_model: CostModel {
                fee_bps: 2.0,
                slippage_bps: 2.0,
                spread_bps: Some(2.0),
                min_cost_bps: None,
            },
            default_no_trade_score_config: NoTradeScoreConfig::default(),
            default_risk_config: GovernorConfig::default(),
            output_root: output_dir(name).display().to_string(),
            reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
        },
        variants,
        continue_on_failure: true,
        require_all_pass: false,
        deterministic_run_id: Some(format!("{name}-seed")),
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

pub fn campaign_config(name: &str, matrix_paths: Vec<String>) -> ResearchCampaignConfig {
    ResearchCampaignConfig {
        campaign_id: name.to_string(),
        description: Some(format!("{name} campaign")),
        matrix_config_paths: matrix_paths,
        embedded_matrices: Vec::new(),
        output_root: output_dir(&format!("{name}-campaign"))
            .display()
            .to_string(),
        evidence_store_path: output_dir(&format!("{name}-evidence"))
            .display()
            .to_string(),
        run_id: Some(format!("{name}-seed")),
        created_at_ms: Some(42),
        ..ResearchCampaignConfig::default()
    }
}

pub fn perfect_prediction_csv(name: &str, fixture: &str) -> String {
    let config = baseline_config(name, fixture);
    let loader = CandleCsvLoader::default();
    let loaded = loader
        .load_from_path(&fixture_path(fixture), &config.build_csv_config())
        .expect("load fixture");
    let evaluator = build_evaluator_for_tests(&config);
    let split = evaluator.split(&loaded.series, config.walk_forward_config);
    let dataset = evaluator.export_dataset(
        &loaded.series,
        &split,
        &DatasetExportConfig {
            include_labels: true,
            include_metadata: true,
            include_reason_codes: true,
            output_format: DatasetOutputFormat::Csv,
        },
    );
    let schema = FeatureSchema::from_engine(&evaluator.feature_engine);
    let rows = dataset
        .rows
        .iter()
        .filter(|row| {
            matches!(
                row.split_kind,
                DatasetSplitKind::Validation | DatasetSplitKind::Test
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
                format!("model_{name}"),
                if row.label_outcome == Some(soma_zero::TripleBarrierOutcome::Win) {
                    0.80
                } else {
                    0.20
                },
                0.20,
                row.label_net_return_pct.unwrap_or(0.0),
                0.01,
                0.80,
                0.10,
                config.triple_barrier_config.horizon_bars as u32,
            )
            .expect("prediction row")
        })
        .collect::<Vec<_>>();
    let frame = prediction_frame_from_rows(
        ModelArtifactMeta {
            model_id: format!("model_{name}"),
            model_kind: ModelKind::ExternalPredictionFile,
            created_at_ms: None,
            feature_schema_version: schema.schema_version,
            feature_schema_hash: schema.checksum,
            training_window: None,
            validation_window: None,
            test_window: None,
            target_label_config: "label_config".to_string(),
            cost_model_summary: "cost_model".to_string(),
            notes: None,
            reason_codes: vec![],
        },
        rows,
        &dataset,
        &schema,
        &soma_zero::PredictionImportConfig {
            require_feature_schema_match: true,
            require_row_alignment: true,
            min_confidence: None,
            max_missing_rows: 0,
            input_format: soma_zero::PredictionInputFormat::Csv,
        },
    );
    prediction_frame_to_csv_string(&frame)
}

fn build_evaluator_for_tests(config: &ExperimentConfig) -> WalkForwardEvaluator {
    let mut evaluator = WalkForwardEvaluator::default();
    evaluator.feature_engine.config = config.feature_config.clone();
    evaluator.regime_classifier.config = config.regime_config;
    evaluator.governor.config = config.risk_config;
    evaluator.triple_barrier_config = config.triple_barrier_config;
    evaluator.cost_model = config.cost_model;
    evaluator.full_auto = config.full_auto;
    evaluator
}

pub fn sprint54_data_path(name: &str) -> PathBuf {
    example_path("sprint54_data").join(name)
}

pub fn sprint54_output_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint54-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    if let Err(err) = fs::create_dir_all(&path) {
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    }
    path
}
