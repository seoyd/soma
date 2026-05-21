mod common;

use std::fs;

use serde_json::json;
use soma_zero::{
    SequenceDatasetReadinessConfig, SequenceDatasetReadinessRunner, SequenceDatasetReadinessStatus,
};

fn base_input() -> serde_json::Value {
    json!({
        "row_count": 4096,
        "official_row_count": 3584,
        "complete_row_count": 2048,
        "estimated_sequence_windows": 1024,
        "symbols": ["005930", "000660", "AAPL", "MSFT"],
        "horizons": [4, 8, 16],
        "window_lengths": [32, 64],
        "outcome_label_distribution": {"Win": 420, "Loss": 360, "NoTrade": 244},
        "feature_schema_locked": true,
        "no_lookahead_safe": true,
        "storage_estimate_bytes": 524288,
        "source_class": "official_non_crypto",
        "summary_derived_ratio": 0.10,
        "reason_codes": ["DeterministicPath"]
    })
}

fn config(name: &str, value: &serde_json::Value) -> SequenceDatasetReadinessConfig {
    let output_dir = common::sprint55_output_dir(name);
    let input_path = output_dir.join("input.json");
    fs::write(
        &input_path,
        serde_json::to_string_pretty(value).expect("json"),
    )
    .expect("write");
    SequenceDatasetReadinessConfig {
        readiness_id: name.to_string(),
        official_evidence_scaleout_paths: vec![input_path.display().to_string()],
        official_evidence_diversity_paths: Vec::new(),
        comparable_evidence_bundle_paths: Vec::new(),
        complete_row_bundle_paths: vec![input_path.display().to_string()],
        feature_schema_paths: vec![input_path.display().to_string()],
        candle_pack_paths: vec![input_path.display().to_string()],
        output_root: output_dir.display().to_string(),
        target_window_lengths: vec![32, 64],
        target_horizons: vec![4, 8, 16],
        min_sequence_windows: 256,
        min_symbols: 4,
        min_outcome_diversity: 3,
        max_summary_derived_ratio: 0.25,
        max_storage_bytes: 1_048_576,
        require_no_lookahead_safe: true,
        require_official_non_crypto: true,
        allow_crypto_only: false,
        allow_research_only: true,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

#[test]
fn insufficient_rows_require_more_rows() {
    let mut value = base_input();
    value["row_count"] = json!(180);
    value["estimated_sequence_windows"] = json!(120);
    let report = SequenceDatasetReadinessRunner::default()
        .run(&config("sequence-need-rows", &value))
        .expect("report");
    assert_eq!(
        report.readiness_status,
        SequenceDatasetReadinessStatus::NeedMoreRows
    );
}

#[test]
fn insufficient_symbols_require_more_symbols() {
    let mut value = base_input();
    value["symbols"] = json!(["005930", "AAPL"]);
    let report = SequenceDatasetReadinessRunner::default()
        .run(&config("sequence-need-symbols", &value))
        .expect("report");
    assert_eq!(
        report.readiness_status,
        SequenceDatasetReadinessStatus::NeedMoreSymbols
    );
}

#[test]
fn insufficient_outcome_labels_require_more_diversity() {
    let mut value = base_input();
    value["outcome_label_distribution"] = json!({"Win": 500, "Loss": 300});
    let report = SequenceDatasetReadinessRunner::default()
        .run(&config("sequence-need-labels", &value))
        .expect("report");
    assert_eq!(
        report.readiness_status,
        SequenceDatasetReadinessStatus::NeedMoreOutcomeLabels
    );
}

#[test]
fn missing_feature_schema_lock_is_blocking() {
    let mut value = base_input();
    value["feature_schema_locked"] = json!(false);
    let report = SequenceDatasetReadinessRunner::default()
        .run(&config("sequence-need-schema-lock", &value))
        .expect("report");
    assert_eq!(
        report.readiness_status,
        SequenceDatasetReadinessStatus::NeedFeatureSchemaLock
    );
}

#[test]
fn missing_no_lookahead_proof_is_blocking() {
    let mut value = base_input();
    value["no_lookahead_safe"] = json!(false);
    let report = SequenceDatasetReadinessRunner::default()
        .run(&config("sequence-need-no-lookahead", &value))
        .expect("report");
    assert_eq!(
        report.readiness_status,
        SequenceDatasetReadinessStatus::NeedNoLookaheadProof
    );
}

#[test]
fn storage_overflow_requires_budget_work() {
    let mut value = base_input();
    value["storage_estimate_bytes"] = json!(4_194_304usize);
    let report = SequenceDatasetReadinessRunner::default()
        .run(&config("sequence-need-storage", &value))
        .expect("report");
    assert_eq!(
        report.readiness_status,
        SequenceDatasetReadinessStatus::NeedStorageBudget
    );
}

#[test]
fn sufficient_fixture_returns_ready_for_export() {
    let report = SequenceDatasetReadinessRunner::default()
        .run(&config("sequence-ready", &base_input()))
        .expect("report");
    assert_eq!(
        report.readiness_status,
        SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport
    );
}

#[test]
fn yfinance_only_stays_research_only() {
    let mut value = base_input();
    value["official_row_count"] = json!(0);
    value["source_class"] = json!("yfinance_only");
    let report = SequenceDatasetReadinessRunner::default()
        .run(&config("sequence-research-only", &value))
        .expect("report");
    assert_eq!(
        report.readiness_status,
        SequenceDatasetReadinessStatus::ResearchOnly
    );
}

#[test]
fn sequence_readiness_is_deterministic() {
    let cfg = config("sequence-deterministic", &base_input());
    let left = SequenceDatasetReadinessRunner::default()
        .run(&cfg)
        .expect("left")
        .to_json_string()
        .expect("left json");
    let right = SequenceDatasetReadinessRunner::default()
        .run(&cfg)
        .expect("right")
        .to_json_string()
        .expect("right json");
    assert_eq!(left, right);
}
