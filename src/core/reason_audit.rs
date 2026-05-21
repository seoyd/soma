use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasonCodeCompletenessStatus {
    Complete,
    UnknownCodesFound,
    MissingCodesSuspected,
    Incomplete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasonCodeAudit {
    pub known_reason_codes: Vec<ReasonCode>,
    pub used_reason_codes: Vec<ReasonCode>,
    pub unknown_reason_codes: Vec<String>,
    #[serde(default)]
    pub unused_reason_codes: Option<Vec<ReasonCode>>,
    #[serde(default)]
    pub missing_reason_code_sites: Option<Vec<String>>,
    pub completeness_status: ReasonCodeCompletenessStatus,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn critical_reason_codes() -> Vec<ReasonCode> {
    stable_reason_codes(&[
        ReasonCode::BudgetExceeded,
        ReasonCode::DataQualityTooLow,
        ReasonCode::InvalidPrediction,
        ReasonCode::LiveModeDisabled,
        ReasonCode::MissingAuth,
        ReasonCode::MissingFile,
        ReasonCode::NoTradeDefault,
        ReasonCode::PreflightFailed,
        ReasonCode::RemotePathRejected,
        ReasonCode::RiskDenied,
        ReasonCode::SchemaMismatch,
    ])
}

pub fn audit_reason_codes(
    used_reason_codes: &[ReasonCode],
    unknown_reason_codes: &[String],
    missing_reason_code_sites: Option<Vec<String>>,
) -> ReasonCodeAudit {
    let known_reason_codes = stable_reason_codes(
        &critical_reason_codes()
            .into_iter()
            .chain(used_reason_codes.iter().cloned())
            .collect::<Vec<_>>(),
    );
    let used_reason_codes = stable_reason_codes(used_reason_codes);
    let unused_reason_codes = Some(
        known_reason_codes
            .iter()
            .filter(|code| !used_reason_codes.contains(code))
            .cloned()
            .collect::<Vec<_>>(),
    );
    let completeness_status = if !unknown_reason_codes.is_empty() {
        ReasonCodeCompletenessStatus::UnknownCodesFound
    } else if missing_reason_code_sites
        .as_ref()
        .is_some_and(|items| !items.is_empty())
    {
        ReasonCodeCompletenessStatus::MissingCodesSuspected
    } else if known_reason_codes.is_empty() {
        ReasonCodeCompletenessStatus::Incomplete
    } else {
        ReasonCodeCompletenessStatus::Complete
    };
    let reason_codes = match completeness_status {
        ReasonCodeCompletenessStatus::Complete => vec![ReasonCode::ReasonCodeAuditBuilt],
        _ => vec![
            ReasonCode::ReasonCodeAuditBuilt,
            ReasonCode::ReasonCodeAuditIncomplete,
        ],
    };
    ReasonCodeAudit {
        known_reason_codes,
        used_reason_codes,
        unknown_reason_codes: stable_ordered_unknown_codes(unknown_reason_codes),
        unused_reason_codes,
        missing_reason_code_sites,
        completeness_status,
        reason_codes,
    }
}

impl ReasonCodeAudit {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&format!(
            "{:?}|{}|{}",
            self.completeness_status,
            self.known_reason_codes
                .iter()
                .map(|code| format!("{code:?}"))
                .collect::<Vec<_>>()
                .join("|"),
            self.unknown_reason_codes.join("|")
        ))
    }

    pub fn to_text(&self) -> String {
        [
            format!("status={:?}", self.completeness_status),
            format!(
                "known={}",
                self.known_reason_codes
                    .iter()
                    .map(|code| format!("{code:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!(
                "used={}",
                self.used_reason_codes
                    .iter()
                    .map(|code| format!("{code:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!("unknown={}", self.unknown_reason_codes.join("|")),
            format!("fingerprint={}", self.fingerprint()),
        ]
        .join("\n")
    }
}

fn stable_ordered_unknown_codes(values: &[String]) -> Vec<String> {
    let mut ordered = values.to_vec();
    ordered.sort();
    ordered.dedup();
    ordered
}
