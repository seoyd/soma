#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use soma_zero::{
    CommitteeCounterfactualAuditConfig, CommitteeOfficialBenchmarkConfig,
    CommitteeOfficialBenchmarkRunner, CommitteeOutcomeCoverageConfig, CommitteeOutcomeLinker,
    CommitteeOutcomeLinkerConfig, CommitteeReferencePackConfig,
    CommitteeScenarioMaterializationLevel, CommitteeScenarioRow, CommitteeScenarioSet,
    CommitteeScenarioSourceKind, EvidenceSourceKind, OfficialCommitteePackSourceKind,
    OfficialCommitteeScenarioPackBuilder, OfficialCommitteeScenarioPackConfig,
    OfficialEvidenceReplicationConfig, OutcomeLinkedCommitteeScenarioPack, PersonaHorizon,
    ProviderMarket, ReasonCode, Regime, SufficiencyClosureConfig,
};

use super::common;

pub fn write_json(path: &Path, value: serde_json::Value) -> PathBuf {
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        }
    }
    fs::write(path, serde_json::to_string_pretty(&value).expect("json")).expect("write json");
    path.to_path_buf()
}

pub fn write_crypto_evidence_lane(name: &str) -> PathBuf {
    let dir = common::output_dir(&format!("{name}-crypto-evidence"));
    let path = dir.join("evidence_lane_btc_krw.json");
    write_json(
        &path,
        json!({
            "provenance": "official-crypto-public",
            "lane_reports": [
                {
                    "symbol": "BTC-KRW",
                    "market": "Crypto",
                    "data_quality_score": 0.91,
                    "expected_edge_after_cost": 0.012,
                    "expected_drawdown": 0.025
                },
                {
                    "symbol": "ETH-KRW",
                    "market": "Crypto",
                    "data_quality_score": 0.89,
                    "expected_edge_after_cost": 0.011,
                    "expected_drawdown": 0.024
                },
                {
                    "symbol": "XRP-KRW",
                    "market": "Crypto",
                    "data_quality_score": 0.88,
                    "expected_edge_after_cost": 0.010,
                    "expected_drawdown": 0.026
                }
            ]
        }),
    )
}

pub fn write_official_csv(name: &str, with_preflight: bool, rows: usize) -> PathBuf {
    let dir = common::output_dir(&format!("{name}-official-csv"));
    let path = dir.join("AAPL.csv");
    let mut text = "timestamp,open,high,low,close,volume\n".to_string();
    for index in 0..rows {
        text.push_str(&format!(
            "{},{},{},{},{},{}\n",
            1_700_000_000 + index as u64,
            100 + index,
            101 + index,
            99 + index,
            100 + index,
            1_000 + index
        ));
    }
    fs::write(&path, text).expect("write csv");
    if with_preflight {
        write_json(
            &dir.join("preflight_report.json"),
            json!({"preflight": true, "status": "ready"}),
        );
    }
    path
}

pub fn write_outcomes(name: &str, no_lookahead_safe: bool) -> PathBuf {
    let dir = common::output_dir(&format!("{name}-outcomes"));
    let path = dir.join("outcomes.json");
    write_json(
        &path,
        json!({
            "outcomes": [
                {
                    "outcome_id": "AAPL-0",
                    "symbol": "AAPL",
                    "timestamp_ms": 1700000000000u64,
                    "horizon_bars": 24,
                    "triple_barrier_label": "TakeProfit",
                    "net_return_pct": 0.040,
                    "cost_bps": 5.0,
                    "slippage_bps": 3.0,
                    "source_kind": "OfficialApiCollected",
                    "no_lookahead_safe": no_lookahead_safe
                },
                {
                    "outcome_id": "AAPL-1",
                    "symbol": "AAPL",
                    "timestamp_ms": 1700000000001u64,
                    "horizon_bars": 24,
                    "triple_barrier_label": "NoTradeCounterfactual",
                    "net_return_pct": -0.010,
                    "cost_bps": 5.0,
                    "slippage_bps": 2.0,
                    "source_kind": "OfficialApiCollected",
                    "no_lookahead_safe": no_lookahead_safe
                },
                {
                    "outcome_id": "AAPL-2",
                    "symbol": "AAPL",
                    "timestamp_ms": 1700000000002u64,
                    "horizon_bars": 24,
                    "triple_barrier_label": "RiskDeniedCounterfactual",
                    "net_return_pct": -0.020,
                    "cost_bps": 5.0,
                    "slippage_bps": 2.0,
                    "source_kind": "OfficialApiCollected",
                    "no_lookahead_safe": no_lookahead_safe
                }
            ]
        }),
    )
}

