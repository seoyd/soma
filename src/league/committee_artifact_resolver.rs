use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::ProviderMarket;

use super::committee_scenario_loader::CommitteeScenarioSourceKind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteeArtifactKind {
    EvidenceLaneReport,
    ProviderRealityEvidenceReport,
    ReadinessMatrix,
    CoreCheckedBenchmarkReport,
    OfficialBenchmarkReport,
    SourceAwareBenchmarkReport,
    YahooResearchEvidenceReport,
    CommitteeV1Bundle,
    CanonicalOhlcvCsv,
    PreflightReport,
    FixtureScenario,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeArtifactDescriptor {
    pub path: String,
    pub artifact_kind: CommitteeArtifactKind,
    #[serde(default)]
    pub source_kind: Option<CommitteeScenarioSourceKind>,
    #[serde(default)]
    pub market: Option<ProviderMarket>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub timeframe: Option<String>,
    pub row_level_available: bool,
    pub summary_available: bool,
    pub provenance_available: bool,
    pub preflight_available: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeArtifactResolver;

impl CommitteeArtifactResolver {
    pub fn resolve(&self, path: &str) -> CommitteeArtifactDescriptor {
        let lowered = path.to_ascii_lowercase();
        let path_ref = Path::new(path);
        let bytes = fs::read(path).ok();
        let text = bytes
            .as_deref()
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .unwrap_or("");
        let artifact_kind = if lowered.ends_with(".csv") {
            CommitteeArtifactKind::CanonicalOhlcvCsv
        } else if lowered.contains("preflight") {
            CommitteeArtifactKind::PreflightReport
        } else if lowered.contains("committee_v1_bundle") || lowered.contains("committee-v1-bundle")
        {
            CommitteeArtifactKind::CommitteeV1Bundle
        } else if lowered.contains("yfinance") {
            CommitteeArtifactKind::YahooResearchEvidenceReport
        } else if lowered.contains("source_benchmark") || lowered.contains("source-benchmark") {
            CommitteeArtifactKind::SourceAwareBenchmarkReport
        } else if lowered.contains("core_benchmark") || lowered.contains("core-benchmark") {
            CommitteeArtifactKind::CoreCheckedBenchmarkReport
        } else if lowered.contains("official") && lowered.contains("benchmark") {
            CommitteeArtifactKind::OfficialBenchmarkReport
        } else if lowered.contains("readiness_matrix") || lowered.contains("readiness-matrix") {
            CommitteeArtifactKind::ReadinessMatrix
        } else if lowered.contains("provider_reality") || lowered.contains("provider-reality") {
            CommitteeArtifactKind::ProviderRealityEvidenceReport
        } else if lowered.contains("evidence_lane")
            || lowered.contains("evidence-lane")
            || text.contains("\"lane_reports\"")
        {
            CommitteeArtifactKind::EvidenceLaneReport
        } else if lowered.contains("fixture") {
            CommitteeArtifactKind::FixtureScenario
        } else if text.contains("\"yfinance_symbols\"") {
            CommitteeArtifactKind::YahooResearchEvidenceReport
        } else if text.contains("\"dataset_inventory\"") {
            CommitteeArtifactKind::SourceAwareBenchmarkReport
        } else if text.contains("\"readiness_matrix\"") {
            CommitteeArtifactKind::ReadinessMatrix
        } else if text.contains("\"final_status\"") && text.contains("CommitteeV1") {
            CommitteeArtifactKind::CommitteeV1Bundle
        } else {
            CommitteeArtifactKind::Unknown
        };
        let source_kind = match artifact_kind {
            CommitteeArtifactKind::EvidenceLaneReport => {
                Some(CommitteeScenarioSourceKind::EvidenceLaneReport)
            }
            CommitteeArtifactKind::CoreCheckedBenchmarkReport => {
                Some(CommitteeScenarioSourceKind::CoreCheckedBenchmarkReport)
            }
            CommitteeArtifactKind::OfficialBenchmarkReport => {
                Some(CommitteeScenarioSourceKind::OfficialBenchmarkReport)
            }
            CommitteeArtifactKind::SourceAwareBenchmarkReport => {
                Some(CommitteeScenarioSourceKind::SourceAwareBenchmarkReport)
            }
            CommitteeArtifactKind::YahooResearchEvidenceReport => {
                Some(CommitteeScenarioSourceKind::YahooResearchEvidenceReport)
            }
            CommitteeArtifactKind::FixtureScenario => Some(CommitteeScenarioSourceKind::Fixture),
            _ => None,
        };
        let market = if lowered.contains("btc") || lowered.contains("krw") {
            Some(ProviderMarket::Crypto)
        } else if lowered.contains("kospi") || lowered.contains("krx") {
            Some(ProviderMarket::KoreanEquity)
        } else if lowered.contains("aapl") || lowered.contains("us") || text.contains("AAPL") {
            Some(ProviderMarket::USEquity)
        } else {
            None
        };
        let symbol = path_ref
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(|stem| stem.to_string());
        let timeframe = if text.contains("OneMinute") || lowered.contains("1m") {
            Some("1m".to_string())
        } else if text.contains("Daily") || lowered.contains("1d") {
            Some("1d".to_string())
        } else {
            None
        };
        let row_level_available = matches!(artifact_kind, CommitteeArtifactKind::CanonicalOhlcvCsv)
            || text.contains("\"rows\"")
            || text.contains("\"lane_reports\"")
            || text.contains("\"records\"")
            || lowered.contains("scenario_set");
        let summary_available = !text.is_empty() || lowered.ends_with(".toml");
        let provenance_available = text.contains("provenance")
            || text.contains("official")
            || matches!(
                artifact_kind,
                CommitteeArtifactKind::CoreCheckedBenchmarkReport
                    | CommitteeArtifactKind::OfficialBenchmarkReport
            );
        let preflight_available = text.contains("preflight")
            || path_ref
                .parent()
                .is_some_and(|parent| parent.join("preflight_report.json").exists());
        let reason_codes = if artifact_kind == CommitteeArtifactKind::Unknown {
            vec![
                ReasonCode::CommitteeArtifactResolverBuilt,
                ReasonCode::CommitteeArtifactUnknown,
            ]
        } else {
            vec![ReasonCode::CommitteeArtifactResolverBuilt]
        };
        CommitteeArtifactDescriptor {
            path: path.to_string(),
            artifact_kind,
            source_kind,
            market,
            symbol,
            timeframe,
            row_level_available,
            summary_available,
            provenance_available,
            preflight_available,
            reason_codes,
        }
    }
}
