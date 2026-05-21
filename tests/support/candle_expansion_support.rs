#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use soma_zero::{
    ComparableEvidenceSourceClass, EvidenceSourceKind, OfficialCandleCoverageGapMap,
    OfficialCandleExpansionPlanConfig, OfficialCandleGapConfig, PreflightFinalStatus,
    ProviderMarket,
};

use crate::common;
#[path = "candle_coverage_support.rs"]
pub mod candle_coverage_support;

pub fn clear_env() {
    for key in [
        "KRX_API_KEY",
        "KRX_ENDPOINT_TEMPLATE",
        "KRX_APPROVAL_READY",
        "KRX_APPROVED",
        "ALPHAVANTAGE_API_KEY",
        "ALPACA_API_KEY",
        "ALPACA_SECRET_KEY",
        "DATA_GO_KR_SERVICE_KEY",
        "DATAGOKR_SERVICE_KEY",
    ] {
        unsafe { std::env::remove_var(key) };
    }
}

pub fn official_csv_fixture(
    name: &str,
    stem: &str,
    symbol: &str,
    timeframe: &str,
    timestamps: &[u64],
    with_provenance: bool,
    with_preflight: bool,
    with_manifest: bool,
) -> (PathBuf, Vec<String>, Vec<String>, Vec<String>) {
    let csv = candle_coverage_support::write_csv(name, stem, symbol, timeframe, timestamps, false);
    let provenance = with_provenance
        .then(|| {
            candle_coverage_support::write_provenance(
                name,
                stem,
                EvidenceSourceKind::OfficialApiCollected,
                "official",
                &csv,
                true,
            )
            .display()
            .to_string()
        })
        .into_iter()
        .collect::<Vec<_>>();
    let preflight = with_preflight
        .then(|| {
            candle_coverage_support::write_preflight(
                name,
                stem,
                symbol,
                parse_timeframe(timeframe),
                &csv,
                EvidenceSourceKind::OfficialApiCollected,
                PreflightFinalStatus::ReadyForRealEvidence,
            )
            .display()
            .to_string()
        })
        .into_iter()
        .collect::<Vec<_>>();
    let manifest = with_manifest
        .then(|| {
            candle_coverage_support::write_manifest(
                name,
                stem,
                symbol,
                parse_timeframe(timeframe),
                EvidenceSourceKind::OfficialApiCollected,
                &csv,
                timestamps,
            )
            .display()
            .to_string()
        })
        .into_iter()
        .collect::<Vec<_>>();
    (csv, provenance, preflight, manifest)
}

pub fn gap_config_path(name: &str, bundle_paths: Vec<String>, pack_paths: Vec<String>) -> PathBuf {
    let path = common::output_dir(&format!("{name}-gap-config")).join("gap.toml");
    fs::write(
        &path,
        OfficialCandleGapConfig {
            gap_id: format!("{name}-gap"),
            comparable_evidence_bundle_paths: bundle_paths,
            candle_coverage_pack_paths: pack_paths,
            output_root: common::output_dir(&format!("{name}-gap-out"))
                .display()
                .to_string(),
            ..OfficialCandleGapConfig::default()
        }
        .to_toml_string()
        .expect("toml"),
    )
    .expect("write gap config");
    path
}

pub fn plan_config(
    name: &str,
    gap_config_path: Option<&Path>,
    gap_map_path: Option<&Path>,
) -> OfficialCandleExpansionPlanConfig {
    OfficialCandleExpansionPlanConfig {
        plan_id: name.to_string(),
        gap_config_path: gap_config_path.map(display),
        gap_map_path: gap_map_path.map(display),
        output_root: common::output_dir(&format!("{name}-plan-out"))
            .display()
            .to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    }
}

pub fn plan_config_path(config: &OfficialCandleExpansionPlanConfig) -> PathBuf {
    let path = common::output_dir(&format!("{}-plan-config", config.plan_id)).join("plan.toml");
    fs::write(&path, config.to_toml_string().expect("toml")).expect("write plan config");
    path
}

pub fn manual_gap_map_path(
    name: &str,
    market: ProviderMarket,
    symbol: &str,
    timeframe: &str,
    source_class: ComparableEvidenceSourceClass,
    related_artifact_paths: Vec<String>,
) -> PathBuf {
    let path = common::output_dir(&format!("{name}-gap-map")).join("gap_map.json");
    let map = OfficialCandleCoverageGapMap {
        gap_id: format!("{name}-map"),
        cells: vec![soma_zero::OfficialCandleGapCell {
            market,
            symbol: symbol.to_string(),
            normalized_symbol: normalize_symbol(symbol),
            venue: None,
            timeframe: timeframe.to_string(),
            horizon_bars: 3,
            source_kind: Some(format!("{:?}", source_class)),
            source_class,
            row_count_impacted: 1,
            comparable_rows_impacted: 1,
            missing_future_bars: 3,
            required_start_timestamp_ms: Some(1_700_000_000_000),
            required_end_timestamp_ms: Some(1_700_259_200_000),
            required_min_rows: 4,
            gap_kinds: vec![
                soma_zero::OfficialCandleGapKind::MissingOfficialCandleSeries,
                soma_zero::OfficialCandleGapKind::MissingCandleSeries,
            ],
            buildable_from_existing_local_csv: !related_artifact_paths.is_empty(),
            buildable_from_provider_collection: true,
            requires_operator_action: true,
            related_artifact_paths,
            reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
        }],
        total_gaps: 1,
        official_gap_count: usize::from(
            source_class == ComparableEvidenceSourceClass::OfficialNonCrypto,
        ),
        non_crypto_official_gap_count: usize::from(
            source_class == ComparableEvidenceSourceClass::OfficialNonCrypto,
        ),
        crypto_gap_count: usize::from(
            source_class == ComparableEvidenceSourceClass::OfficialCryptoOnly,
        ),
        diagnostic_gap_count: usize::from(
            source_class == ComparableEvidenceSourceClass::ControlledDiagnostic,
        ),
        research_only_gap_count: usize::from(
            source_class == ComparableEvidenceSourceClass::YFinanceResearch,
        ),
        fixture_gap_count: usize::from(matches!(
            source_class,
            ComparableEvidenceSourceClass::FixtureArchitectureTest
                | ComparableEvidenceSourceClass::SyntheticTest
        )),
        buildable_gap_count: 1,
        operator_action_gap_count: 1,
        gap_status: soma_zero::OfficialCandleGapStatus::MissingOfficialCandles,
        warnings: Vec::new(),
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    };
    fs::write(&path, serde_json::to_string_pretty(&map).expect("json")).expect("write map");
    path
}

pub fn row_bundle_path(
    name: &str,
    symbol: &str,
    timeframe: &str,
    timestamp_ms: u64,
    source_class: ComparableEvidenceSourceClass,
    summary_derived: bool,
) -> PathBuf {
    candle_coverage_support::write_bundle(
        name,
        vec![candle_coverage_support::comparable_row(
            &format!("{name}-row"),
            symbol,
            timeframe,
            timestamp_ms,
            source_class,
            source_class == ComparableEvidenceSourceClass::OfficialNonCrypto,
            source_class != ComparableEvidenceSourceClass::OfficialNonCrypto,
            summary_derived,
        )],
    )
}

pub fn display(path: &Path) -> String {
    path.display().to_string()
}

fn parse_timeframe(value: &str) -> soma_zero::Timeframe {
    match value {
        "1m" => soma_zero::Timeframe::OneMinute,
        "5m" => soma_zero::Timeframe::FiveMinute,
        _ => soma_zero::Timeframe::OneDay,
    }
}

fn normalize_symbol(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase()
}