pub fn write_baselines(name: &str) -> PathBuf {
    let dir = common::output_dir(&format!("{name}-baselines"));
    let path = dir.join("baseline.json");
    write_json(
        &path,
        json!({
            "baseline_references": [
                {
                    "symbol": "AAPL",
                    "timestamp_ms": 1700000000000u64,
                    "horizon_bars": 24,
                    "baseline_action": "Approve",
                    "baseline_confidence": 0.75,
                    "baseline_expected_edge": 0.02
                },
                {
                    "symbol": "AAPL",
                    "timestamp_ms": 1700000000001u64,
                    "horizon_bars": 24,
                    "baseline_action": "NoTrade",
                    "baseline_confidence": 0.60,
                    "baseline_expected_edge": 0.0
                },
                {
                    "symbol": "AAPL",
                    "timestamp_ms": 1700000000002u64,
                    "horizon_bars": 24,
                    "baseline_action": "ReduceSize",
                    "baseline_confidence": 0.55,
                    "baseline_expected_edge": 0.01
                }
            ]
        }),
    )
}

pub fn write_externals(name: &str, prediction_schema_valid: bool) -> PathBuf {
    let dir = common::output_dir(&format!("{name}-externals"));
    let path = dir.join("external.json");
    write_json(
        &path,
        json!({
            "predictions": [
                {
                    "symbol": "AAPL",
                    "timestamp_ms": 1700000000000u64,
                    "horizon_bars": 24,
                    "external_action": "Approve",
                    "external_p_win": 0.61,
                    "external_confidence": 0.7,
                    "prediction_schema_valid": prediction_schema_valid
                }
            ]
        }),
    )
}

pub fn write_pack_config(name: &str, config: &OfficialCommitteeScenarioPackConfig) -> PathBuf {
    let path = common::output_dir(&format!("{name}-pack-config")).join("pack.toml");
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        }
    }
    fs::write(&path, config.to_toml_string().expect("pack toml")).expect("write pack toml");
    path
}

pub fn write_linker_config(name: &str, config: &CommitteeOutcomeLinkerConfig) -> PathBuf {
    let path = common::output_dir(&format!("{name}-linker-config")).join("linker.toml");
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        }
    }
    fs::write(&path, config.to_toml_string().expect("linker toml")).expect("write linker toml");
    path
}

pub fn write_benchmark_config(name: &str, config: &CommitteeOfficialBenchmarkConfig) -> PathBuf {
    let path = common::output_dir(&format!("{name}-benchmark-config")).join("benchmark.toml");
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        }
    }
    fs::write(&path, config.to_toml_string().expect("benchmark toml"))
        .expect("write benchmark toml");
    path
}

pub fn crypto_pack_config(name: &str) -> OfficialCommitteeScenarioPackConfig {
    OfficialCommitteeScenarioPackConfig {
        pack_id: format!("{name}-pack"),
        input_artifact_paths: vec![write_crypto_evidence_lane(name).display().to_string()],
        output_root: common::output_dir(&format!("{name}-pack-out"))
            .display()
            .to_string(),
        max_rows: 10,
        max_symbols: 3,
        max_bytes: 500_000,
        allow_crypto_only: true,
        require_provenance: true,
        require_preflight: true,
        ..OfficialCommitteeScenarioPackConfig::default()
    }
}

pub fn controlled_pack_config(
    name: &str,
    no_preflight: bool,
) -> OfficialCommitteeScenarioPackConfig {
    OfficialCommitteeScenarioPackConfig {
        pack_id: format!("{name}-pack"),
        input_artifact_paths: vec![
            write_official_csv(name, !no_preflight, 3)
                .display()
                .to_string(),
        ],
        output_root: common::output_dir(&format!("{name}-pack-out"))
            .display()
            .to_string(),
        max_rows: 10,
        max_symbols: 3,
        max_bytes: 500_000,
        allow_crypto_only: false,
        require_provenance: true,
        require_preflight: true,
        ..OfficialCommitteeScenarioPackConfig::default()
    }
}

pub fn yfinance_pack_config(name: &str) -> OfficialCommitteeScenarioPackConfig {
    OfficialCommitteeScenarioPackConfig {
        pack_id: format!("{name}-pack"),
        input_artifact_paths: vec!["virtual-yfinance".to_string()],
        allowed_source_kinds: vec![OfficialCommitteePackSourceKind::YFinanceResearch],
        output_root: common::output_dir(&format!("{name}-pack-out"))
            .display()
            .to_string(),
        allow_yfinance_research: true,
        allow_summary_derived_rows: true,
        require_preflight: false,
        require_provenance: false,
        ..OfficialCommitteeScenarioPackConfig::default()
    }
}

