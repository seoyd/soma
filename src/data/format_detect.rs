use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::{CandleCsvFormat, CustomColumnMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CsvFormatDetectionConfidence {
    High,
    Medium,
    Low,
    Ambiguous,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CsvFormatCandidateMapping {
    pub format_name: String,
    pub matched_columns: Vec<String>,
    pub missing_required_columns: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CsvFormatDetectionResult {
    #[serde(default)]
    pub detected_format: Option<CandleCsvFormat>,
    pub confidence: CsvFormatDetectionConfidence,
    pub header_present: bool,
    pub detected_columns: Vec<String>,
    pub missing_required_columns: Vec<String>,
    pub candidate_mappings: Vec<CsvFormatCandidateMapping>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CsvFormatDetector;

impl CsvFormatDetector {
    pub fn detect_from_path(
        &self,
        path: &Path,
        custom_map: Option<&CustomColumnMap>,
    ) -> Result<CsvFormatDetectionResult, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Ok(self.detect_from_str(&text, custom_map))
    }

    pub fn detect_from_str(
        &self,
        input: &str,
        custom_map: Option<&CustomColumnMap>,
    ) -> CsvFormatDetectionResult {
        let mut lines = input.lines().filter(|line| !line.trim().is_empty());
        let Some(first_line) = lines.next() else {
            return CsvFormatDetectionResult {
                detected_format: None,
                confidence: CsvFormatDetectionConfidence::Unsupported,
                header_present: false,
                detected_columns: Vec::new(),
                missing_required_columns: Vec::new(),
                candidate_mappings: Vec::new(),
                reason_codes: vec![ReasonCode::UnsupportedCsvFormat],
            };
        };
        let raw_columns = first_line
            .split(',')
            .map(|value| value.trim().to_string())
            .collect::<Vec<_>>();
        let header_present = !raw_columns
            .iter()
            .all(|value| value.parse::<f64>().is_ok() || value.is_empty());
        if !header_present {
            return CsvFormatDetectionResult {
                detected_format: None,
                confidence: CsvFormatDetectionConfidence::Unsupported,
                header_present: false,
                detected_columns: raw_columns,
                missing_required_columns: Vec::new(),
                candidate_mappings: Vec::new(),
                reason_codes: vec![ReasonCode::UnsupportedCsvFormat],
            };
        }

        let detected_columns = raw_columns
            .iter()
            .map(|value| normalize_column(value))
            .collect::<Vec<_>>();
        let header = detected_columns.iter().cloned().collect::<BTreeSet<_>>();

        if let Some(column_map) = custom_map {
            return self.validate_format(
                &header,
                CandleCsvFormat::Custom {
                    column_map: column_map.clone(),
                },
                true,
            );
        }

        let candidates = [
            CandleCsvFormat::GenericOhlcv,
            CandleCsvFormat::BinanceKline,
            CandleCsvFormat::UpbitCandle,
            CandleCsvFormat::KrxOhlcv,
        ]
        .into_iter()
        .map(|format| score_candidate(&header, &format))
        .collect::<Vec<_>>();
        build_detection_result(detected_columns, candidates)
    }

    pub fn validate_format_from_path(
        &self,
        path: &Path,
        format: CandleCsvFormat,
    ) -> Result<CsvFormatDetectionResult, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Ok(self.validate_format_from_str(&text, format))
    }

    pub fn validate_format_from_str(
        &self,
        input: &str,
        format: CandleCsvFormat,
    ) -> CsvFormatDetectionResult {
        let mut lines = input.lines().filter(|line| !line.trim().is_empty());
        let Some(first_line) = lines.next() else {
            return CsvFormatDetectionResult {
                detected_format: None,
                confidence: CsvFormatDetectionConfidence::Unsupported,
                header_present: false,
                detected_columns: Vec::new(),
                missing_required_columns: Vec::new(),
                candidate_mappings: Vec::new(),
                reason_codes: vec![ReasonCode::UnsupportedCsvFormat],
            };
        };
        let raw_columns = first_line
            .split(',')
            .map(|value| value.trim().to_string())
            .collect::<Vec<_>>();
        let detected_columns = raw_columns
            .iter()
            .map(|value| normalize_column(value))
            .collect::<Vec<_>>();
        let header = detected_columns.iter().cloned().collect::<BTreeSet<_>>();
        self.validate_format(&header, format, true)
    }

    fn validate_format(
        &self,
        header: &BTreeSet<String>,
        format: CandleCsvFormat,
        header_present: bool,
    ) -> CsvFormatDetectionResult {
        let candidate = score_candidate(header, &format);
        let mut result = CsvFormatDetectionResult {
            detected_format: candidate
                .missing_required_columns
                .is_empty()
                .then_some(format.clone()),
            confidence: if candidate.missing_required_columns.is_empty() {
                CsvFormatDetectionConfidence::High
            } else if candidate.matched_required_columns > 0 {
                CsvFormatDetectionConfidence::Low
            } else {
                CsvFormatDetectionConfidence::Unsupported
            },
            header_present,
            detected_columns: header.iter().cloned().collect(),
            missing_required_columns: candidate.missing_required_columns.clone(),
            candidate_mappings: vec![CsvFormatCandidateMapping {
                format_name: format_name(&format),
                matched_columns: candidate.matched_columns.clone(),
                missing_required_columns: candidate.missing_required_columns,
            }],
            reason_codes: vec![ReasonCode::CsvFormatDetected],
        };
        if matches!(format, CandleCsvFormat::Custom { .. }) {
            result.reason_codes.push(ReasonCode::CustomColumnMapApplied);
        }
        if result.detected_format.is_none() {
            result.reason_codes = vec![ReasonCode::MissingRequiredColumn];
        }
        result
    }
}

#[derive(Clone, Debug)]
struct ScoredCandidate {
    format: CandleCsvFormat,
    matched_columns: Vec<String>,
    missing_required_columns: Vec<String>,
    matched_required_columns: usize,
    score: usize,
}

fn score_candidate(header: &BTreeSet<String>, format: &CandleCsvFormat) -> ScoredCandidate {
    let (required, preferred) = profile_columns(format);
    let matched_columns = required
        .iter()
        .filter(|column| header.contains(column.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let missing_required_columns = required
        .iter()
        .filter(|column| !header.contains(column.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let matched_required_columns = required
        .len()
        .saturating_sub(missing_required_columns.len());
    let matched_preferred = preferred
        .iter()
        .filter(|column| header.contains(**column))
        .count();
    let signature_bonus = signature_bonus(header, format);
    ScoredCandidate {
        format: format.clone(),
        matched_columns,
        missing_required_columns,
        matched_required_columns,
        score: matched_required_columns * 10 + matched_preferred + signature_bonus,
    }
}

fn build_detection_result(
    detected_columns: Vec<String>,
    mut candidates: Vec<ScoredCandidate>,
) -> CsvFormatDetectionResult {
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then(
                left.missing_required_columns
                    .len()
                    .cmp(&right.missing_required_columns.len()),
            )
            .then(format_name(&left.format).cmp(&format_name(&right.format)))
    });
    let candidate_mappings = candidates
        .iter()
        .filter(|candidate| candidate.matched_required_columns > 0 || candidate.score > 0)
        .map(|candidate| CsvFormatCandidateMapping {
            format_name: format_name(&candidate.format),
            matched_columns: candidate.matched_columns.clone(),
            missing_required_columns: candidate.missing_required_columns.clone(),
        })
        .collect::<Vec<_>>();
    let Some(best) = candidates.first() else {
        return CsvFormatDetectionResult {
            detected_format: None,
            confidence: CsvFormatDetectionConfidence::Unsupported,
            header_present: true,
            detected_columns,
            missing_required_columns: Vec::new(),
            candidate_mappings,
            reason_codes: vec![ReasonCode::UnsupportedCsvFormat],
        };
    };
    if best.matched_required_columns == 0 {
        return CsvFormatDetectionResult {
            detected_format: None,
            confidence: CsvFormatDetectionConfidence::Unsupported,
            header_present: true,
            detected_columns,
            missing_required_columns: best.missing_required_columns.clone(),
            candidate_mappings,
            reason_codes: vec![ReasonCode::UnsupportedCsvFormat],
        };
    }
    if best.missing_required_columns.is_empty() {
        if let Some(second) = candidates.get(1)
            && second.score == best.score
            && second.missing_required_columns.is_empty()
        {
            return CsvFormatDetectionResult {
                detected_format: None,
                confidence: CsvFormatDetectionConfidence::Ambiguous,
                header_present: true,
                detected_columns,
                missing_required_columns: Vec::new(),
                candidate_mappings,
                reason_codes: vec![ReasonCode::CsvFormatAmbiguous],
            };
        }
        return CsvFormatDetectionResult {
            detected_format: Some(best.format.clone()),
            confidence: if best.score >= 75 {
                CsvFormatDetectionConfidence::High
            } else {
                CsvFormatDetectionConfidence::Medium
            },
            header_present: true,
            detected_columns,
            missing_required_columns: Vec::new(),
            candidate_mappings,
            reason_codes: vec![ReasonCode::CsvFormatDetected],
        };
    }
    if let Some(second) = candidates.get(1)
        && second.score == best.score
    {
        return CsvFormatDetectionResult {
            detected_format: None,
            confidence: CsvFormatDetectionConfidence::Ambiguous,
            header_present: true,
            detected_columns,
            missing_required_columns: best.missing_required_columns.clone(),
            candidate_mappings,
            reason_codes: vec![ReasonCode::CsvFormatAmbiguous],
        };
    }
    CsvFormatDetectionResult {
        detected_format: None,
        confidence: CsvFormatDetectionConfidence::Low,
        header_present: true,
        detected_columns,
        missing_required_columns: best.missing_required_columns.clone(),
        candidate_mappings,
        reason_codes: vec![ReasonCode::CsvFormatDetected],
    }
}

fn profile_columns(format: &CandleCsvFormat) -> (Vec<String>, &'static [&'static str]) {
    match format {
        CandleCsvFormat::GenericOhlcv => (
            ["timestamp_ms", "open", "high", "low", "close", "volume"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            &["trade_value", "bid", "ask", "spread_bps"],
        ),
        CandleCsvFormat::BinanceKline => (
            ["open_time", "open", "high", "low", "close", "volume"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            &[
                "quote_asset_volume",
                "close_time",
                "number_of_trades",
                "taker_buy_base_asset_volume",
            ],
        ),
        CandleCsvFormat::UpbitCandle => (
            [
                "timestamp_ms",
                "opening_price",
                "high_price",
                "low_price",
                "trade_price",
                "candle_acc_trade_volume",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            &["candle_acc_trade_price", "market"],
        ),
        CandleCsvFormat::KrxOhlcv => (
            [
                "timestamp_ms",
                "open",
                "high",
                "low",
                "close",
                "volume",
                "trade_value",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            &["amount"],
        ),
        CandleCsvFormat::Custom { column_map } => (
            vec![
                column_map.timestamp.clone(),
                column_map.open.clone(),
                column_map.high.clone(),
                column_map.low.clone(),
                column_map.close.clone(),
                column_map.volume.clone(),
            ]
            .into_iter()
            .map(|value| normalize_column(&value))
            .collect(),
            &[],
        ),
    }
}

fn signature_bonus(header: &BTreeSet<String>, format: &CandleCsvFormat) -> usize {
    match format {
        CandleCsvFormat::GenericOhlcv => {
            usize::from(header.contains("bid"))
                + usize::from(header.contains("ask"))
                + usize::from(header.contains("spread_bps"))
                + usize::from(header.contains("trade_value"))
                + 20 * usize::from(
                    header.contains("bid")
                        || header.contains("ask")
                        || header.contains("spread_bps"),
                )
        }
        CandleCsvFormat::BinanceKline => {
            15 * usize::from(header.contains("quote_asset_volume"))
                + usize::from(header.contains("close_time"))
                + usize::from(header.contains("number_of_trades"))
        }
        CandleCsvFormat::UpbitCandle => {
            15 * usize::from(header.contains("candle_acc_trade_price"))
                + usize::from(header.contains("market"))
        }
        CandleCsvFormat::KrxOhlcv => {
            10 * usize::from(
                header.contains("trade_value")
                    && !header.contains("bid")
                    && !header.contains("ask")
                    && !header.contains("spread_bps"),
            )
        }
        CandleCsvFormat::Custom { .. } => 20,
    }
}

fn format_name(format: &CandleCsvFormat) -> String {
    match format {
        CandleCsvFormat::Custom { .. } => "CustomColumnMap".to_string(),
        _ => format!("{format:?}"),
    }
}

fn normalize_column(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-', '/'], "_")
}
