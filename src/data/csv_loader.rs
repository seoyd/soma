use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::backtest::{Candle, CandleSeries};
use crate::core::ReasonCode;

use super::csv_format::{CandleCsvConfig, CandleCsvFormat, TimestampFormat, logical_column_map};
use super::quality::{DataQualityReport, build_data_quality_report};
use super::symbol::{MarketVenue, SymbolRegistry, SymbolSpec};
use super::timeframe::TimeframeSpec;
use super::validation::{
    CandleParseError, CandleParseIssue, DataValidationConfig, ValidationStats,
    detect_temporal_issues, validate_candle,
};
use super::{DataManifest, DataProvenance, infer_source_kind_from_path};

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedCandleData {
    pub series: CandleSeries,
    pub symbol_spec: SymbolSpec,
    pub timeframe_spec: TimeframeSpec,
    pub quality_report: DataQualityReport,
    pub manifest: DataManifest,
    pub parse_issues: Vec<CandleParseIssue>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CandleLoadFailure {
    pub issues: Vec<CandleParseIssue>,
    pub reason_codes: Vec<ReasonCode>,
    pub quality_report: Option<DataQualityReport>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CandleCsvLoader {
    pub registry: SymbolRegistry,
    pub validation: DataValidationConfig,
}

impl CandleCsvLoader {
    pub fn load_from_path(
        &self,
        path: &Path,
        config: &CandleCsvConfig,
    ) -> Result<LoadedCandleData, CandleLoadFailure> {
        let path_string = path.to_string_lossy().to_string();
        if path_string.starts_with("http://") || path_string.starts_with("https://") {
            return Err(CandleLoadFailure {
                issues: Vec::new(),
                reason_codes: vec![ReasonCode::LocalFileOnly],
                quality_report: None,
            });
        }
        let contents = fs::read_to_string(path).map_err(|_| CandleLoadFailure {
            issues: Vec::new(),
            reason_codes: vec![ReasonCode::LocalFileOnly],
            quality_report: None,
        })?;
        self.load_from_str(&contents, config, Some(path))
    }

    pub fn load_from_str(
        &self,
        input: &str,
        config: &CandleCsvConfig,
        source_path: Option<&Path>,
    ) -> Result<LoadedCandleData, CandleLoadFailure> {
        if matches!(config.timestamp_format, TimestampFormat::CustomUnsupported) {
            return Err(CandleLoadFailure {
                issues: vec![CandleParseIssue {
                    row_number: None,
                    column: Some("timestamp".to_string()),
                    value: None,
                    error: CandleParseError::UnsupportedFormat,
                    reason_codes: vec![ReasonCode::UnsupportedTimestampFormat],
                }],
                reason_codes: vec![ReasonCode::UnsupportedTimestampFormat],
                quality_report: None,
            });
        }

        let timeframe_spec = TimeframeSpec::from_timeframe(config.timeframe);
        if !timeframe_spec.is_supported() {
            return Err(CandleLoadFailure {
                issues: vec![CandleParseIssue {
                    row_number: None,
                    column: Some("timeframe".to_string()),
                    value: Some(format!("{:?}", config.timeframe)),
                    error: CandleParseError::UnsupportedFormat,
                    reason_codes: vec![ReasonCode::UnsupportedTimeframe],
                }],
                reason_codes: vec![ReasonCode::UnsupportedTimeframe],
                quality_report: None,
            });
        }

        let symbol_spec = self
            .registry
            .lookup_symbol(&config.symbol)
            .cloned()
            .unwrap_or_else(|| {
                SymbolSpec::guessed(config.symbol.clone(), venue_for_format(&config.format))
            });
        let delimiter = config.delimiter;
        let mut lines = input
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>();
        if lines.is_empty() {
            return Err(CandleLoadFailure {
                issues: vec![CandleParseIssue {
                    row_number: None,
                    column: None,
                    value: None,
                    error: CandleParseError::MissingColumn,
                    reason_codes: vec![ReasonCode::MissingRequiredColumn],
                }],
                reason_codes: vec![ReasonCode::MissingRequiredColumn],
                quality_report: None,
            });
        }

        let header = if config.has_header {
            lines
                .remove(0)
                .split(delimiter)
                .map(|value| value.trim().to_string())
                .collect::<Vec<_>>()
        } else {
            let column_map = logical_column_map(&config.format);
            vec![
                column_map.timestamp,
                column_map.open,
                column_map.high,
                column_map.low,
                column_map.close,
                column_map.volume,
            ]
        };
        let column_index = header
            .iter()
            .enumerate()
            .map(|(index, name)| (name.as_str(), index))
            .collect::<BTreeMap<_, _>>();
        let column_map = logical_column_map(&config.format);
        let Some(required_columns) = resolve_required_columns(&column_index, &column_map) else {
            return Err(CandleLoadFailure {
                issues: vec![CandleParseIssue {
                    row_number: None,
                    column: None,
                    value: None,
                    error: CandleParseError::MissingColumn,
                    reason_codes: vec![ReasonCode::MissingRequiredColumn],
                }],
                reason_codes: vec![ReasonCode::MissingRequiredColumn],
                quality_report: None,
            });
        };

        let optional_columns = resolve_optional_columns(&column_index, &column_map);
        let mut stats = ValidationStats {
            input_row_count: lines.len(),
            reason_codes: config.reason_codes.clone(),
            ..ValidationStats::default()
        };
        let validation = DataValidationConfig {
            strict: config.strict && self.validation.strict,
            allow_sort_repair: config.allow_repair_sort || self.validation.allow_sort_repair,
            allow_duplicate_drop: self.validation.allow_duplicate_drop,
            allow_gap: self.validation.allow_gap,
            max_gap_count: self.validation.max_gap_count,
            max_gap_ratio: self.validation.max_gap_ratio,
            max_invalid_ratio: self.validation.max_invalid_ratio,
            expected_step_ms: self
                .validation
                .expected_step_ms
                .or(Some(timeframe_spec.expected_ms_step)),
            reason_codes: self.validation.reason_codes.clone(),
        };

        let mut parse_issues = Vec::new();
        let mut candles = Vec::new();
        for (offset, line) in lines.iter().enumerate() {
            let row_number = offset + if config.has_header { 2 } else { 1 };
            let columns = line.split(delimiter).map(str::trim).collect::<Vec<_>>();
            match parse_candle(
                &columns,
                row_number,
                config,
                &required_columns,
                &optional_columns,
            ) {
                Ok(candle) => {
                    let issues = validate_candle(&candle, row_number);
                    if issues.is_empty() {
                        stats.observe_valid_candle(&candle);
                        candles.push(candle);
                    } else {
                        for issue in issues {
                            stats.observe_issue(&issue);
                            parse_issues.push(issue);
                        }
                    }
                }
                Err(issue) => {
                    stats.observe_issue(&issue);
                    parse_issues.push(issue);
                }
            }
        }

        if !parse_issues.is_empty() && !config.allow_drop_invalid_rows {
            let mut reason_codes = parse_issues
                .iter()
                .flat_map(|issue| issue.reason_codes.iter().cloned())
                .collect::<Vec<_>>();
            if reason_codes.is_empty() {
                reason_codes.push(ReasonCode::InvalidRowDropped);
            }
            return Err(CandleLoadFailure {
                issues: parse_issues,
                reason_codes,
                quality_report: None,
            });
        }

        if parse_issues.len() > config.max_invalid_rows && config.allow_drop_invalid_rows {
            parse_issues.push(CandleParseIssue {
                row_number: None,
                column: None,
                value: Some(parse_issues.len().to_string()),
                error: CandleParseError::TooManyInvalidRows,
                reason_codes: vec![ReasonCode::InvalidRowDropped],
            });
            let quality_report = Some(build_data_quality_report(
                symbol_spec.normalized_symbol.clone(),
                config.timeframe,
                &stats,
            ));
            return Err(CandleLoadFailure {
                issues: parse_issues,
                reason_codes: vec![ReasonCode::InvalidRowDropped],
                quality_report,
            });
        }

        stats.dropped_row_count = parse_issues.len();
        let (_, out_of_order, _, _) = detect_temporal_issues(&candles, validation.expected_step_ms);
        if out_of_order > 0 {
            stats.out_of_order_count = out_of_order;
            if validation.allow_sort_repair {
                candles.sort_by_key(|candle| candle.timestamp_ms);
                stats.repaired_row_count += out_of_order;
                stats.reason_codes.push(ReasonCode::CsvSortedRepairApplied);
            } else {
                return Err(CandleLoadFailure {
                    issues: vec![CandleParseIssue {
                        row_number: None,
                        column: Some("timestamp_ms".to_string()),
                        value: None,
                        error: CandleParseError::OutOfOrderTimestamp,
                        reason_codes: vec![ReasonCode::OutOfOrderTimestampDetected],
                    }],
                    reason_codes: vec![ReasonCode::OutOfOrderTimestampDetected],
                    quality_report: None,
                });
            }
        }

        let (duplicates, _, gaps, max_gap_ms) =
            detect_temporal_issues(&candles, validation.expected_step_ms);
        stats.duplicate_timestamp_count = duplicates;
        stats.gap_count = gaps;
        stats.max_gap_ms = max_gap_ms;
        if duplicates > 0 {
            if validation.allow_duplicate_drop {
                let original_len = candles.len();
                candles.dedup_by_key(|candle| candle.timestamp_ms);
                let dropped = original_len.saturating_sub(candles.len());
                stats.dropped_row_count += dropped;
                stats.repaired_row_count += dropped;
                stats
                    .reason_codes
                    .push(ReasonCode::DuplicateTimestampDropped);
            } else {
                return Err(CandleLoadFailure {
                    issues: vec![CandleParseIssue {
                        row_number: None,
                        column: Some("timestamp_ms".to_string()),
                        value: None,
                        error: CandleParseError::DuplicateTimestamp,
                        reason_codes: vec![ReasonCode::DuplicateTimestampDetected],
                    }],
                    reason_codes: vec![ReasonCode::DuplicateTimestampDetected],
                    quality_report: None,
                });
            }
        }
        stats.valid_row_count = candles.len();

        let gap_ratio = if candles.len() > 1 {
            stats.gap_count as f64 / (candles.len() - 1) as f64
        } else {
            0.0
        };
        if (!validation.allow_gap && gaps > 0)
            || gaps > validation.max_gap_count
            || gap_ratio > validation.max_gap_ratio
        {
            stats.reason_codes.push(ReasonCode::GapDetected);
        }
        let invalid_ratio = if stats.input_row_count > 0 {
            stats.invalid_row_count as f64 / stats.input_row_count as f64
        } else {
            1.0
        };
        if invalid_ratio > validation.max_invalid_ratio {
            stats.reason_codes.push(ReasonCode::InvalidRowDropped);
        }

        let quality_report = build_data_quality_report(
            symbol_spec.normalized_symbol.clone(),
            config.timeframe,
            &stats,
        );
        if candles.is_empty() {
            return Err(CandleLoadFailure {
                issues: parse_issues,
                reason_codes: quality_report.reason_codes.clone(),
                quality_report: Some(quality_report),
            });
        }

        let series = CandleSeries {
            symbol: symbol_spec.normalized_symbol.clone(),
            timeframe: config.timeframe,
            candles,
        };
        let manifest = DataManifest::build(
            &series,
            &symbol_spec,
            &timeframe_spec,
            &quality_report,
            infer_source_kind_from_path(source_path),
            source_path.map(|path| path.to_string_lossy().to_string()),
            Some(DataProvenance::inferred_from_path(
                source_path.and_then(|path| path.to_str()),
            )),
            None,
        );
        let mut reason_codes = quality_report.reason_codes.clone();
        reason_codes.extend(manifest.reason_codes.iter().cloned());
        reason_codes.push(ReasonCode::CsvLoaded);
        Ok(LoadedCandleData {
            series,
            symbol_spec,
            timeframe_spec,
            quality_report,
            manifest,
            parse_issues,
            reason_codes,
        })
    }
}

fn resolve_required_columns(
    header: &BTreeMap<&str, usize>,
    column_map: &super::csv_format::CustomColumnMap,
) -> Option<[usize; 6]> {
    Some([
        *header.get(column_map.timestamp.as_str())?,
        *header.get(column_map.open.as_str())?,
        *header.get(column_map.high.as_str())?,
        *header.get(column_map.low.as_str())?,
        *header.get(column_map.close.as_str())?,
        *header.get(column_map.volume.as_str())?,
    ])
}

fn resolve_optional_columns(
    header: &BTreeMap<&str, usize>,
    column_map: &super::csv_format::CustomColumnMap,
) -> [Option<usize>; 4] {
    [
        column_map
            .trade_value
            .as_ref()
            .and_then(|name| header.get(name.as_str()).copied()),
        column_map
            .bid
            .as_ref()
            .and_then(|name| header.get(name.as_str()).copied()),
        column_map
            .ask
            .as_ref()
            .and_then(|name| header.get(name.as_str()).copied()),
        column_map
            .spread_bps
            .as_ref()
            .and_then(|name| header.get(name.as_str()).copied()),
    ]
}

fn parse_candle(
    columns: &[&str],
    row_number: usize,
    config: &CandleCsvConfig,
    required: &[usize; 6],
    optional: &[Option<usize>; 4],
) -> Result<Candle, CandleParseIssue> {
    let timestamp = parse_timestamp(value_at(columns, required[0]), config.timestamp_format)
        .ok_or_else(|| CandleParseIssue {
            row_number: Some(row_number),
            column: Some("timestamp".to_string()),
            value: value_at(columns, required[0]).map(|value| value.to_string()),
            error: CandleParseError::InvalidTimestamp,
            reason_codes: vec![ReasonCode::OutOfOrderTimestampDetected],
        })?;
    let open = parse_number(value_at(columns, required[1]), row_number, "open")?;
    let high = parse_number(value_at(columns, required[2]), row_number, "high")?;
    let low = parse_number(value_at(columns, required[3]), row_number, "low")?;
    let close = parse_number(value_at(columns, required[4]), row_number, "close")?;
    let volume = parse_number(value_at(columns, required[5]), row_number, "volume")?;
    let trade_value = parse_optional_number(columns, optional[0], row_number, "trade_value")?;
    let bid = parse_optional_number(columns, optional[1], row_number, "bid")?;
    let ask = parse_optional_number(columns, optional[2], row_number, "ask")?;
    let spread_bps = parse_optional_number(columns, optional[3], row_number, "spread_bps")?;
    Ok(Candle {
        timestamp_ms: timestamp,
        open,
        high,
        low,
        close,
        volume,
        trade_value,
        bid,
        ask,
        spread_bps,
    })
}

fn parse_timestamp(value: Option<&str>, format: TimestampFormat) -> Option<u64> {
    let value = value?;
    match format {
        TimestampFormat::Millis => value.parse().ok(),
        TimestampFormat::Seconds => value.parse::<u64>().ok().map(|seconds| seconds * 1_000),
        TimestampFormat::Iso8601Utc | TimestampFormat::CustomUnsupported => None,
    }
}

fn parse_number(
    value: Option<&str>,
    row_number: usize,
    column: &str,
) -> Result<f64, CandleParseIssue> {
    value
        .ok_or_else(|| CandleParseIssue {
            row_number: Some(row_number),
            column: Some(column.to_string()),
            value: None,
            error: CandleParseError::MissingColumn,
            reason_codes: vec![ReasonCode::MissingRequiredColumn],
        })?
        .parse::<f64>()
        .map_err(|_| CandleParseIssue {
            row_number: Some(row_number),
            column: Some(column.to_string()),
            value: value.map(|entry| entry.to_string()),
            error: CandleParseError::InvalidNumber,
            reason_codes: vec![ReasonCode::InvalidRowDropped],
        })
}

fn parse_optional_number(
    columns: &[&str],
    index: Option<usize>,
    row_number: usize,
    column: &str,
) -> Result<Option<f64>, CandleParseIssue> {
    let Some(index) = index else {
        return Ok(None);
    };
    let Some(value) = columns.get(index).copied() else {
        return Ok(None);
    };
    if value.is_empty() {
        return Ok(None);
    }
    value
        .parse::<f64>()
        .map(Some)
        .map_err(|_| CandleParseIssue {
            row_number: Some(row_number),
            column: Some(column.to_string()),
            value: Some(value.to_string()),
            error: CandleParseError::InvalidNumber,
            reason_codes: vec![ReasonCode::InvalidRowDropped],
        })
}

fn value_at<'a>(columns: &'a [&'a str], index: usize) -> Option<&'a str> {
    columns.get(index).copied()
}

fn venue_for_format(format: &CandleCsvFormat) -> MarketVenue {
    match format {
        CandleCsvFormat::GenericOhlcv | CandleCsvFormat::Custom { .. } => MarketVenue::Generic,
        CandleCsvFormat::BinanceKline => MarketVenue::Binance,
        CandleCsvFormat::UpbitCandle => MarketVenue::Upbit,
        CandleCsvFormat::KrxOhlcv => MarketVenue::KRX,
    }
}
