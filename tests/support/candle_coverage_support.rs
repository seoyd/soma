#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use soma_zero::{
    AssetClass, ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass, DataManifest, DataProvenance,
    EvidenceSourceKind, MarketVenue, OfficialCandleCoveragePackConfig, PreflightCheck,
    PreflightCheckResult, PreflightCheckStatus, PreflightFinalStatus, PreflightReport,
    ProviderMarket, ReasonCode, Timeframe,
};

use super::common;

fn persistent_output_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint10-tests")
        .join(name);
    fs::create_dir_all(&path).expect("create persistent test output dir");
    path
}

pub fn write_csv(
    name: &str,
    stem: &str,
    symbol: &str,
    timeframe: &str,
    timestamps: &[u64],
    invalid: bool,
) -> PathBuf {
    let dir = persistent_output_dir(&format!("{name}-csv"));
    let path = dir.join(format!("{stem}.csv"));
    let mut text = "timestamp,open,high,low,close,volume,symbol,timeframe\n".to_string();
    for (index, timestamp) in timestamps.iter().enumerate() {
        if invalid && index == 0 {
            text.push_str("bad,1,2,0.5,1.5,1000,AAPL,1d\n");
        } else {
            text.push_str(&format!(
                "{timestamp},{},{},{},{},{},{symbol},{timeframe}\n",
                100.0 + index as f64,
                101.0 + index as f64,
                99.0 + index as f64,
                100.5 + index as f64,
                1000.0 + index as f64,
            ));
        }
    }
    fs::write(&path, text).expect("write candle csv");
    path
}

