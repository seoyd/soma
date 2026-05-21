use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::ReasonCode;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomColumnMap {
    pub timestamp: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub trade_value: Option<String>,
    pub bid: Option<String>,
    pub ask: Option<String>,
    pub spread_bps: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandleCsvFormat {
    GenericOhlcv,
    BinanceKline,
    UpbitCandle,
    KrxOhlcv,
    Custom { column_map: CustomColumnMap },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimestampFormat {
    Millis,
    Seconds,
    Iso8601Utc,
    CustomUnsupported,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleCsvConfig {
    pub format: CandleCsvFormat,
    pub symbol: String,
    pub timeframe: Timeframe,
    pub has_header: bool,
    pub delimiter: char,
    pub timestamp_format: TimestampFormat,
    pub strict: bool,
    pub allow_repair_sort: bool,
    pub allow_drop_invalid_rows: bool,
    pub max_invalid_rows: usize,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CandleCsvConfig {
    fn default() -> Self {
        Self {
            format: CandleCsvFormat::GenericOhlcv,
            symbol: "UNKNOWN".to_string(),
            timeframe: Timeframe::OneMinute,
            has_header: true,
            delimiter: ',',
            timestamp_format: TimestampFormat::Millis,
            strict: true,
            allow_repair_sort: false,
            allow_drop_invalid_rows: false,
            max_invalid_rows: 0,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

pub fn logical_column_map(format: &CandleCsvFormat) -> CustomColumnMap {
    match format {
        CandleCsvFormat::GenericOhlcv => CustomColumnMap {
            timestamp: "timestamp_ms".to_string(),
            open: "open".to_string(),
            high: "high".to_string(),
            low: "low".to_string(),
            close: "close".to_string(),
            volume: "volume".to_string(),
            trade_value: Some("trade_value".to_string()),
            bid: Some("bid".to_string()),
            ask: Some("ask".to_string()),
            spread_bps: Some("spread_bps".to_string()),
        },
        CandleCsvFormat::BinanceKline => CustomColumnMap {
            timestamp: "open_time".to_string(),
            open: "open".to_string(),
            high: "high".to_string(),
            low: "low".to_string(),
            close: "close".to_string(),
            volume: "volume".to_string(),
            trade_value: Some("quote_asset_volume".to_string()),
            bid: None,
            ask: None,
            spread_bps: None,
        },
        CandleCsvFormat::UpbitCandle => CustomColumnMap {
            timestamp: "timestamp_ms".to_string(),
            open: "opening_price".to_string(),
            high: "high_price".to_string(),
            low: "low_price".to_string(),
            close: "trade_price".to_string(),
            volume: "candle_acc_trade_volume".to_string(),
            trade_value: Some("candle_acc_trade_price".to_string()),
            bid: None,
            ask: None,
            spread_bps: None,
        },
        CandleCsvFormat::KrxOhlcv => CustomColumnMap {
            timestamp: "timestamp_ms".to_string(),
            open: "open".to_string(),
            high: "high".to_string(),
            low: "low".to_string(),
            close: "close".to_string(),
            volume: "volume".to_string(),
            trade_value: Some("trade_value".to_string()),
            bid: None,
            ask: None,
            spread_bps: None,
        },
        CandleCsvFormat::Custom { column_map } => column_map.clone(),
    }
}