pub fn fixture_pack_config(name: &str) -> OfficialCommitteeScenarioPackConfig {
    OfficialCommitteeScenarioPackConfig {
        pack_id: format!("{name}-pack"),
        input_artifact_paths: vec!["virtual-fixture".to_string()],
        allowed_source_kinds: vec![OfficialCommitteePackSourceKind::Fixture],
        output_root: common::output_dir(&format!("{name}-pack-out"))
            .display()
            .to_string(),
        allow_fixture: true,
        allow_summary_derived_rows: true,
        require_preflight: false,
        require_provenance: false,
        ..OfficialCommitteeScenarioPackConfig::default()
    }
}

pub fn controlled_linker_config(
    name: &str,
    pack_path: &Path,
    no_lookahead_safe: bool,
) -> CommitteeOutcomeLinkerConfig {
    CommitteeOutcomeLinkerConfig {
        linker_id: format!("{name}-linker"),
        scenario_pack_path: Some(pack_path.display().to_string()),
        outcome_artifact_paths: vec![
            write_outcomes(name, no_lookahead_safe)
                .display()
                .to_string(),
        ],
        baseline_artifact_paths: vec![write_baselines(name).display().to_string()],
        external_prediction_paths: vec![write_externals(name, true).display().to_string()],
        output_root: common::output_dir(&format!("{name}-linker-out"))
            .display()
            .to_string(),
        strict_timestamp_match: true,
        max_timestamp_tolerance_ms: 0,
        require_same_symbol: true,
        require_same_horizon: true,
        reason_codes: vec![],
    }
}

pub fn controlled_benchmark_config(
    name: &str,
    pack_config_path: &Path,
    linker_config_path: &Path,
    require_core_check: bool,
) -> CommitteeOfficialBenchmarkConfig {
    CommitteeOfficialBenchmarkConfig {
        benchmark_id: name.to_string(),
        scenario_pack_config_path: Some(pack_config_path.display().to_string()),
        outcome_linker_config_path: Some(linker_config_path.display().to_string()),
        output_root: common::output_dir(&format!("{name}-benchmark-out"))
            .display()
            .to_string(),
        require_core_check,
        require_outcome_linked_rows: true,
        min_official_rows: 3,
        min_outcome_linked_rows: 3,
        min_baseline_linked_rows: 3,
        min_no_trade_counterfactuals: 1,
        min_risk_denial_counterfactuals: 1,
        max_summary_derived_ratio: 0.40,
        max_research_only_ratio: 0.0,
        max_fixture_ratio: 0.0,
        reason_codes: vec![],
        ..CommitteeOfficialBenchmarkConfig::default()
    }
}

pub fn write_candle_series(
    name: &str,
    symbol: &str,
    start_timestamp_ms: u64,
    drift: f64,
) -> PathBuf {
    let dir = common::output_dir(&format!("{name}-candles"));
    let path = dir.join(format!("{}_candles.json", symbol.to_ascii_lowercase()));
    let candles = (0..32)
        .map(|index| {
            let base = 100.0 + drift * index as f64;
            json!({
                "timestamp_ms": start_timestamp_ms + index as u64,
                "open": base,
                "high": base * 1.015,
                "low": base * 0.99,
                "close": base * (1.0 + drift / 100.0),
                "volume": 1000.0 + index as f64,
                "spread_bps": 4.0
            })
        })
        .collect::<Vec<_>>();
    write_json(
        &path,
        json!({
            "symbol": symbol,
            "timeframe": "OneDay",
            "candles": candles
        }),
    )
}

pub fn write_coverage_config(name: &str, config: &CommitteeOutcomeCoverageConfig) -> PathBuf {
    let path = common::output_dir(&format!("{name}-coverage-config")).join("coverage.toml");
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        }
    }
    fs::write(&path, config.to_toml_string().expect("coverage toml")).expect("write coverage");
    path
}

pub fn write_counterfactual_audit_config(
    name: &str,
    config: &CommitteeCounterfactualAuditConfig,
) -> PathBuf {
    let path = common::output_dir(&format!("{name}-counterfactual-config")).join("audit.toml");
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        }
    }
    fs::write(&path, config.to_toml_string().expect("audit toml")).expect("write audit");
    path
}

