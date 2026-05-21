use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXRawResponseArchiveSource {
    FixtureReplay,
    LiveProvider,
    LocalImport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KRXRawResponseArchiveRecord {
    pub record_id: String,
    pub provider_symbol: String,
    pub normalized_symbol: String,
    pub timeframe: String,
    pub request_metadata_redacted: String,
    pub response_path: String,
    pub response_bytes: usize,
    #[serde(default)]
    pub collected_at: Option<String>,
    pub source: KRXRawResponseArchiveSource,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KRXRawResponseArchiveSummary {
    pub archive_id: String,
    pub records: Vec<KRXRawResponseArchiveRecord>,
    pub total_bytes: usize,
    pub reason_codes: Vec<ReasonCode>,
}

impl KRXRawResponseArchiveRecord {
    pub fn from_fixture(
        output_root: &Path,
        provider_symbol: &str,
        normalized_symbol: &str,
        response_path: &Path,
    ) -> Result<Self, String> {
        fs::create_dir_all(output_root).map_err(|err| err.to_string())?;
        let destination = output_root.join(format!(
            "{}_raw_response.json",
            normalized_symbol.to_ascii_lowercase()
        ));
        fs::copy(response_path, &destination).map_err(|err| err.to_string())?;
        let response_bytes = fs::metadata(&destination)
            .map(|metadata| metadata.len() as usize)
            .unwrap_or(0);
        Ok(Self {
            record_id: format!("{}-fixture-archive", normalized_symbol.to_ascii_lowercase()),
            provider_symbol: provider_symbol.to_string(),
            normalized_symbol: normalized_symbol.to_string(),
            timeframe: "1d".to_string(),
            request_metadata_redacted:
                "source=fixture-replay;endpoint=redacted;auth=redacted;query=redacted".to_string(),
            response_path: destination.display().to_string(),
            response_bytes,
            collected_at: None,
            source: KRXRawResponseArchiveSource::FixtureReplay,
            reason_codes: stable_reason_codes(&[
                ReasonCode::ProviderResponseArchived,
                ReasonCode::MockFixtureLoaded,
            ]),
        })
    }

    pub fn to_text(&self) -> String {
        format!(
            "record_id={};provider_symbol={};normalized_symbol={};timeframe={};request_metadata_redacted={};response_path={};response_bytes={};collected_at={};source={:?};reason_codes={}",
            self.record_id,
            self.provider_symbol,
            self.normalized_symbol,
            self.timeframe,
            self.request_metadata_redacted,
            self.response_path,
            self.response_bytes,
            self.collected_at.clone().unwrap_or_default(),
            self.source,
            self.reason_codes
                .iter()
                .map(|reason| format!("{reason:?}"))
                .collect::<Vec<_>>()
                .join("|")
        )
    }
}

impl KRXRawResponseArchiveSummary {
    pub fn new(archive_id: impl Into<String>, records: Vec<KRXRawResponseArchiveRecord>) -> Self {
        let total_bytes = records.iter().map(|record| record.response_bytes).sum();
        let reason_codes = stable_reason_codes(
            &records
                .iter()
                .flat_map(|record| record.reason_codes.clone())
                .chain([ReasonCode::ProviderResponseArchived])
                .collect::<Vec<_>>(),
        );
        Self {
            archive_id: archive_id.into(),
            records,
            total_bytes,
            reason_codes,
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("archive_id={}", self.archive_id),
            format!("total_bytes={}", self.total_bytes),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ];
        lines.extend(
            self.records
                .iter()
                .map(KRXRawResponseArchiveRecord::to_text),
        );
        lines.join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("krx_raw_archive_summary.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_raw_archive_summary.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

pub(crate) fn raw_archive_dir(output_root: &Path) -> PathBuf {
    output_root.join("raw_archive")
}
