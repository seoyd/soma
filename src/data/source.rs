use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum EvidenceSourceKind {
    #[serde(alias = "LocalCsv")]
    RealLocal,
    OfficialApiCollected,
    YFinanceResearch,
    #[serde(alias = "Synthetic")]
    SyntheticFixture,
    TestFixture,
    GeneratedSynthetic,
    ExternalPredictionOnly,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceUse {
    PipelineSmoke,
    BacktestResearch,
    RealDataEvidence,
    ReadinessEvidence,
    DisallowedForReadiness,
}

impl EvidenceSourceKind {
    pub fn supports(self, use_case: EvidenceUse) -> bool {
        match use_case {
            EvidenceUse::PipelineSmoke | EvidenceUse::BacktestResearch => true,
            EvidenceUse::RealDataEvidence => {
                matches!(self, Self::RealLocal | Self::OfficialApiCollected)
            }
            EvidenceUse::ReadinessEvidence => self.readiness_eligible(),
            EvidenceUse::DisallowedForReadiness => !self.readiness_eligible(),
        }
    }

    pub fn readiness_eligible(self) -> bool {
        matches!(self, Self::RealLocal | Self::OfficialApiCollected)
    }
}

pub type DataSourceKind = EvidenceSourceKind;

pub fn infer_source_kind_from_path(path: Option<&Path>) -> EvidenceSourceKind {
    path.and_then(|path| path.to_str())
        .map(|value| {
            let lower = value.to_ascii_lowercase();
            if lower.contains("generated") && lower.contains("synthetic") {
                EvidenceSourceKind::GeneratedSynthetic
            } else if lower.contains("yfinance") || lower.contains("yahoo") {
                EvidenceSourceKind::YFinanceResearch
            } else if lower.contains("synthetic") {
                EvidenceSourceKind::SyntheticFixture
            } else if lower.contains("tests/fixtures") || lower.contains("testdata") {
                EvidenceSourceKind::TestFixture
            } else if lower.is_empty() {
                EvidenceSourceKind::Unknown
            } else {
                EvidenceSourceKind::Unknown
            }
        })
        .unwrap_or(EvidenceSourceKind::Unknown)
}