pub fn controlled_coverage_config(
    name: &str,
    benchmark_config_path: &Path,
    pack_config_path: &Path,
    candle_path: &Path,
) -> CommitteeOutcomeCoverageConfig {
    CommitteeOutcomeCoverageConfig {
        coverage_id: name.to_string(),
        official_benchmark_report_paths: vec![benchmark_config_path.display().to_string()],
        scenario_pack_paths: vec![pack_config_path.display().to_string()],
        candle_series_paths: vec![candle_path.display().to_string()],
        output_root: common::output_dir(&format!("{name}-coverage-out"))
            .display()
            .to_string(),
        max_rows: 20,
        max_symbols: 3,
        max_bytes: 500_000,
        require_official_rows: true,
        allow_crypto_only: false,
        allow_yfinance_research: false,
        allow_fixture: false,
        allow_estimated_counterfactuals: false,
        require_no_lookahead_safe: true,
        reason_codes: vec![],
        ..CommitteeOutcomeCoverageConfig::default()
    }
}

pub fn build_controlled_linked_pack(
    name: &str,
    no_lookahead_safe: bool,
) -> (
    soma_zero::OfficialCommitteeScenarioPack,
    OutcomeLinkedCommitteeScenarioPack,
) {
    let pack_config = controlled_pack_config(name, false);
    let pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&pack_config)
        .expect("pack");
    let pack_dir = common::output_dir(&format!("{name}-pack-store"));
    pack.write_to_dir(&pack_dir).expect("write pack");
    let linker_config = controlled_linker_config(
        name,
        &pack_dir.join("official_scenario_pack.json"),
        no_lookahead_safe,
    );
    let linked = CommitteeOutcomeLinker::default()
        .link(&pack, &linker_config)
        .expect("linked");
    (pack, linked)
}

pub fn build_controlled_benchmark_bundle(
    name: &str,
    no_lookahead_safe: bool,
) -> soma_zero::CommitteeOfficialBenchmarkBundle {
    let pack_config = controlled_pack_config(name, false);
    let pack_config_path = write_pack_config(name, &pack_config);
    let pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&pack_config)
        .expect("pack");
    let pack_dir = common::output_dir(&format!("{name}-benchmark-pack-store"));
    pack.write_to_dir(&pack_dir).expect("write pack");
    let linker_config = controlled_linker_config(
        name,
        &pack_dir.join("official_scenario_pack.json"),
        no_lookahead_safe,
    );
    let linker_config_path = write_linker_config(name, &linker_config);
    CommitteeOfficialBenchmarkRunner::default()
        .run_bundle(&controlled_benchmark_config(
            name,
            &pack_config_path,
            &linker_config_path,
            false,
        ))
        .expect("bundle")
}

