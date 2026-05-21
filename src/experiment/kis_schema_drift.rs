use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_reason_codes};

use super::kis_raw_archive::{KISRawResponseArchiveRecord, KISRawResponseArchiveSummary};
use super::kis_symbol_whitelist::KISMarket;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISResponseSchemaStatus {
    SchemaValid,
    MissingRequiredField,
    UnexpectedFieldSet,
    EmptyResponse,
    BadDateField,
    BadPriceField,
    BadVolumeField,
    UnsupportedSchema,
    EndpointDenied,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISResponseSchemaDriftReport {
    pub report_id: String,
    pub records_checked: usize,
    pub valid_records: usize,
    pub invalid_records: usize,
    pub domestic_records: usize,
    pub overseas_records: usize,
    pub missing_fields: Vec<String>,
    pub unexpected_fields: Vec<String>,
    pub schema_status: KISResponseSchemaStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl KISResponseSchemaDriftReport {
    pub fn from_archive(summary: Option<&KISRawResponseArchiveSummary>, report_id: &str) -> Self {
        let records = summary
            .map(|summary| summary.records.as_slice())
            .unwrap_or(&[]);
        Self::check_records(records, report_id)
    }

    pub fn check_records(records: &[KISRawResponseArchiveRecord], report_id: &str) -> Self {
        if records.is_empty() {
            return Self {
                report_id: report_id.to_string(),
                records_checked: 0,
                valid_records: 0,
                invalid_records: 0,
                domestic_records: 0,
                overseas_records: 0,
                missing_fields: Vec::new(),
                unexpected_fields: Vec::new(),
                schema_status: KISResponseSchemaStatus::DiagnosticOnly,
                reason_codes: vec![ReasonCode::KISSchemaDriftBuilt],
            };
        }
        let mut valid_records = 0usize;
        let mut invalid_records = 0usize;
        let mut domestic_records = 0usize;
        let mut overseas_records = 0usize;
        let mut missing_fields = BTreeSet::new();
        let mut unexpected_fields = BTreeSet::new();
        let mut status = KISResponseSchemaStatus::SchemaValid;
        let mut reason_codes = vec![ReasonCode::KISSchemaDriftBuilt];
        for record in records {
            if record.market == KISMarket::KoreanEquity {
                domestic_records += 1;
            } else {
                overseas_records += 1;
            }
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
            status = KISResponseSchemaStatus::UnexpectedFieldSet;
            reason_codes.push(ReasonCode::SchemaMismatch);
        } else if invalid_records == 0 {
            status = KISResponseSchemaStatus::SchemaValid;
        }
        Self {
            report_id: report_id.to_string(),
            records_checked: records.len(),
            valid_records,
            invalid_records,
            domestic_records,
            overseas_records,
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
            format!("domestic_records={}", self.domestic_records),
            format!("overseas_records={}", self.overseas_records),
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
        let text_path = output_dir.join("kis_schema_drift_report.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_schema_drift_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

fn check_record(
    record: &KISRawResponseArchiveRecord,
) -> Result<
    BTreeSet<String>,
    (
        KISResponseSchemaStatus,
        BTreeSet<String>,
        BTreeSet<String>,
        ReasonCode,
    ),
> {
    if record.endpoint_category.is_broker_surface() {
        return Err((
            KISResponseSchemaStatus::EndpointDenied,
            BTreeSet::new(),
            BTreeSet::new(),
            ReasonCode::KISEndpointDenied,
        ));
    }
    let text = fs::read_to_string(&record.response_path).map_err(|_| {
        (
            KISResponseSchemaStatus::UnsupportedSchema,
            BTreeSet::new(),
            BTreeSet::new(),
            ReasonCode::FeatureSchemaMismatch,
        )
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|_| {
        (
            KISResponseSchemaStatus::UnsupportedSchema,
            BTreeSet::new(),
            BTreeSet::new(),
            ReasonCode::FeatureSchemaMismatch,
        )
    })?;
    let object = value.as_object().ok_or_else(|| {
        (
            KISResponseSchemaStatus::UnsupportedSchema,
            BTreeSet::new(),
            BTreeSet::new(),
            ReasonCode::FeatureSchemaMismatch,
        )
    })?;
    if !object.contains_key("output1") {
        let mut missing = BTreeSet::new();
        missing.insert("output1".to_string());
        return Err((
            KISResponseSchemaStatus::MissingRequiredField,
            missing,
            BTreeSet::new(),
            ReasonCode::MissingRequiredColumn,
        ));
    }
    let mut unexpected = object
        .keys()
        .filter(|key| !matches!(key.as_str(), "output1" | "rt_cd" | "msg_cd" | "msg1"))
        .map(|key| format!("root:{key}"))
        .collect::<BTreeSet<_>>();
    let rows = object
        .get("output1")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            (
                KISResponseSchemaStatus::UnsupportedSchema,
                BTreeSet::new(),
                unexpected.clone(),
                ReasonCode::FeatureSchemaMismatch,
            )
        })?;
    if rows.is_empty() {
        return Err((
            KISResponseSchemaStatus::EmptyResponse,
            BTreeSet::new(),
            unexpected,
            ReasonCode::MissingFile,
        ));
    }
    for row in rows {
        let row_object = row.as_object().ok_or_else(|| {
            (
                KISResponseSchemaStatus::UnsupportedSchema,
                BTreeSet::new(),
                unexpected.clone(),
                ReasonCode::FeatureSchemaMismatch,
            )
        })?;
        let required = if record.market == KISMarket::KoreanEquity {
            domestic_required_fields()
        } else {
            overseas_required_fields()
        };
        let mut missing = BTreeSet::new();
        for field in &required {
            if !row_object.contains_key(*field) {
                missing.insert(field.to_string());
            }
        }
        if !missing.is_empty() {
            return Err((
                KISResponseSchemaStatus::MissingRequiredField,
                missing,
                unexpected,
                ReasonCode::MissingRequiredColumn,
            ));
        }
        let date_field = if record.market == KISMarket::KoreanEquity {
            "stck_bsop_date"
        } else {
            "xymd"
        };
        let date_value = row_object
            .get(date_field)
            .and_then(Value::as_str)
            .unwrap_or_default();
        let date_valid = if record.market == KISMarket::KoreanEquity {
            date_value.len() == 8
                && date_value
                    .chars()
                    .all(|character| character.is_ascii_digit())
        } else {
            let bytes = date_value.as_bytes();
            bytes.len() == 10
                && bytes[4] == b'-'
                && bytes[7] == b'-'
                && bytes
                    .iter()
                    .enumerate()
                    .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
        };
        if !date_valid {
            return Err((
                KISResponseSchemaStatus::BadDateField,
                BTreeSet::new(),
                unexpected,
                ReasonCode::UnsupportedTimestampFormat,
            ));
        }
        let (price_fields, volume_fields) = if record.market == KISMarket::KoreanEquity {
            (
                vec![
                    "stck_oprc",
                    "stck_hgpr",
                    "stck_lwpr",
                    "stck_clpr",
                    "bidp1",
                    "askp1",
                ],
                vec!["acml_vol", "acml_tr_pbmn"],
            )
        } else {
            (
                vec!["open", "high", "low", "clos", "bid", "ask"],
                vec!["tvol", "tamt"],
            )
        };
        for field in price_fields {
            let parsed = row_object.get(field).and_then(as_f64_string_or_number);
            if !parsed.is_some_and(|value| value >= 0.0) {
                return Err((
                    KISResponseSchemaStatus::BadPriceField,
                    BTreeSet::new(),
                    unexpected,
                    ReasonCode::NonPositivePrice,
                ));
            }
        }
        for field in volume_fields {
            let parsed = row_object.get(field).and_then(as_f64_string_or_number);
            if !parsed.is_some_and(|value| value >= 0.0) {
                return Err((
                    KISResponseSchemaStatus::BadVolumeField,
                    BTreeSet::new(),
                    unexpected,
                    ReasonCode::NegativeVolumeDetected,
                ));
            }
        }
        unexpected.extend(
            row_object
                .keys()
                .filter(|key| !required.contains(&key.as_str()))
                .map(|key| format!("row:{key}")),
        );
    }
    Ok(unexpected)
}

fn as_f64_string_or_number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
}

fn domestic_required_fields() -> Vec<&'static str> {
    vec![
        "stck_bsop_date",
        "stck_oprc",
        "stck_hgpr",
        "stck_lwpr",
        "stck_clpr",
        "acml_vol",
        "acml_tr_pbmn",
        "bidp1",
        "askp1",
    ]
}

fn overseas_required_fields() -> Vec<&'static str> {
    vec![
        "xymd", "open", "high", "low", "clos", "tvol", "tamt", "bid", "ask",
    ]
}

fn choose_status(
    current: KISResponseSchemaStatus,
    candidate: KISResponseSchemaStatus,
) -> KISResponseSchemaStatus {
    use KISResponseSchemaStatus as Status;
    match (current, candidate) {
        (Status::EndpointDenied, _) | (_, Status::EndpointDenied) => Status::EndpointDenied,
        (Status::UnsupportedSchema, _) | (_, Status::UnsupportedSchema) => {
            Status::UnsupportedSchema
        }
        (Status::MissingRequiredField, _) | (_, Status::MissingRequiredField) => {
            Status::MissingRequiredField
        }
        (Status::BadDateField, _) | (_, Status::BadDateField) => Status::BadDateField,
        (Status::BadPriceField, _) | (_, Status::BadPriceField) => Status::BadPriceField,
        (Status::BadVolumeField, _) | (_, Status::BadVolumeField) => Status::BadVolumeField,
        (Status::EmptyResponse, _) | (_, Status::EmptyResponse) => Status::EmptyResponse,
        _ => candidate,
    }
}
