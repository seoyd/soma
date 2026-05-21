use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::backtest::{CostModel, Timeframe, TripleBarrierConfig};
use crate::core::{ReasonCode, stable_hash};
use crate::eval::WalkForwardConfig;

use super::{
    AssetClass, CandleCsvConfig, CandleCsvFormat, CustomColumnMap, DataProvenance,
    EvidenceSourceKind, MarketVenue, TimestampFormat,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalDataOnboardingConfig {
    pub onboarding_id: String,
    pub input_path: String,
    pub output_root: String,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub venue: Option<MarketVenue>,
    #[serde(default)]
    pub asset_class: Option<AssetClass>,
    #[serde(default)]
    pub timeframe: Option<Timeframe>,
    #[serde(default)]
    pub csv_format_hint: Option<CandleCsvFormat>,
    #[serde(default)]
    pub custom_column_map: Option<CustomColumnMap>,
    #[serde(default)]
    pub source_kind: Option<EvidenceSourceKind>,
    #[serde(default = "default_true")]
    pub user_supplied: bool,
    #[serde(default)]
    pub source_label: Option<String>,
    #[serde(default = "default_true")]
    pub strict: bool,
    #[serde(default = "default_true")]
    pub allow_format_autodetect: bool,
    #[serde(default)]
    pub allow_sort_repair: bool,
    #[serde(default)]
    pub allow_duplicate_drop: bool,
    #[serde(default = "default_min_rows_for_preflight")]
    pub min_rows_for_preflight: usize,
    #[serde(default = "default_twenty")]
    pub target_min_outcomes: usize,
    #[serde(default = "default_two")]
    pub target_min_comparable_variants: usize,
    #[serde(default = "default_one")]
    pub target_min_usable_datasets: usize,
    #[serde(default)]
    pub walk_forward_config: Option<WalkForwardConfig>,
    #[serde(default)]
    pub triple_barrier_config: Option<TripleBarrierConfig>,
    #[serde(default)]
    pub cost_model: Option<CostModel>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for LocalDataOnboardingConfig {
    fn default() -> Self {
        Self {
            onboarding_id: "real-local-onboarding".to_string(),
            input_path: "data/local/PUT_REAL_CSV_HERE.csv".to_string(),
            output_root: "target/soma_data_onboarding".to_string(),
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
            min_rows_for_preflight: default_min_rows_for_preflight(),
            target_min_outcomes: default_twenty(),
            target_min_comparable_variants: default_two(),
            target_min_usable_datasets: default_one(),
            walk_forward_config: None,
            triple_barrier_config: None,
            cost_model: None,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl LocalDataOnboardingConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&contents)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = self.reason_codes.clone();
        if self.input_path.contains("://") || self.output_root.contains("://") {
            reasons.push(ReasonCode::LocalPathRejected);
        }
        dedupe_reasons(reasons)
    }

    pub fn resolved_source_label(&self) -> String {
        self.source_label.clone().unwrap_or_else(|| {
            let symbol = self
                .symbol
                .clone()
                .unwrap_or_else(|| "unknown".to_string())
                .replace(['/', '-', ' '], "_")
                .to_ascii_lowercase();
            let timeframe = self
                .timeframe
                .map(|timeframe| format!("{timeframe:?}"))
                .unwrap_or_else(|| "unknown".to_string())
                .to_ascii_lowercase();
            let path_hash = stable_hash(&self.input_path);
            format!("user-local-{symbol}-{timeframe}-{path_hash:08x}")
        })
    }

    pub fn resolved_symbol(&self) -> String {
        self.symbol
            .clone()
            .unwrap_or_else(|| "BTC-USDT".to_string())
    }

    pub fn resolved_venue(&self, format: Option<&CandleCsvFormat>) -> MarketVenue {
        self.venue.unwrap_or_else(|| match format {
            Some(CandleCsvFormat::BinanceKline) => MarketVenue::Binance,
            Some(CandleCsvFormat::UpbitCandle) => MarketVenue::Upbit,
            Some(CandleCsvFormat::KrxOhlcv) => MarketVenue::KRX,
            Some(CandleCsvFormat::GenericOhlcv | CandleCsvFormat::Custom { .. }) | None => {
                MarketVenue::Generic
            }
        })
    }

    pub fn resolved_asset_class(&self) -> AssetClass {
        self.asset_class.unwrap_or(AssetClass::Crypto)
    }

    pub fn resolved_timeframe(&self) -> Timeframe {
        self.timeframe.unwrap_or(Timeframe::OneMinute)
    }

    pub fn resolved_walk_forward_config(&self) -> WalkForwardConfig {
        self.walk_forward_config.unwrap_or_default()
    }

    pub fn resolved_triple_barrier_config(&self) -> TripleBarrierConfig {
        self.triple_barrier_config.unwrap_or(TripleBarrierConfig {
            take_profit_pct: 0.02,
            stop_loss_pct: 0.01,
            horizon_bars: 2,
            fee_bps: 2.0,
            slippage_bps: 2.0,
            side: crate::core::Side::Long,
            use_high_low_intrabar: true,
        })
    }

    pub fn resolved_cost_model(&self) -> CostModel {
        self.cost_model.unwrap_or(CostModel {
            fee_bps: 2.0,
            slippage_bps: 2.0,
            spread_bps: Some(2.0),
            min_cost_bps: None,
        })
    }

    pub fn build_provenance(&self) -> DataProvenance {
        let remote = self.input_path.contains("://");
        let source_kind = self.source_kind.unwrap_or(EvidenceSourceKind::RealLocal);
        let mut reason_codes = self.reason_codes.clone();
        if remote {
            reason_codes.push(ReasonCode::LocalPathRejected);
        }
        if !self.user_supplied && source_kind == EvidenceSourceKind::RealLocal {
            reason_codes.push(ReasonCode::PreflightNotRealLocalEligible);
        }
        DataProvenance {
            source_kind,
            source_label: self.resolved_source_label(),
            provider_label: match source_kind {
                EvidenceSourceKind::OfficialApiCollected => Some("official-api-collected".to_string()),
                EvidenceSourceKind::YFinanceResearch => Some("yfinance".to_string()),
                _ => None,
            },
            upstream_label: match source_kind {
                EvidenceSourceKind::YFinanceResearch => Some("Yahoo Finance".to_string()),
                _ => None,
            },
            local_path: Some(self.input_path.clone()),
            generated_by: None,
            user_supplied: self.user_supplied,
            downloaded_by_soma: source_kind == EvidenceSourceKind::OfficialApiCollected,
            remote_url_present: remote,
            official_provider: Some(source_kind == EvidenceSourceKind::OfficialApiCollected),
            affiliated_or_endorsed: Some(source_kind == EvidenceSourceKind::OfficialApiCollected),
            intended_use: Some(match source_kind {
                EvidenceSourceKind::YFinanceResearch => {
                    "research-only unofficial supplemental benchmark data".to_string()
                }
                EvidenceSourceKind::OfficialApiCollected => {
                    "official evidence collection".to_string()
                }
                _ => "local research data onboarding".to_string(),
            }),
            readiness_eligible: Some(source_kind.readiness_eligible()),
            benchmark_eligible: Some(source_kind != EvidenceSourceKind::Unknown),
            license_note: Some(match source_kind {
                EvidenceSourceKind::YFinanceResearch => {
                    "User must verify Yahoo Finance and yfinance licensing and personal-use restrictions before use.".to_string()
                }
                _ => "User must verify local-market data licensing before use.".to_string(),
            }),
            notes: Some(match source_kind {
                EvidenceSourceKind::YFinanceResearch => {
                    "Generated by Sprint 27 research-only yfinance bridge.".to_string()
                }
                _ => "Generated by Sprint 17 onboarding flow.".to_string(),
            }),
            reason_codes: dedupe_reasons(reason_codes),
        }
    }

    pub fn build_csv_config(&self, format: CandleCsvFormat, has_header: bool) -> CandleCsvConfig {
        CandleCsvConfig {
            format,
            symbol: self.resolved_symbol(),
            timeframe: self.resolved_timeframe(),
            has_header,
            delimiter: ',',
            timestamp_format: default_timestamp_format(),
            strict: self.strict,
            allow_repair_sort: self.allow_sort_repair,
            allow_drop_invalid_rows: true,
            max_invalid_rows: usize::MAX,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

fn default_timestamp_format() -> TimestampFormat {
    TimestampFormat::Millis
}

fn default_true() -> bool {
    true
}

fn default_one() -> usize {
    1
}

fn default_two() -> usize {
    2
}

fn default_twenty() -> usize {
    20
}

fn default_min_rows_for_preflight() -> usize {
    40
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}