pub fn scenario_row(
    name: &str,
    index: usize,
    symbol: &str,
    timestamp_ms: u64,
) -> CommitteeScenarioRow {
    CommitteeScenarioRow {
        scenario_row_id: format!("{name}-{index}"),
        symbol: symbol.to_string(),
        timestamp_ms,
        source_kind: CommitteeScenarioSourceKind::OfficialBenchmarkReport,
        evidence_source_kind: EvidenceSourceKind::OfficialApiCollected,
        market: ProviderMarket::USEquity,
        target_horizon: PersonaHorizon::Swing,
        feature_vector: None,
        regime: Regime::TrendUp,
        signal_summary: "approve".to_string(),
        data_quality_score: 0.92,
        spread_bps: Some(4.0),
        expected_edge_after_cost: 0.02,
        expected_drawdown: 0.01,
        risk_snapshot_summary: Some("stable".to_string()),
        provenance_summary: "official local controlled".to_string(),
        benchmark_status: Some("Ready".to_string()),
        baseline_signal_summary: Some("Approve".to_string()),
        external_prediction_summary: Some("Approve".to_string()),
        no_trade_counterfactual: None,
        risk_denial_counterfactual: None,
        outcome_reference: None,
        materialization_level: CommitteeScenarioMaterializationLevel::RowLevel,
        materialization_confidence: 1.0,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

pub fn write_scenario_set(name: &str, rows: Vec<CommitteeScenarioRow>) -> PathBuf {
    let dir = common::output_dir(&format!("{name}-scenario-set"));
    let path = dir.join("committee_scenario_set.json");
    let set = CommitteeScenarioSet {
        scenario_id: name.to_string(),
        row_count: rows.len(),
        official_row_count: rows
            .iter()
            .filter(|row| row.evidence_source_kind.readiness_eligible())
            .count(),
        research_only_row_count: rows
            .iter()
            .filter(|row| row.evidence_source_kind == EvidenceSourceKind::YFinanceResearch)
            .count(),
        fixture_row_count: rows
            .iter()
            .filter(|row| {
                matches!(
                    row.source_kind,
                    CommitteeScenarioSourceKind::Fixture
                        | CommitteeScenarioSourceKind::SyntheticTest
                )
            })
            .count(),
        skipped_row_count: 0,
        source_summary: "tests".to_string(),
        rows,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    fs::write(&path, set.to_json_string().expect("scenario json")).expect("write scenario set");
    path
}

pub fn controlled_reference_pack_config(name: &str) -> CommitteeReferencePackConfig {
    let pack_config = controlled_pack_config(name, false);
    let pack_config_path = write_pack_config(name, &pack_config);
    CommitteeReferencePackConfig {
        reference_pack_id: name.to_string(),
        scenario_pack_paths: vec![pack_config_path.display().to_string()],
        candle_series_paths: vec![
            write_candle_series(name, "AAPL", 1_700_000_000_000, 1.0)
                .display()
                .to_string(),
        ],
        baseline_reference_paths: vec![write_baselines(name).display().to_string()],
        external_prediction_paths: vec![write_externals(name, true).display().to_string()],
        output_root: common::output_dir(&format!("{name}-references-out"))
            .display()
            .to_string(),
        max_rows: 20,
        max_symbols: 3,
        max_bytes: 500_000,
        reason_codes: vec![ReasonCode::DeterministicPath],
        ..CommitteeReferencePackConfig::default()
    }
}

pub fn diagnostics_reference_pack_config(name: &str) -> CommitteeReferencePackConfig {
    let mut config = controlled_reference_pack_config(name);
    config.allow_estimated_references = true;
    config.timestamp_tolerance_ms = 10;
    config.candle_series_paths = vec![
        write_candle_series(name, "AAPL", 1_700_000_000_005, 1.0)
            .display()
            .to_string(),
    ];
    config
}

pub fn write_reference_pack_config(name: &str, config: &CommitteeReferencePackConfig) -> PathBuf {
    let path =
        common::output_dir(&format!("{name}-reference-pack-config")).join("reference_pack.toml");
    fs::write(&path, config.to_toml_string().expect("reference pack toml"))
        .expect("write reference pack");
    path
}

pub fn write_sufficiency_closure_config(name: &str, config: &SufficiencyClosureConfig) -> PathBuf {
    let path = common::output_dir(&format!("{name}-closure-config")).join("closure.toml");
    fs::write(&path, config.to_toml_string().expect("closure toml")).expect("write closure");
    path
}

pub fn write_replication_config(name: &str, config: &OfficialEvidenceReplicationConfig) -> PathBuf {
    let path = common::output_dir(&format!("{name}-replication-config")).join("replication.toml");
    fs::write(&path, config.to_toml_string().expect("replication toml"))
        .expect("write replication");
    path
}

pub fn write_official_csv_bundle(
    name: &str,
    symbol: &str,
    rows: usize,
    include_preflight: bool,
    include_provenance: bool,
    official: bool,
) -> PathBuf {
    let dir = common::output_dir(&format!("{name}-official-bundle"));
    let csv_path = dir.join(format!("{symbol}.csv"));
    let mut text = "timestamp,open,high,low,close,volume\n".to_string();
    for index in 0..rows {
        text.push_str(&format!(
            "{},{},{},{},{},{}\n",
            1_700_000_000_000u64 + index as u64,
            100.0 + index as f64,
            101.0 + index as f64,
            99.0 + index as f64,
            100.5 + index as f64,
            1_000.0 + index as f64
        ));
    }
    fs::write(&csv_path, text).expect("write official bundle csv");
    if include_preflight {
        write_json(
            &dir.join("preflight_report.json"),
            json!({
                "onboarding_id": name,
                "final_status": "ReadyForRealEvidence",
                "checks": [],
                "symbol": symbol,
                "timeframe": "OneDay"
            }),
        );
    }
    if include_provenance {
        write_json(
            &dir.join("official_provenance.json"),
            json!({
                "source_kind": if official { "OfficialApiCollected" } else { "RealLocal" },
                "source_label": if official { "official-api-collected" } else { "controlled-local" },
                "official_provider": official,
                "downloaded_by_soma": official,
                "local_path": csv_path.display().to_string()
            }),
        );
    }
    csv_path
}
