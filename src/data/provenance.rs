use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::{EvidenceSourceKind, infer_source_kind_from_path};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataProvenance {
    pub source_kind: EvidenceSourceKind,
    pub source_label: String,
    #[serde(default)]
    pub provider_label: Option<String>,
    #[serde(default)]
    pub upstream_label: Option<String>,
    #[serde(default)]
    pub local_path: Option<String>,
    #[serde(default)]
    pub generated_by: Option<String>,
    #[serde(default)]
    pub user_supplied: bool,
    #[serde(default)]
    pub downloaded_by_soma: bool,
    #[serde(default)]
    pub remote_url_present: bool,
    #[serde(default)]
    pub official_provider: Option<bool>,
    #[serde(default)]
    pub affiliated_or_endorsed: Option<bool>,
    #[serde(default)]
    pub intended_use: Option<String>,
    #[serde(default)]
    pub readiness_eligible: Option<bool>,
    #[serde(default)]
    pub benchmark_eligible: Option<bool>,
    #[serde(default)]
    pub license_note: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl DataProvenance {
    pub fn inferred_from_path(path: Option<&str>) -> Self {
        let local_path = path.map(|value| value.to_string());
        let remote = local_path
            .as_deref()
            .is_some_and(|value| value.contains("://"));
        let source_kind = local_path
            .as_deref()
            .map(std::path::Path::new)
            .map(|path| infer_source_kind_from_path(Some(path)))
            .unwrap_or(EvidenceSourceKind::Unknown);
        Self {
            source_kind,
            source_label: "inferred-local-path".to_string(),
            provider_label: None,
            upstream_label: None,
            local_path,
            generated_by: None,
            user_supplied: false,
            downloaded_by_soma: false,
            remote_url_present: remote,
            official_provider: None,
            affiliated_or_endorsed: None,
            intended_use: None,
            readiness_eligible: None,
            benchmark_eligible: None,
            license_note: None,
            notes: None,
            reason_codes: if remote {
                vec![ReasonCode::LocalPathRejected]
            } else {
                vec![ReasonCode::DeterministicPath]
            },
        }
    }

    pub fn validate_local_only(&self) -> Vec<ReasonCode> {
        let mut reason_codes = self.reason_codes.clone();
        let remote = self.remote_url_present
            || self
                .local_path
                .as_deref()
                .is_some_and(|path| path.contains("://"));
        if remote {
            reason_codes.push(ReasonCode::LocalPathRejected);
        }
        if self.source_kind == EvidenceSourceKind::Unknown {
            reason_codes.push(ReasonCode::UnknownDataProvenance);
        }
        if self.downloaded_by_soma && self.source_kind != EvidenceSourceKind::OfficialApiCollected {
            reason_codes.push(ReasonCode::DoctrineViolation);
        }
        dedupe_reasons(reason_codes)
    }

    pub fn to_deterministic_string(&self) -> String {
        [
            format!("source_kind={:?}", self.source_kind),
            format!("source_label={}", self.source_label),
            format!(
                "provider_label={}",
                self.provider_label.clone().unwrap_or_default()
            ),
            format!(
                "upstream_label={}",
                self.upstream_label.clone().unwrap_or_default()
            ),
            format!("local_path={}", self.local_path.clone().unwrap_or_default()),
            format!(
                "generated_by={}",
                self.generated_by.clone().unwrap_or_default()
            ),
            format!("user_supplied={}", self.user_supplied),
            format!("downloaded_by_soma={}", self.downloaded_by_soma),
            format!("remote_url_present={}", self.remote_url_present),
            format!(
                "official_provider={}",
                self.official_provider
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!(
                "affiliated_or_endorsed={}",
                self.affiliated_or_endorsed
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!(
                "intended_use={}",
                self.intended_use.clone().unwrap_or_default()
            ),
            format!(
                "readiness_eligible={}",
                self.readiness_eligible
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!(
                "benchmark_eligible={}",
                self.benchmark_eligible
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!(
                "license_note={}",
                self.license_note.clone().unwrap_or_default()
            ),
            format!("notes={}", self.notes.clone().unwrap_or_default()),
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
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}
