use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::backtest::Candle;
use crate::core::ReasonCode;

use super::ProviderKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataGoKrProviderStatus {
    Ready,
    MissingAuth,
    DeferredUntilEndpointProfile,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataGoKrFscStockPriceImportConfig {
    pub import_id: String,
    pub fixture_path: String,
    pub output_root: String,
    pub symbol: String,
    #[serde(default = "default_service_key_env")]
    pub service_key_env_var: String,
    #[serde(default)]
    pub endpoint_profile: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataGoKrFscStockPriceImportReport {
    pub import_id: String,
    pub provider_kind: ProviderKind,
    pub status: DataGoKrProviderStatus,
    pub symbol: String,
    pub canonical_csv_path: String,
    pub row_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Deserialize)]
struct DataGoKrFixtureRow {
    #[serde(rename = "basDt")]
    base_date: String,
    #[serde(rename = "mkp")]
    open: String,
    #[serde(rename = "hipr")]
    high: String,
    #[serde(rename = "lopr")]
    low: String,
    #[serde(rename = "clpr")]
    close: String,
    #[serde(rename = "trqu")]
    volume: String,
}

impl DataGoKrFscStockPriceImportConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }
}

pub fn parse_datagokr_fsc_stock_price_fixture(input: &str) -> Result<Vec<Candle>, String> {
    let mut rows =
        serde_json::from_str::<Vec<DataGoKrFixtureRow>>(input).map_err(|err| err.to_string())?;
    rows.sort_by(|left, right| left.base_date.cmp(&right.base_date));
    rows.into_iter()
        .map(|row| {
            Ok(Candle {
                timestamp_ms: parse_yyyymmdd_to_timestamp_ms(&row.base_date)?,
                open: parse_number(&row.open)?,
                high: parse_number(&row.high)?,
                low: parse_number(&row.low)?,
                close: parse_number(&row.close)?,
                volume: parse_number(&row.volume)?,
                trade_value: None,
                bid: None,
                ask: None,
                spread_bps: None,
            })
        })
        .collect()
}

pub fn run_datagokr_fsc_stock_price_import(
    config: &DataGoKrFscStockPriceImportConfig,
) -> Result<DataGoKrFscStockPriceImportReport, String> {
    let fixture = fs::read_to_string(&config.fixture_path).map_err(|err| err.to_string())?;
    let candles = parse_datagokr_fsc_stock_price_fixture(&fixture)?;
    let output_dir = Path::new(&config.output_root)
        .join(&config.import_id)
        .join(&config.symbol);
    fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;
    let canonical_csv_path = output_dir.join("canonical_ohlcv.csv");
    write_canonical_csv(&canonical_csv_path, &candles)?;

    let mut reason_codes = vec![ReasonCode::DataGoKrFixtureParsed];
    let status = if !env_var_present(&config.service_key_env_var) {
        reason_codes.push(ReasonCode::MissingAuth);
        DataGoKrProviderStatus::MissingAuth
    } else if config
        .endpoint_profile
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        reason_codes.push(ReasonCode::DataGoKrEndpointProfileMissing);
        DataGoKrProviderStatus::DeferredUntilEndpointProfile
    } else {
        DataGoKrProviderStatus::Ready
    };

    Ok(DataGoKrFscStockPriceImportReport {
        import_id: config.import_id.clone(),
        provider_kind: ProviderKind::DataGoKrFscStockPrice,
        status,
        symbol: config.symbol.clone(),
        canonical_csv_path: canonical_csv_path.to_string_lossy().to_string(),
        row_count: candles.len(),
        reason_codes,
    })
}

fn write_canonical_csv(path: &Path, candles: &[Candle]) -> Result<(), String> {
    let mut text = "timestamp_ms,open,high,low,close,volume\n".to_string();
    for candle in candles {
        text.push_str(&format!(
            "{},{:.8},{:.8},{:.8},{:.8},{:.8}\n",
            candle.timestamp_ms, candle.open, candle.high, candle.low, candle.close, candle.volume
        ));
    }
    fs::write(path, text).map_err(|err| err.to_string())
}

fn parse_number(value: &str) -> Result<f64, String> {
    value
        .replace(',', "")
        .parse::<f64>()
        .map_err(|err| err.to_string())
}

fn env_var_present(name: &str) -> bool {
    env::var_os(name)
        .map(|value| !value.is_empty())
        .unwrap_or(false)
}

fn default_service_key_env() -> String {
    "DATA_GO_KR_SERVICE_KEY".to_string()
}

fn parse_yyyymmdd_to_timestamp_ms(value: &str) -> Result<u64, String> {
    if value.len() != 8 || !value.chars().all(|character| character.is_ascii_digit()) {
        return Err(format!("invalid yyyymmdd date: {value}"));
    }
    let year = value[0..4].parse::<i32>().map_err(|err| err.to_string())?;
    let month = value[4..6].parse::<u32>().map_err(|err| err.to_string())?;
    let day = value[6..8].parse::<u32>().map_err(|err| err.to_string())?;
    let days = days_from_civil(year, month, day);
    u64::try_from(days.saturating_mul(86_400_000)).map_err(|_| "timestamp overflow".to_string())
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let mut year = year as i64;
    let month = month as i64;
    let day = day as i64;
    year -= if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn default_datagokr_output_dir(root: &Path) -> PathBuf {
    root.join("soma_datagokr")
}
