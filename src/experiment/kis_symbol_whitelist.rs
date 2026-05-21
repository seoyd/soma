use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KISMarket {
    KoreanEquity,
    USEquity,
    JapaneseEquity,
    HongKongEquity,
    OtherOverseasEquity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KISDataFreshness {
    Eod,
    Delayed,
    Realtime,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISSymbolEntry {
    pub provider_symbol: String,
    #[serde(default)]
    pub normalized_symbol: String,
    #[serde(default = "default_market")]
    pub market: KISMarket,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub exchange_code: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub max_rows: Option<usize>,
    #[serde(default = "default_timeframe")]
    pub timeframe: String,
    #[serde(default = "default_freshness")]
    pub data_freshness: KISDataFreshness,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KISSymbolWhitelistConfig {
    pub whitelist_id: String,
    #[serde(default)]
    pub symbols: Vec<KISSymbolEntry>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_true")]
    pub require_market: bool,
    #[serde(default = "default_true")]
    pub require_provider_symbol: bool,
    #[serde(default = "default_true")]
    pub require_exchange_code_for_overseas: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISSymbolWhitelist {
    pub whitelist_id: String,
    pub entries: Vec<KISSymbolEntry>,
    pub enabled_entries: Vec<String>,
    pub skipped_entries: Vec<String>,
    pub domestic_count: usize,
    pub overseas_count: usize,
    pub symbol_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for KISSymbolWhitelistConfig {
    fn default() -> Self {
        Self {
            whitelist_id: "kis_symbol_whitelist".to_string(),
            symbols: Vec::new(),
            output_root: default_output_root(),
            max_symbols: default_max_symbols(),
            require_market: true,
            require_provider_symbol: true,
            require_exchange_code_for_overseas: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl KISSymbolWhitelistConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        if self.output_root.contains("://") {
            vec![ReasonCode::RemotePathRejected]
        } else {
            Vec::new()
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.whitelist_id.trim().is_empty() {
            return Err("kis whitelist_id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("kis symbol whitelist paths must be local".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err("kis max_symbols must be between 1 and 5".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.whitelist_id)
    }

    pub fn build(&self) -> KISSymbolWhitelist {
        let mut entries = self.symbols.clone();
        for entry in &mut entries {
            entry.provider_symbol = entry.provider_symbol.trim().to_string();
            entry.normalized_symbol = normalize_symbol_for_market(
                if entry.normalized_symbol.trim().is_empty() {
                    &entry.provider_symbol
                } else {
                    &entry.normalized_symbol
                },
                entry.market,
            );
            entry.timeframe = entry.timeframe.trim().to_ascii_lowercase();
            entry.reason_codes = entry_reason_codes(entry, self);
        }
        entries.sort_by(|left, right| {
            left.market
                .cmp(&right.market)
                .then(left.normalized_symbol.cmp(&right.normalized_symbol))
                .then(left.provider_symbol.cmp(&right.provider_symbol))
        });
        let enabled_entries = entries
            .iter()
            .filter(|entry| entry.enabled && entry.is_valid())
            .map(|entry| entry.normalized_symbol.clone())
            .collect::<Vec<_>>();
        let skipped_entries = entries
            .iter()
            .filter(|entry| !entry.enabled || !entry.is_valid())
            .map(|entry| entry.provider_symbol.clone())
            .collect::<Vec<_>>();
        let domestic_count = entries
            .iter()
            .filter(|entry| entry.market == KISMarket::KoreanEquity)
            .count();
        let overseas_count = entries.len().saturating_sub(domestic_count);
        let mut reason_codes = self.reason_codes.clone();
        reason_codes.push(ReasonCode::KISSymbolWhitelistBuilt);
        if entries
            .iter()
            .any(|entry| is_all_symbol(&entry.provider_symbol))
        {
            reason_codes.push(ReasonCode::DeniedByDefault);
        }
        if enabled_entries.len() > self.max_symbols {
            reason_codes.push(ReasonCode::BudgetExceeded);
        }
        KISSymbolWhitelist {
            whitelist_id: self.whitelist_id.clone(),
            entries,
            enabled_entries,
            skipped_entries,
            domestic_count,
            overseas_count,
            symbol_count: self.symbols.len(),
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }
}

impl KISSymbolEntry {
    pub fn is_overseas(&self) -> bool {
        self.market != KISMarket::KoreanEquity
    }

    pub fn is_valid(&self) -> bool {
        !self.reason_codes.iter().any(|reason| {
            matches!(
                reason,
                ReasonCode::InvalidSymbol
                    | ReasonCode::DeniedByDefault
                    | ReasonCode::UnsupportedTimeframe
            )
        })
    }
}

impl KISSymbolWhitelist {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            "research_only_warning=kis symbol whitelist remains market-data-only and bounded"
                .to_string(),
            "scope_warning=no wildcard or all-symbol collection is permitted".to_string(),
            format!("whitelist_id={}", self.whitelist_id),
            format!("enabled_entries={}", self.enabled_entries.join("|")),
            format!("skipped_entries={}", self.skipped_entries.join("|")),
            format!("domestic_count={}", self.domestic_count),
            format!("overseas_count={}", self.overseas_count),
            format!("symbol_count={}", self.symbol_count),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ];
        lines.extend(self.entries.iter().map(|entry| {
            format!(
                "provider_symbol={};normalized_symbol={};market={:?};venue={};exchange_code={};display_name={};enabled={};max_rows={};timeframe={};data_freshness={:?};reason_codes={}",
                entry.provider_symbol,
                entry.normalized_symbol,
                entry.market,
                entry.venue.clone().unwrap_or_default(),
                entry.exchange_code.clone().unwrap_or_default(),
                entry.display_name.clone().unwrap_or_default(),
                entry.enabled,
                entry.max_rows.map(|value| value.to_string()).unwrap_or_default(),
                entry.timeframe,
                entry.data_freshness,
                entry.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            )
        }));
        lines.join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("kis_symbol_whitelist.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_symbol_whitelist.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

fn entry_reason_codes(
    entry: &KISSymbolEntry,
    config: &KISSymbolWhitelistConfig,
) -> Vec<ReasonCode> {
    let mut reason_codes = Vec::new();
    if config.require_provider_symbol && entry.provider_symbol.trim().is_empty() {
        reason_codes.push(ReasonCode::InvalidSymbol);
    }
    if config.require_market
        && matches!(entry.market, KISMarket::OtherOverseasEquity)
        && entry.venue.is_none()
    {
        reason_codes.push(ReasonCode::InvalidSymbol);
    }
    if is_all_symbol(&entry.provider_symbol) || is_all_symbol(&entry.normalized_symbol) {
        reason_codes.push(ReasonCode::DeniedByDefault);
    }
    if !matches!(entry.timeframe.as_str(), "1d" | "1m" | "5m" | "15m") {
        reason_codes.push(ReasonCode::UnsupportedTimeframe);
    }
    if entry.market == KISMarket::KoreanEquity {
        if !looks_like_domestic_symbol(&entry.normalized_symbol) {
            reason_codes.push(ReasonCode::InvalidSymbol);
        }
    } else {
        if config.require_exchange_code_for_overseas
            && entry.enabled
            && entry
                .exchange_code
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
        {
            reason_codes.push(ReasonCode::InvalidSymbol);
        }
        if !looks_like_overseas_symbol(&entry.normalized_symbol) {
            reason_codes.push(ReasonCode::InvalidSymbol);
        }
    }
    stable_reason_codes(&reason_codes)
}

fn looks_like_domestic_symbol(value: &str) -> bool {
    value.len() == 6 && value.chars().all(|character| character.is_ascii_digit())
}

fn looks_like_overseas_symbol(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.len() <= 10
        && trimmed.chars().all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '-'
        })
}

fn is_all_symbol(value: &str) -> bool {
    let normalized = value.trim().to_ascii_uppercase();
    normalized == "ALL" || normalized == "ALL_SYMBOLS" || normalized.contains('*')
}

pub fn normalize_symbol_for_market(value: &str, market: KISMarket) -> String {
    let trimmed = value.trim();
    match market {
        KISMarket::KoreanEquity => trimmed
            .chars()
            .filter(|character| character.is_ascii_digit())
            .collect(),
        _ => trimmed
            .to_ascii_uppercase()
            .replace(' ', "-")
            .replace('_', "-"),
    }
}

fn default_true() -> bool {
    true
}

fn default_output_root() -> String {
    "target/soma_kis_market_data_activation".to_string()
}

fn default_max_symbols() -> usize {
    5
}

fn default_timeframe() -> String {
    "1d".to_string()
}

fn default_freshness() -> KISDataFreshness {
    KISDataFreshness::Eod
}

fn default_market() -> KISMarket {
    KISMarket::KoreanEquity
}
