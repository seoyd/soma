use std::collections::{BTreeMap, BTreeSet};

use crate::backtest::Timeframe;
use crate::core::ReasonCode;
use crate::eval::{DatasetFrame, DatasetSplitKind, FeatureSchema};

use super::meta::ModelArtifactMeta;
use super::prediction::{
    PredictionFrame, PredictionImportConfig, PredictionRow, PredictionValidationResult,
};

pub fn prediction_frame_from_rows(
    model_meta: ModelArtifactMeta,
    rows: Vec<PredictionRow>,
    dataset: &DatasetFrame,
    feature_schema: &FeatureSchema,
    config: &PredictionImportConfig,
) -> PredictionFrame {
    let schema_validation =
        validate_prediction_rows(&model_meta, &rows, dataset, feature_schema, config);
    let reason_codes = schema_validation.reason_codes.clone();
    PredictionFrame {
        model_meta,
        rows,
        schema_validation,
        reason_codes,
    }
}

pub fn prediction_frame_to_csv_string(frame: &PredictionFrame) -> String {
    let header = [
        "row_id",
        "symbol",
        "timestamp_ms",
        "timeframe",
        "fold_id",
        "split_kind",
        "model_id",
        "p_win",
        "p_stop",
        "expected_return",
        "expected_drawdown",
        "confidence",
        "no_trade_probability",
        "horizon_bars",
        "reason_codes",
    ]
    .join(",");

    let mut lines = vec![header];
    for row in &frame.rows {
        lines.push(
            [
                row.row_id.clone(),
                row.symbol.clone(),
                row.timestamp_ms.to_string(),
                format!("{:?}", row.timeframe),
                row.fold_id
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                row.split_kind
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default(),
                row.model_id.clone(),
                format!("{:.8}", row.p_win),
                format!("{:.8}", row.p_stop),
                format!("{:.8}", row.expected_return),
                format!("{:.8}", row.expected_drawdown),
                format!("{:.8}", row.confidence),
                format!("{:.8}", row.no_trade_probability),
                row.horizon_bars.to_string(),
                row.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|"),
            ]
            .join(","),
        );
    }
    lines.join("\n")
}

pub fn prediction_frame_from_csv_string(
    csv: &str,
    model_meta: ModelArtifactMeta,
    dataset: &DatasetFrame,
    feature_schema: &FeatureSchema,
    config: &PredictionImportConfig,
) -> PredictionFrame {
    let mut rows = Vec::new();
    let mut parse_reason_codes = Vec::new();
    let mut lines = csv.lines();
    let Some(header_line) = lines.next() else {
        let mut validation = PredictionValidationResult::default();
        validation.reason_codes = vec![
            ReasonCode::PredictionParseFailed,
            ReasonCode::MissingRequiredColumn,
        ];
        return PredictionFrame {
            model_meta,
            rows,
            schema_validation: validation.clone(),
            reason_codes: validation.reason_codes,
        };
    };

    let header = header_line.split(',').collect::<Vec<_>>();
    let index_map = header
        .iter()
        .enumerate()
        .map(|(index, column)| ((*column).to_string(), index))
        .collect::<BTreeMap<_, _>>();
    for required in required_columns() {
        if !index_map.contains_key(required) {
            let mut validation = PredictionValidationResult::default();
            validation.reason_codes = vec![
                ReasonCode::MissingRequiredColumn,
                ReasonCode::PredictionParseFailed,
            ];
            return PredictionFrame {
                model_meta,
                rows,
                schema_validation: validation.clone(),
                reason_codes: validation.reason_codes,
            };
        }
    }

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split(',').collect::<Vec<_>>();
        match parse_row(&columns, &index_map) {
            Ok(row) => rows.push(row),
            Err(mut errors) => parse_reason_codes.append(&mut errors),
        }
    }

    let mut frame = prediction_frame_from_rows(model_meta, rows, dataset, feature_schema, config);
    for reason in parse_reason_codes {
        if !frame.reason_codes.contains(&reason) {
            frame.reason_codes.push(reason.clone());
        }
        if !frame.schema_validation.reason_codes.contains(&reason) {
            frame.schema_validation.reason_codes.push(reason);
        }
    }
    frame.schema_validation.valid = frame.schema_validation.valid
        && !frame
            .reason_codes
            .contains(&ReasonCode::PredictionParseFailed);
    frame
}

