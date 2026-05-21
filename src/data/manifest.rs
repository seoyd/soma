use serde::{Deserialize, Serialize};

use crate::backtest::{CandleSeries, Timeframe};
use crate::core::{ReasonCode, stable_hash};

use super::{
    AssetClass, DataProvenance, DataQualityReport, MarketVenue, SymbolSpec, TimeframeSpec,
};
use super::{DataSourceKind, infer_source_kind_from_path};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataManifest {
    pub manifest_version: u32,
    pub dataset_id: String,
    pub symbol: String,
    pub normalized_symbol: String,
    pub venue: MarketVenue,
    pub asset_class: AssetClass,
    pub timeframe: Timeframe,
    pub source_kind: DataSourceKind,
    pub source_path: Option<String>,
    #[serde(default)]
    pub provenance: Option<DataProvenance>,
    pub row_count: usize,
    pub first_timestamp_ms: u64,
    pub last_timestamp_ms: u64,
    pub expected_step_ms: u64,
    pub data_quality_score: f64,
    pub feature_schema_hash: Option<u64>,
    pub label_config_summary: Option<String>,
    pub cost_model_summary: Option<String>,
    #[serde(default)]
    pub adjusted_price_policy_summary: Option<String>,
    #[serde(default)]
    pub corporate_action_adjusted: Option<bool>,
    #[serde(default)]
    pub provider_symbol: Option<String>,
    #[serde(default)]
    pub collection_size_policy_summary: Option<String>,
    #[serde(default)]
    pub truncated: bool,
    #[serde(default)]
    pub row_limit_applied: bool,
    #[serde(default)]
    pub raw_archive_policy_summary: Option<String>,
    #[serde(default)]
    pub auth_requirement_summary: Option<String>,
    pub created_at_ms: Option<u64>,
    pub reason_codes: Vec<ReasonCode>,
}

impl DataManifest {
    pub fn build(
        series: &CandleSeries,
        symbol_spec: &SymbolSpec,
        timeframe_spec: &TimeframeSpec,
        report: &DataQualityReport,
        source_kind: DataSourceKind,
        source_path: Option<String>,
        provenance: Option<DataProvenance>,
        created_at_ms: Option<u64>,
    ) -> Self {
        let first_timestamp_ms = series
            .candles
            .first()
            .map(|candle| candle.timestamp_ms)
            .unwrap_or(0);
        let last_timestamp_ms = series
            .candles
            .last()
            .map(|candle| candle.timestamp_ms)
            .unwrap_or(0);
        let dataset_id = format!(
            "{:016x}",
            stable_hash(&format!(
                "{}|{:?}|{}|{}|{}",
                symbol_spec.normalized_symbol,
                timeframe_spec.timeframe,
                first_timestamp_ms,
                last_timestamp_ms,
                series.len()
            ))
        );
        let mut reason_codes = symbol_spec.reason_codes.clone();
        reason_codes.extend(timeframe_spec.reason_codes.iter().cloned());
        reason_codes.extend(report.reason_codes.iter().cloned());
        reason_codes.push(ReasonCode::DataManifestBuilt);
        let provenance =
            provenance.or_else(|| Some(DataProvenance::inferred_from_path(source_path.as_deref())));
        Self {
            manifest_version: 1,
            dataset_id,
            symbol: symbol_spec.raw_symbol.clone(),
            normalized_symbol: symbol_spec.normalized_symbol.clone(),
            venue: symbol_spec.venue,
            asset_class: symbol_spec.asset_class,
            timeframe: series.timeframe,
            source_kind,
            source_path,
            provenance,
            row_count: series.len(),
            first_timestamp_ms,
            last_timestamp_ms,
            expected_step_ms: timeframe_spec.expected_ms_step,
            data_quality_score: report.data_quality_score,
            feature_schema_hash: None,
            label_config_summary: None,
            cost_model_summary: None,
            adjusted_price_policy_summary: None,
            corporate_action_adjusted: None,
            provider_symbol: None,
            collection_size_policy_summary: None,
            truncated: false,
            row_limit_applied: false,
            raw_archive_policy_summary: None,
            auth_requirement_summary: None,
            created_at_ms,
            reason_codes,
        }
    }

    pub fn to_deterministic_string(&self) -> String {
        [
            format!("manifest_version={}", self.manifest_version),
            format!("dataset_id={}", self.dataset_id),
            format!("symbol={}", self.symbol),
            format!("normalized_symbol={}", self.normalized_symbol),
            format!("venue={:?}", self.venue),
            format!("asset_class={:?}", self.asset_class),
            format!("timeframe={:?}", self.timeframe),
            format!("source_kind={:?}", self.source_kind),
            format!(
                "source_path={}",
                self.source_path.clone().unwrap_or_default()
            ),
            format!(
                "provenance.source_kind={:?}",
                self.provenance
                    .as_ref()
                    .map(|provenance| provenance.source_kind)
                    .unwrap_or_else(|| {
                        self.source_path
                            .as_deref()
                            .map(std::path::Path::new)
                            .map(|path| infer_source_kind_from_path(Some(path)))
                            .unwrap_or(DataSourceKind::Unknown)
                    })
            ),
            format!("row_count={}", self.row_count),
            format!("first_timestamp_ms={}", self.first_timestamp_ms),
            format!("last_timestamp_ms={}", self.last_timestamp_ms),
            format!("expected_step_ms={}", self.expected_step_ms),
            format!("data_quality_score={:.6}", self.data_quality_score),
            format!(
                "feature_schema_hash={}",
                self.feature_schema_hash
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!(
                "label_config_summary={}",
                self.label_config_summary.clone().unwrap_or_default()
            ),
            format!(
                "cost_model_summary={}",
                self.cost_model_summary.clone().unwrap_or_default()
            ),
            format!(
                "adjusted_price_policy_summary={}",
                self.adjusted_price_policy_summary
                    .clone()
                    .unwrap_or_default()
            ),
            format!(
                "corporate_action_adjusted={}",
                self.corporate_action_adjusted
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!(
                "provider_symbol={}",
                self.provider_symbol.clone().unwrap_or_default()
            ),
            format!(
                "collection_size_policy_summary={}",
                self.collection_size_policy_summary
                    .clone()
                    .unwrap_or_default()
            ),
            format!("truncated={}", self.truncated),
            format!("row_limit_applied={}", self.row_limit_applied),
            format!(
                "raw_archive_policy_summary={}",
                self.raw_archive_policy_summary.clone().unwrap_or_default()
            ),
            format!(
                "auth_requirement_summary={}",
                self.auth_requirement_summary.clone().unwrap_or_default()
            ),
            format!(
                "created_at_ms={}",
                self.created_at_ms
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }
}