pub fn write_provenance(
    name: &str,
    stem: &str,
    source_kind: EvidenceSourceKind,
    source_label: &str,
    local_path: &Path,
    official_provider: bool,
) -> PathBuf {
    let dir = persistent_output_dir(&format!("{name}-sidecars"));
    let path = dir.join(format!("{stem}_provenance.json"));
    let provenance = DataProvenance {
        source_kind,
        source_label: source_label.to_string(),
        provider_label: Some(
            if official_provider {
                "AlphaVantage"
            } else {
                "MockFixture"
            }
            .to_string(),
        ),
        upstream_label: None,
        local_path: Some(local_path.display().to_string()),
        generated_by: None,
        user_supplied: true,
        downloaded_by_soma: false,
        remote_url_present: false,
        official_provider: Some(official_provider),
        affiliated_or_endorsed: Some(official_provider),
        intended_use: Some("research-only".to_string()),
        readiness_eligible: Some(official_provider),
        benchmark_eligible: Some(true),
        license_note: None,
        notes: None,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    fs::write(
        &path,
        serde_json::to_string_pretty(&provenance).expect("json"),
    )
    .expect("write provenance");
    path
}

pub fn write_preflight(
    name: &str,
    stem: &str,
    symbol: &str,
    timeframe: Timeframe,
    input_path: &Path,
    source_kind: EvidenceSourceKind,
    final_status: PreflightFinalStatus,
) -> PathBuf {
    let dir = persistent_output_dir(&format!("{name}-sidecars"));
    let path = dir.join(format!("{stem}_preflight.json"));
    let report = PreflightReport {
        onboarding_id: format!("{name}-onboarding"),
        input_path: input_path.display().to_string(),
        detected_format: None,
        symbol: symbol.to_string(),
        timeframe,
        provenance: DataProvenance {
            source_kind,
            source_label: "preflight-source".to_string(),
            provider_label: None,
            upstream_label: None,
            local_path: Some(input_path.display().to_string()),
            generated_by: None,
            user_supplied: true,
            downloaded_by_soma: false,
            remote_url_present: false,
            official_provider: Some(matches!(
                source_kind,
                EvidenceSourceKind::OfficialApiCollected
            )),
            affiliated_or_endorsed: Some(matches!(
                source_kind,
                EvidenceSourceKind::OfficialApiCollected
            )),
            intended_use: Some("research-only".to_string()),
            readiness_eligible: Some(matches!(
                source_kind,
                EvidenceSourceKind::OfficialApiCollected
            )),
            benchmark_eligible: Some(true),
            license_note: None,
            notes: None,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        data_quality_report: None,
        data_manifest_preview: None,
        evidence_target_estimate: None,
        row_count: 32,
        usable_row_count: 32,
        estimated_walk_forward_folds: 2,
        estimated_outcome_records: 8,
        estimated_comparable_variants: 2,
        checks: vec![PreflightCheckResult {
            check: PreflightCheck::RealLocalEligible,
            status: if final_status == PreflightFinalStatus::ReadyForRealEvidence {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Warning
            },
            summary: format!("{final_status:?}"),
            reason_codes: vec![ReasonCode::PreflightReportBuilt],
        }],
        final_status,
        blockers: Vec::new(),
        warnings: Vec::new(),
        reason_codes: vec![ReasonCode::PreflightReportBuilt],
    };
    fs::write(&path, serde_json::to_string_pretty(&report).expect("json"))
        .expect("write preflight");
    path
}

pub fn write_manifest(
    name: &str,
    stem: &str,
    symbol: &str,
    timeframe: Timeframe,
    source_kind: EvidenceSourceKind,
    input_path: &Path,
    timestamps: &[u64],
) -> PathBuf {
    let dir = persistent_output_dir(&format!("{name}-sidecars"));
    let path = dir.join(format!("{stem}_manifest.json"));
    let market = market_for_symbol(symbol);
    let manifest = DataManifest {
        manifest_version: 1,
        dataset_id: format!("{stem}-dataset"),
        symbol: symbol.to_string(),
        normalized_symbol: symbol
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_uppercase(),
        venue: if market == ProviderMarket::Crypto {
            MarketVenue::Binance
        } else {
            MarketVenue::US
        },
        asset_class: if market == ProviderMarket::Crypto {
            AssetClass::Crypto
        } else {
            AssetClass::Equity
        },
        timeframe,
        source_kind,
        source_path: Some(input_path.display().to_string()),
        provenance: None,
        row_count: timestamps.len(),
        first_timestamp_ms: *timestamps.first().unwrap_or(&0),
        last_timestamp_ms: *timestamps.last().unwrap_or(&0),
        expected_step_ms: timestamps
            .get(1)
            .copied()
            .unwrap_or(0)
            .saturating_sub(*timestamps.first().unwrap_or(&0)),
        data_quality_score: 0.99,
        feature_schema_hash: None,
        label_config_summary: None,
        cost_model_summary: None,
        adjusted_price_policy_summary: Some("raw".to_string()),
        corporate_action_adjusted: Some(false),
        provider_symbol: None,
        collection_size_policy_summary: None,
        truncated: false,
        row_limit_applied: false,
        raw_archive_policy_summary: None,
        auth_requirement_summary: None,
        created_at_ms: Some(42),
        reason_codes: vec![ReasonCode::DataManifestBuilt],
    };
    fs::write(
        &path,
        serde_json::to_string_pretty(&manifest).expect("json"),
    )
    .expect("write manifest");
    path
}

pub fn pack_config(
    name: &str,
    csv_paths: Vec<String>,
    provenance_paths: Vec<String>,
    preflight_paths: Vec<String>,
    manifest_paths: Vec<String>,
) -> OfficialCandleCoveragePackConfig {
    OfficialCandleCoveragePackConfig {
        pack_id: name.to_string(),
        canonical_csv_paths: csv_paths,
        provenance_paths,
        preflight_report_paths: preflight_paths,
        manifest_paths,
        output_root: common::output_dir(&format!("{name}-pack-out"))
            .display()
            .to_string(),
        max_rows: 1000,
        max_symbols: 5,
        max_timeframes: 5,
        max_bytes: 5_000_000,
        ..OfficialCandleCoveragePackConfig::default()
    }
}

pub fn comparable_row(
    id: &str,
    symbol: &str,
    timeframe: &str,
    timestamp_ms: u64,
    source_class: ComparableEvidenceSourceClass,
    official_readiness_eligible: bool,
    diagnostic_only: bool,
    summary_derived: bool,
) -> ComparableCommitteeEvidenceRow {
    ComparableCommitteeEvidenceRow {
        row_id: id.to_string(),
        symbol: symbol.to_string(),
        market: market_for_symbol(symbol),
        timeframe: timeframe.to_string(),
        horizon_bars: 3,
        timestamp_ms,
        source_kind: format!("{:?}", source_class),
        source_class,
        scenario_row_id: Some(id.to_string()),
        committee_decision_id: None,
        committee_final_action: "Approve".to_string(),
        chair_decision: None,
        risk_governor_decision: Some("Approve".to_string()),
        baseline_action: Some("Approve".to_string()),
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: None,
        net_return_pct: None,
        cost_bps: 2.0,
        slippage_bps: 1.0,
        committee_vs_baseline_delta: None,
        committee_vs_notrade_delta: None,
        risk_denied_value_proxy: None,
        no_trade_value_proxy: None,
        outcome_reference_available: false,
        baseline_reference_available: true,
        no_trade_counterfactual_available: true,
        risk_denied_counterfactual_available: true,
        external_reference_available: false,
        row_level: !summary_derived,
        summary_derived,
        no_lookahead_safe: true,
        official_readiness_eligible,
        diagnostic_only,
        candle_coverage_available: false,
        matched_candle_series_id: None,
        candle_match_status: None,
        candle_official_ready_match: false,
        candle_benchmark_ready_match: false,
        candle_diagnostic_only: false,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

pub fn write_bundle(name: &str, rows: Vec<ComparableCommitteeEvidenceRow>) -> PathBuf {
    let config = ComparableCommitteeEvidenceConfig {
        comparable_id: name.to_string(),
        output_root: common::output_dir(&format!("{name}-bundle-out"))
            .display()
            .to_string(),
        allow_summary_derived_rows: true,
        require_outcome_reference: false,
        ..ComparableCommitteeEvidenceConfig::default()
    };
    let bundle = ComparableCommitteeEvidenceBundle::from_rows(&config, rows);
    bundle
        .write_to_dir(&config.output_dir())
        .expect("write bundle")
}

pub fn write_pack_config_file(name: &str, config: &OfficialCandleCoveragePackConfig) -> PathBuf {
    let path = common::output_dir(&format!("{name}-pack-config")).join("pack.toml");
    fs::write(&path, config.to_toml_string().expect("toml")).expect("write pack config");
    path
}

pub fn market_for_symbol(symbol: &str) -> ProviderMarket {
    if symbol.to_ascii_uppercase().contains("BTC") {
        ProviderMarket::Crypto
    } else {
        ProviderMarket::USEquity
    }
}
