use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::backtest::Candle;
use crate::core::ReasonCode;

use super::ProviderKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlpacaProviderStatus {
    Ready,
    MissingAuth,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlpacaHistoricalBarsImportConfig {
    pub import_id: String,
    pub fixture_path: String,
    pub output_root: String,
    pub symbol: String,
    #[serde(default = "default_alpaca_key_env")]
    pub api_key_env_var: String,
    #[serde(default = "default_alpaca_secret_env")]
    pub api_secret_env_var: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlpacaHistoricalBarsImportReport {
    pub import_id: String,
    pub provider_kind: ProviderKind,
    pub status: AlpacaProviderStatus,
    pub symbol: String,
    pub canonical_csv_path: String,
    pub row_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Deserialize)]
struct AlpacaBarsFixture {
    bars: Vec<AlpacaBarRow>,
}

#[derive(Clone, Debug, Deserialize)]
struct AlpacaBarRow {
    #[serde(rename = "t")]
    timestamp: String,
    #[serde(rename = "o")]
    open: f64,
    #[serde(rename = "h")]
    high: f64,
    #[serde(rename = "l")]
    low: f64,
    #[serde(rename = "c")]
    close: f64,
    #[serde(rename = "v")]
    volume: f64,
}

impl AlpacaHistoricalBarsImportConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }
}

pub fn parse_alpaca_historical_bars_fixture(input: &str) -> Result<Vec<Candle>, String> {
    let fixture =
        serde_json::from_str::<AlpacaBarsFixture>(input).map_err(|err| err.to_string())?;
    fixture
        .bars
        .into_iter()
        .map(|row| {
            Ok(Candle {
                timestamp_ms: parse_iso_date_to_timestamp_ms(&row.timestamp)?,
                open: row.open,
                high: row.high,
                low: row.low,
                close: row.close,
                volume: row.volume,
                trade_value: None,
                bid: None,
                ask: None,
                spread_bps: None,
            })
        })
        .collect()
}

pub fn run_alpaca_historical_bars_import(
    config: &AlpacaHistoricalBarsImportConfig,
) -> Result<AlpacaHistoricalBarsImportReport, String> {
    let fixture = fs::read_to_string(&config.fixture_path).map_err(|err| err.to_string())?;
    let mut candles = parse_alpaca_historical_bars_fixture(&fixture)?;
    if candles.len() > config.max_rows {
        candles.truncate(config.max_rows);
    }
    let output_dir = Path::new(&config.output_root)
        .join(&config.import_id)
        .join(&config.symbol);
    fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;
    let canonical_csv_path = output_dir.join("canonical_ohlcv.csv");
    write_canonical_csv(&canonical_csv_path, &candles)?;

    let mut reason_codes = vec![ReasonCode::AlpacaFixtureParsed];
    let status = if env_var_present(&config.api_key_env_var)
        && env_var_present(&config.api_secret_env_var)
    {
        AlpacaProviderStatus::Ready
    } else {
        reason_codes.push(ReasonCode::MissingAuth);
        AlpacaProviderStatus::MissingAuth
    };

    Ok(AlpacaHistoricalBarsImportReport {
        import_id: config.import_id.clone(),
        provider_kind: ProviderKind::Alpaca,
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

fn env_var_present(name: &str) -> bool {
    env::var_os(name)
        .map(|value| !value.is_empty())
        .unwrap_or(false)
}

fn default_alpaca_key_env() -> String {
    "ALPACA_API_KEY_ID".to_string()
}

fn default_alpaca_secret_env() -> String {
    "ALPACA_API_SECRET_KEY".to_string()
}

fn default_max_rows() -> usize {
    500
}

fn parse_iso_date_to_timestamp_ms(value: &str) -> Result<u64, String> {
    let value = value
        .strip_suffix("T00:00:00Z")
        .or_else(|| value.strip_suffix("Z").and_then(|raw| raw.get(..10)))
        .unwrap_or(value);
    if value.len() != 10 {
        return Err(format!("invalid ISO date: {value}"));
    }
    let year = value[0..4].parse::<i32>().map_err(|err| err.to_string())?;
    let month = value[5..7].parse::<u32>().map_err(|err| err.to_string())?;
    let day = value[8..10].parse::<u32>().map_err(|err| err.to_string())?;
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

pub fn default_alpaca_output_dir(root: &Path) -> PathBuf {
    root.join("soma_alpaca")
}
