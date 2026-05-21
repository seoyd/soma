use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_reason_codes};

use super::krx_raw_archive::{KRXRawResponseArchiveRecord, KRXRawResponseArchiveSummary};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXResponseSchemaStatus {
    SchemaValid,
    MissingRequiredField,
    UnexpectedFieldSet,
    EmptyResponse,
    BadDateField,
    BadPriceField,
    BadVolumeField,
    UnsupportedSchema,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KRXResponseSchemaDriftReport {
    pub report_id: String,
    pub records_checked: usize,
    pub valid_records: usize,
    pub invalid_records: usize,
    pub missing_fields: Vec<String>,
    pub unexpected_fields: Vec<String>,
    pub schema_status: KRXResponseSchemaStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl KRXResponseSchemaDriftReport {
    pub fn from_archive(summary: Option<&KRXRawResponseArchiveSummary>, report_id: &str) -> Self {
        let records = summary
            .map(|summary| summary.records.as_slice())
            .unwrap_or(&[]);
        Self::check_records(records, report_id)
    }

    pub fn check_records(records: &[KRXRawResponseArchiveRecord], report_id: &str) -> Self {
        if records.is_empty() {
            return Self {
                report_id: report_id.to_string(),
                records_checked: 0,
                valid_records: 0,
                invalid_records: 0,
                missing_fields: Vec::new(),
                unexpected_fields: Vec::new(),
                schema_status: KRXResponseSchemaStatus::DiagnosticOnly,
                reason_codes: vec![ReasonCode::ProviderResponseArchived],
            };
        }
        let mut valid_records = 0usize;
        let mut invalid_records = 0usize;
        let mut missing_fields = BTreeSet::new();
        let mut unexpected_fields = BTreeSet::new();
        let mut status = KRXResponseSchemaStatus::SchemaValid;
        let mut reason_codes = vec![ReasonCode::FeatureSchemaValidated];
        for record in records {
            match check_record(record) {
                Ok(record_unexpected) => {
                    valid_records += 1;
                    unexpected_fields.extend(record_unexpected);
                }
                Err((record_status, record_missing, record_unexpected, reason)) => {
                    invalid_records += 1;
                    status = choose_status(status, record_status);
                    missing_fields.extend(record_missing);
                    unexpected_fields.extend(record_unexpected);
                    reason_codes.push(reason);
                }
            }
        }
        if invalid_records == 0 && !unexpected_fields.is_empty() {
            status = KRXResponseSchemaStatus::UnexpectedFieldSet;
            reason_codes.push(ReasonCode::SchemaMismatch);
        } else if invalid_records == 0 {
            status = KRXResponseSchemaStatus::SchemaValid;
        }
        Self {
            report_id: report_id.to_string(),
            records_checked: records.len(),
            valid_records,
            invalid_records,
            missing_fields: missing_fields.into_iter().collect(),
            unexpected_fields: unexpected_fields.into_iter().collect(),
            schema_status: status,
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }

    pub fn to_text(&self) -> String {
        [
            format!("report_id={}", self.report_id),
            format!("records_checked={}", self.records_checked),
            format!("valid_records={}", self.valid_records),
            format!("invalid_records={}", self.invalid_records),
            format!("missing_fields={}", self.missing_fields.join("|")),
            format!("unexpected_fields={}", self.unexpected_fields.join("|")),
            format!("schema_status={:?}", self.schema_status),
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

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("krx_schema_drift_report.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_schema_drift_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

fn check_record(
    record: &KRXRawResponseArchiveRecord,
) -> Result<
    BTreeSet<String>,
    (
        KRXResponseSchemaStatus,
        BTreeSet<String>,
        BTreeSet<String>,
        ReasonCode,
    ),
> {
    let text = fs::read_to_string(&record.response_path).map_err(|_| {
        (
            KRXResponseSchemaStatus::UnsupportedSchema,
            BTreeSet::new(),
            BTreeSet::new(),
            ReasonCode::FeatureSchemaMismatch,
        )
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|_| {
        (
            KRXResponseSchemaStatus::UnsupportedSchema,
            BTreeSet::new(),
            BTreeSet::new(),
            ReasonCode::FeatureSchemaMismatch,
        )
    })?;
    let object = value.as_object().ok_or_else(|| {
        (
            KRXResponseSchemaStatus::UnsupportedSchema,
            BTreeSet::new(),
            BTreeSet::new(),
            ReasonCode::FeatureSchemaMismatch,
        )
    })?;
    let mut missing = BTreeSet::new();
    for required in ["symbol", "timeframe", "rows"] {
        if !object.contains_key(required) {
            missing.insert(required.to_string());
        }
    }
    if !missing.is_empty() {
        return Err((
            KRXResponseSchemaStatus::MissingRequiredField,
            missing,
            BTreeSet::new(),
            ReasonCode::MissingRequiredColumn,
        ));
    }
    let mut unexpected = object
        .keys()
        .filter(|key| !matches!(key.as_str(), "symbol" | "timeframe" | "rows"))
        .map(|key| format!("root:{key}"))
        .collect::<BTreeSet<_>>();
    let rows = object
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            (
                KRXResponseSchemaStatus::UnsupportedSchema,
                BTreeSet::new(),
                unexpected.clone(),
                ReasonCode::FeatureSchemaMismatch,
            )
        })?;
    if rows.is_empty() {
        return Err((
            KRXResponseSchemaStatus::EmptyResponse,
            BTreeSet::new(),
            unexpected,
            ReasonCode::MissingFile,
        ));
    }
    for row in rows {
        let row_object = row.as_object().ok_or_else(|| {
            (
                KRXResponseSchemaStatus::UnsupportedSchema,
                BTreeSet::new(),
                unexpected.clone(),
                ReasonCode::FeatureSchemaMismatch,
            )
        })?;
        for required in [
            "date",
            "open",
            "high",
            "low",
            "close",
            "volume",
            "trade_value",
            "bid",
            "ask",
            "spread_bps",
        ] {
            if !row_object.contains_key(required) {
                missing.insert(required.to_string());
            }
        }
        if !missing.is_empty() {
            return Err((
                KRXResponseSchemaStatus::MissingRequiredField,
                missing,
                unexpected,
                ReasonCode::MissingRequiredColumn,
            ));
        }
        let date_valid = row_object
            .get("date")
            .and_then(Value::as_str)
            .is_some_and(|value| {
                let bytes = value.as_bytes();
                bytes.len() == 10
                    && bytes[4] == b'-'
                    && bytes[7] == b'-'
                    && bytes
                        .iter()
                        .enumerate()
                        .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
            });
        if !date_valid {
            return Err((
                KRXResponseSchemaStatus::BadDateField,
                BTreeSet::new(),
                unexpected,
                ReasonCode::UnsupportedTimestampFormat,
            ));
        }
        for price_name in [
            "open",
            "high",
            "low",
            "close",
            "trade_value",
            "bid",
            "ask",
            "spread_bps",
        ] {
            let valid = row_object
                .get(price_name)
                .and_then(Value::as_f64)
                .is_some_and(|value| value >= 0.0);
            if !valid {
                return Err((
                    KRXResponseSchemaStatus::BadPriceField,
                    BTreeSet::new(),
                    unexpected,
                    ReasonCode::NonPositivePrice,
                ));
            }
        }
        let volume_valid = row_object
            .get("volume")
            .and_then(Value::as_f64)
            .or_else(|| {
                row_object
                    .get("volume")
                    .and_then(Value::as_i64)
                    .map(|value| value as f64)
            })
            .is_some_and(|value| value >= 0.0);
        if !volume_valid {
            return Err((
                KRXResponseSchemaStatus::BadVolumeField,
                BTreeSet::new(),
                unexpected,
                ReasonCode::NegativeVolumeDetected,
            ));
        }
        unexpected.extend(
            row_object
                .keys()
                .filter(|key| {
                    !matches!(
                        key.as_str(),
                        "date"
                            | "open"
                            | "high"
                            | "low"
                            | "close"
                            | "volume"
                            | "trade_value"
                            | "bid"
                            | "ask"
                            | "spread_bps"
                    )
                })
                .map(|key| format!("row:{key}")),
        );
    }
    Ok(unexpected)
}

fn choose_status(
    current: KRXResponseSchemaStatus,
    next: KRXResponseSchemaStatus,
) -> KRXResponseSchemaStatus {
    use KRXResponseSchemaStatus::*;
    match (current, next) {
        (UnsupportedSchema, _) | (_, UnsupportedSchema) => UnsupportedSchema,
        (MissingRequiredField, _) | (_, MissingRequiredField) => MissingRequiredField,
        (BadDateField, _) | (_, BadDateField) => BadDateField,
        (BadPriceField, _) | (_, BadPriceField) => BadPriceField,
        (BadVolumeField, _) | (_, BadVolumeField) => BadVolumeField,
        (EmptyResponse, _) | (_, EmptyResponse) => EmptyResponse,
        (_, value) => value,
    }
}
