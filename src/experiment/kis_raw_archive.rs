use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::kis_endpoint_policy::KISEndpointCategory;
use super::kis_symbol_whitelist::KISMarket;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISRawResponseArchiveSource {
    FixtureReplay,
    LiveProvider,
    LocalImport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISRawResponseArchiveRecord {
    pub record_id: String,
    pub market: KISMarket,
    pub provider_symbol: String,
    pub normalized_symbol: String,
    pub timeframe: String,
    pub endpoint_category: KISEndpointCategory,
    pub request_metadata_redacted: String,
    pub response_path: String,
    pub response_bytes: usize,
    #[serde(default)]
    pub collected_at: Option<String>,
    pub source: KISRawResponseArchiveSource,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KISRawResponseArchiveSummary {
    pub archive_id: String,
    pub records: Vec<KISRawResponseArchiveRecord>,
    pub total_bytes: usize,
    pub reason_codes: Vec<ReasonCode>,
}

impl KISRawResponseArchiveRecord {
    pub fn from_fixture(
        output_root: &Path,
        market: KISMarket,
        provider_symbol: &str,
        normalized_symbol: &str,
        timeframe: &str,
        endpoint_category: KISEndpointCategory,
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
            market,
            provider_symbol: provider_symbol.to_string(),
            normalized_symbol: normalized_symbol.to_string(),
            timeframe: timeframe.to_string(),
            endpoint_category,
            request_metadata_redacted: redacted_request_metadata(
                endpoint_category,
                provider_symbol,
                normalized_symbol,
                None,
            ),
            response_path: destination.display().to_string(),
            response_bytes,
            collected_at: None,
            source: KISRawResponseArchiveSource::FixtureReplay,
            reason_codes: stable_reason_codes(&[
                ReasonCode::ProviderResponseArchived,
                ReasonCode::MockFixtureLoaded,
                ReasonCode::KISRawArchiveBuilt,
            ]),
        })
    }

    pub fn from_local_import(
        output_root: &Path,
        market: KISMarket,
        provider_symbol: &str,
        normalized_symbol: &str,
        timeframe: &str,
        endpoint_category: KISEndpointCategory,
        response_path: &Path,
    ) -> Result<Self, String> {
        fs::create_dir_all(output_root).map_err(|err| err.to_string())?;
        let destination = output_root.join(format!(
            "{}_local_import_payload.json",
            normalized_symbol.to_ascii_lowercase()
        ));
        let payload = serde_json::json!({
            "source": "local-import",
            "canonical_path": response_path.display().to_string(),
            "provider_symbol": provider_symbol,
            "normalized_symbol": normalized_symbol,
        });
        fs::write(
            &destination,
            serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        let response_bytes = fs::metadata(&destination)
            .map(|metadata| metadata.len() as usize)
            .unwrap_or(0);
        Ok(Self {
            record_id: format!(
                "{}-local-import-archive",
                normalized_symbol.to_ascii_lowercase()
            ),
            market,
            provider_symbol: provider_symbol.to_string(),
            normalized_symbol: normalized_symbol.to_string(),
            timeframe: timeframe.to_string(),
            endpoint_category,
            request_metadata_redacted: redacted_request_metadata(
                endpoint_category,
                provider_symbol,
                normalized_symbol,
                Some("local-import"),
            ),
            response_path: destination.display().to_string(),
            response_bytes,
            collected_at: None,
            source: KISRawResponseArchiveSource::LocalImport,
            reason_codes: stable_reason_codes(&[
                ReasonCode::ProviderResponseArchived,
                ReasonCode::LocalFileOnly,
                ReasonCode::KISRawArchiveBuilt,
            ]),
        })
    }

    pub fn to_text(&self) -> String {
        format!(
            "record_id={};market={:?};provider_symbol={};normalized_symbol={};timeframe={};endpoint_category={:?};request_metadata_redacted={};response_path={};response_bytes={};collected_at={};source={:?};reason_codes={}",
            self.record_id,
            self.market,
            self.provider_symbol,
            self.normalized_symbol,
            self.timeframe,
            self.endpoint_category,
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

impl KISRawResponseArchiveSummary {
    pub fn new(
        archive_id: impl Into<String>,
        mut records: Vec<KISRawResponseArchiveRecord>,
    ) -> Self {
        records.sort_by(|left, right| left.record_id.cmp(&right.record_id));
        let total_bytes = records.iter().map(|record| record.response_bytes).sum();
        let reason_codes = stable_reason_codes(
            &records
                .iter()
                .flat_map(|record| record.reason_codes.clone())
                .chain([
                    ReasonCode::ProviderResponseArchived,
                    ReasonCode::KISRawArchiveBuilt,
                ])
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
                .map(KISRawResponseArchiveRecord::to_text),
        );
        lines.join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("kis_raw_archive_summary.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_raw_archive_summary.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

pub(crate) fn raw_archive_dir(output_root: &Path) -> PathBuf {
    output_root.join("raw_archive")
}

pub fn redacted_request_metadata(
    endpoint_category: KISEndpointCategory,
    provider_symbol: &str,
    normalized_symbol: &str,
    source: Option<&str>,
) -> String {
    format!(
        "source={};endpoint_category={:?};provider_symbol={};normalized_symbol={};auth=redacted;headers=redacted;query=redacted;body=redacted",
        source.unwrap_or("fixture-replay"),
        endpoint_category,
        provider_symbol,
        normalized_symbol,
    )
}