fn validate_prediction_rows(
    model_meta: &ModelArtifactMeta,
    rows: &[PredictionRow],
    dataset: &DatasetFrame,
    feature_schema: &FeatureSchema,
    config: &PredictionImportConfig,
) -> PredictionValidationResult {
    let expected_rows = dataset
        .rows
        .iter()
        .filter(|row| {
            matches!(
                row.split_kind,
                DatasetSplitKind::Validation | DatasetSplitKind::Test
            )
        })
        .collect::<Vec<_>>();
    let expected_map = expected_rows
        .iter()
        .map(|row| (row.row_id.clone(), *row))
        .collect::<BTreeMap<_, _>>();
    let provided_map = rows
        .iter()
        .map(|row| (row.row_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let missing_row_count = expected_map
        .keys()
        .filter(|row_id| !provided_map.contains_key(*row_id))
        .count();
    let extra_row_count = provided_map
        .keys()
        .filter(|row_id| !expected_map.contains_key(*row_id))
        .count();
    let timestamp_mismatch_count = rows
        .iter()
        .filter(|row| {
            expected_map
                .get(&row.row_id)
                .map(|expected| {
                    expected.timestamp_ms != row.timestamp_ms
                        || expected.symbol != row.symbol
                        || expected.timeframe != row.timeframe
                })
                .unwrap_or(false)
        })
        .count();
    let invalid_probability_count = rows
        .iter()
        .filter(|row| {
            row.validation_errors()
                .contains(&ReasonCode::InvalidProbability)
        })
        .count();
    let nan_or_inf_count = rows
        .iter()
        .filter(|row| {
            row.validation_errors()
                .contains(&ReasonCode::InvalidPrediction)
        })
        .count();

    let schema_match = model_meta.feature_schema_version == feature_schema.schema_version;
    let feature_schema_hash_match = model_meta.feature_schema_hash == feature_schema.checksum;

    let mut reason_codes = Vec::new();
    if invalid_probability_count > 0 || nan_or_inf_count > 0 {
        reason_codes.push(ReasonCode::InvalidPrediction);
    }
    if config.require_feature_schema_match && (!schema_match || !feature_schema_hash_match) {
        reason_codes.push(ReasonCode::PredictionSchemaMismatch);
    }
    if config.require_row_alignment
        && (missing_row_count > config.max_missing_rows
            || extra_row_count > 0
            || timestamp_mismatch_count > 0)
    {
        reason_codes.push(ReasonCode::PredictionAlignmentMismatch);
        if missing_row_count > 0 {
            reason_codes.push(ReasonCode::MissingPredictionRows);
        }
        if extra_row_count > 0 {
            reason_codes.push(ReasonCode::ExtraPredictionRows);
        }
    }

    PredictionValidationResult {
        valid: invalid_probability_count == 0
            && nan_or_inf_count == 0
            && (!config.require_feature_schema_match
                || (schema_match && feature_schema_hash_match))
            && (!config.require_row_alignment
                || (missing_row_count <= config.max_missing_rows
                    && extra_row_count == 0
                    && timestamp_mismatch_count == 0)),
        row_count: rows.len(),
        missing_row_count,
        extra_row_count,
        schema_match,
        feature_schema_hash_match,
        invalid_probability_count,
        nan_or_inf_count,
        timestamp_mismatch_count,
        reason_codes,
    }
}

fn parse_row(
    columns: &[&str],
    index_map: &BTreeMap<String, usize>,
) -> Result<PredictionRow, Vec<ReasonCode>> {
    let fetch = |name: &str| -> Result<&str, Vec<ReasonCode>> {
        index_map
            .get(name)
            .and_then(|index| columns.get(*index).copied())
            .ok_or_else(|| {
                vec![
                    ReasonCode::MissingRequiredColumn,
                    ReasonCode::PredictionParseFailed,
                ]
            })
    };

    let row_id = fetch("row_id")?.to_string();
    let symbol = fetch("symbol")?.to_string();
    let timestamp_ms = fetch("timestamp_ms")?
        .parse::<u64>()
        .map_err(|_| vec![ReasonCode::PredictionParseFailed])?;
    let timeframe = parse_timeframe(fetch("timeframe")?)?;
    let fold_id = optional_parse(fetch("fold_id")?);
    let split_kind = parse_split_kind(fetch("split_kind")?)?;
    let model_id = fetch("model_id")?.to_string();
    let p_win = parse_f64(fetch("p_win")?)?;
    let p_stop = parse_f64(fetch("p_stop")?)?;
    let expected_return = parse_f64(fetch("expected_return")?)?;
    let expected_drawdown = parse_f64(fetch("expected_drawdown")?)?;
    let confidence = parse_f64(fetch("confidence")?)?;
    let no_trade_probability = parse_f64(fetch("no_trade_probability")?)?;
    let horizon_bars = fetch("horizon_bars")?
        .parse::<u32>()
        .map_err(|_| vec![ReasonCode::PredictionParseFailed])?;
    let reason_codes = optional_reason_codes(fetch("reason_codes").unwrap_or_default());

    let mut row = PredictionRow::new(
        row_id,
        symbol,
        timestamp_ms,
        timeframe,
        fold_id,
        split_kind,
        model_id,
        p_win,
        p_stop,
        expected_return,
        expected_drawdown,
        confidence,
        no_trade_probability,
        horizon_bars,
    )?;
    row.reason_codes = reason_codes;
    Ok(row)
}

fn parse_f64(value: &str) -> Result<f64, Vec<ReasonCode>> {
    value
        .parse::<f64>()
        .map_err(|_| vec![ReasonCode::PredictionParseFailed])
}

fn parse_timeframe(value: &str) -> Result<Timeframe, Vec<ReasonCode>> {
    match value {
        "OneMinute" => Ok(Timeframe::OneMinute),
        "FiveMinute" => Ok(Timeframe::FiveMinute),
        "FifteenMinute" => Ok(Timeframe::FifteenMinute),
        "OneHour" => Ok(Timeframe::OneHour),
        "OneDay" => Ok(Timeframe::OneDay),
        _ => Err(vec![ReasonCode::PredictionParseFailed]),
    }
}

fn parse_split_kind(value: &str) -> Result<Option<DatasetSplitKind>, Vec<ReasonCode>> {
    if value.is_empty() {
        return Ok(None);
    }
    match value {
        "Train" => Ok(Some(DatasetSplitKind::Train)),
        "Validation" => Ok(Some(DatasetSplitKind::Validation)),
        "Test" => Ok(Some(DatasetSplitKind::Test)),
        "Embargo" => Ok(Some(DatasetSplitKind::Embargo)),
        "Unsafe" => Ok(Some(DatasetSplitKind::Unsafe)),
        _ => Err(vec![ReasonCode::PredictionParseFailed]),
    }
}

fn optional_parse(value: &str) -> Option<usize> {
    if value.is_empty() {
        None
    } else {
        value.parse::<usize>().ok()
    }
}

fn optional_reason_codes(value: &str) -> Vec<ReasonCode> {
    if value.is_empty() {
        Vec::new()
    } else {
        value
            .split('|')
            .filter_map(parse_reason_code)
            .collect::<Vec<_>>()
    }
}

fn parse_reason_code(value: &str) -> Option<ReasonCode> {
    match value {
        "MissingPrediction" => Some(ReasonCode::MissingPrediction),
        "InvalidPrediction" => Some(ReasonCode::InvalidPrediction),
        "InvalidProbability" => Some(ReasonCode::InvalidProbability),
        _ => None,
    }
}

fn required_columns() -> BTreeSet<&'static str> {
    [
        "row_id",
        "symbol",
        "timestamp_ms",
        "timeframe",
        "fold_id",
        "split_kind",
        "model_id",
        "p_win",
        "p_stop",
        "expected_return",
        "expected_drawdown",
        "confidence",
        "no_trade_probability",
        "horizon_bars",
        "reason_codes",
    ]
    .into_iter()
    .collect()
}
