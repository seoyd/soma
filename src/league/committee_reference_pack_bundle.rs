use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::committee_reference_pack::GeneratedCommitteeReferencePack;
use super::reference_pack_quality::{ReferencePackQualityReport, ReferencePackQualityStatus};
use super::sufficiency_closure::{
    SufficiencyClosureFinalRecommendation, SufficiencyClosureReport, SufficiencyClosureStatus,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteeReferencePackFinalStatus {
    ReferencePackBuilt,
    ReferencePackDiagnosticOnly,
    NeedMoreCandleData,
    NeedBetterTimestampAlignment,
    NeedMoreScenarios,
    NeedMoreOfficialRows,
    NeedMoreReferences,
    BlockedNoLookahead,
    BlockedBadDataQuality,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteeReferencePackRecommendation {
    OutcomeLinksImproved,
    CounterfactualDepthImproved,
    NeedMoreCandleData,
    NeedBetterTimestampAlignment,
    NeedMoreOfficialRows,
    ImproveScenarioMaterializationFirst,
    ImproveBaselineReferenceDepth,
    ImproveRiskGovernorFirst,
    CommitteeBenchmarkReadyForControlledEvidence,
    MoreOfficialCommitteeEvidence,
    KeepTrinity,
    NeedMoreEvidence,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeReferencePackBundle {
    pub reference_pack: GeneratedCommitteeReferencePack,
    pub quality_report: ReferencePackQualityReport,
    #[serde(default)]
    pub sufficiency_closure_report: Option<SufficiencyClosureReport>,
    pub storage_summary: String,
    pub final_status: CommitteeReferencePackFinalStatus,
    pub final_recommendation: CommitteeReferencePackRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CommitteeReferencePackBundle {
    pub fn new(
        reference_pack: GeneratedCommitteeReferencePack,
        quality_report: ReferencePackQualityReport,
        sufficiency_closure_report: Option<SufficiencyClosureReport>,
        storage_summary: String,
    ) -> Self {
        let (final_status, final_recommendation) = determine_final_status(
            &reference_pack,
            &quality_report,
            sufficiency_closure_report.as_ref(),
        );
        Self {
            reference_pack,
            quality_report,
            sufficiency_closure_report,
            storage_summary,
            final_status,
            final_recommendation,
            reason_codes: stable_reason_codes(&[
                ReasonCode::CommitteeReferencePackBundleBuilt,
                ReasonCode::CommitteeReferencePackBuilt,
            ]),
        }
    }

    pub fn to_text(&self) -> String {
        [
            format!("storage_summary={}", self.storage_summary),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            self.reference_pack.to_text(),
            self.quality_report.to_text(),
            self.sufficiency_closure_report
                .as_ref()
                .map(SufficiencyClosureReport::to_text)
                .unwrap_or_else(|| "sufficiency_closure=none".to_string()),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        self.reference_pack.write_to_dir(output_dir)?;
        fs::write(
            output_dir.join("reference_pack_quality.txt"),
            self.quality_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("sufficiency_closure.txt"),
            self.sufficiency_closure_report
                .as_ref()
                .map(SufficiencyClosureReport::to_text)
                .unwrap_or_else(|| "sufficiency_closure=none".to_string()),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("committee_reference_pack_summary.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("committee_reference_pack_bundle.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn determine_final_status(
    reference_pack: &GeneratedCommitteeReferencePack,
    quality_report: &ReferencePackQualityReport,
    sufficiency_closure_report: Option<&SufficiencyClosureReport>,
) -> (
    CommitteeReferencePackFinalStatus,
    CommitteeReferencePackRecommendation,
) {
    if reference_pack.scenario_count == 0 {
        return (
            CommitteeReferencePackFinalStatus::NeedMoreScenarios,
            CommitteeReferencePackRecommendation::ImproveScenarioMaterializationFirst,
        );
    }
    match reference_pack.alignment_report.alignment_status {
        super::candle_alignment::CandleAlignmentOverallStatus::NeedMoreCandleData
        | super::candle_alignment::CandleAlignmentOverallStatus::NeedLongerFutureWindows => {
            return (
                CommitteeReferencePackFinalStatus::NeedMoreCandleData,
                CommitteeReferencePackRecommendation::NeedMoreCandleData,
            );
        }
        super::candle_alignment::CandleAlignmentOverallStatus::NeedBetterTimestampAlignment
        | super::candle_alignment::CandleAlignmentOverallStatus::DiagnosticOnly => {
            return (
                CommitteeReferencePackFinalStatus::NeedBetterTimestampAlignment,
                CommitteeReferencePackRecommendation::NeedBetterTimestampAlignment,
            );
        }
        super::candle_alignment::CandleAlignmentOverallStatus::BadDataQuality => {
            return (
                CommitteeReferencePackFinalStatus::BlockedBadDataQuality,
                CommitteeReferencePackRecommendation::NeedMoreEvidence,
            );
        }
        super::candle_alignment::CandleAlignmentOverallStatus::HealthyAlignment
        | super::candle_alignment::CandleAlignmentOverallStatus::Unknown => {}
    }
    if reference_pack.rejected_count > 0 && reference_pack.no_lookahead_safe_count() == 0 {
        return (
            CommitteeReferencePackFinalStatus::BlockedNoLookahead,
            CommitteeReferencePackRecommendation::NeedMoreEvidence,
        );
    }
    if matches!(
        quality_report.quality_status,
        ReferencePackQualityStatus::NeedMoreOutcomeReferences
            | ReferencePackQualityStatus::NeedMoreBaselineReferences
            | ReferencePackQualityStatus::NeedMoreNoTradeCounterfactuals
            | ReferencePackQualityStatus::NeedMoreRiskDeniedCounterfactuals
            | ReferencePackQualityStatus::InsufficientReferenceQuality
    ) {
        return (
            CommitteeReferencePackFinalStatus::NeedMoreReferences,
            match quality_report.quality_status {
                ReferencePackQualityStatus::NeedMoreBaselineReferences => {
                    CommitteeReferencePackRecommendation::ImproveBaselineReferenceDepth
                }
                ReferencePackQualityStatus::NeedMoreNoTradeCounterfactuals
                | ReferencePackQualityStatus::NeedMoreRiskDeniedCounterfactuals => {
                    CommitteeReferencePackRecommendation::CounterfactualDepthImproved
                }
                _ => CommitteeReferencePackRecommendation::OutcomeLinksImproved,
            },
        );
    }
    if quality_report.official_ready_reference_count == 0 {
        if let Some(closure) = sufficiency_closure_report {
            if closure.closure_status
                == SufficiencyClosureStatus::SufficiencyGatePassedForControlledEvidence
            {
                return (
                    CommitteeReferencePackFinalStatus::ReferencePackDiagnosticOnly,
                    CommitteeReferencePackRecommendation::CommitteeBenchmarkReadyForControlledEvidence,
                );
            }
        }
        return (
            CommitteeReferencePackFinalStatus::NeedMoreOfficialRows,
            CommitteeReferencePackRecommendation::MoreOfficialCommitteeEvidence,
        );
    }
    if let Some(closure) = sufficiency_closure_report {
        if closure.current_status
            == super::committee_evidence_sufficiency::CommitteeEvidenceSufficiencyStatus::SufficientForCryptoOnlyBenchmark
        {
            return (
                CommitteeReferencePackFinalStatus::ReferencePackDiagnosticOnly,
                CommitteeReferencePackRecommendation::KeepTrinity,
            );
        }
    }
    if reference_pack.diagnostic_only_count == reference_pack.generated_references.len() {
        return (
            CommitteeReferencePackFinalStatus::ReferencePackDiagnosticOnly,
            CommitteeReferencePackRecommendation::KeepTrinity,
        );
    }
    (
        CommitteeReferencePackFinalStatus::ReferencePackBuilt,
        match sufficiency_closure_report.map(|report| report.final_recommendation) {
            Some(SufficiencyClosureFinalRecommendation::CommitteeV1BenchmarkReady) => {
                CommitteeReferencePackRecommendation::KeepTrinity
            }
            Some(
                SufficiencyClosureFinalRecommendation::CommitteeBenchmarkReadyForControlledEvidence,
            ) => CommitteeReferencePackRecommendation::CommitteeBenchmarkReadyForControlledEvidence,
            Some(SufficiencyClosureFinalRecommendation::ImproveBaselineReferenceDepth) => {
                CommitteeReferencePackRecommendation::ImproveBaselineReferenceDepth
            }
            Some(SufficiencyClosureFinalRecommendation::ImproveCounterfactualDepthFirst) => {
                CommitteeReferencePackRecommendation::CounterfactualDepthImproved
            }
            Some(SufficiencyClosureFinalRecommendation::ImproveOutcomeLinkingFirst) => {
                CommitteeReferencePackRecommendation::OutcomeLinksImproved
            }
            Some(SufficiencyClosureFinalRecommendation::MoreOfficialCommitteeEvidence) => {
                CommitteeReferencePackRecommendation::MoreOfficialCommitteeEvidence
            }
            Some(SufficiencyClosureFinalRecommendation::MoreCandleData) => {
                CommitteeReferencePackRecommendation::NeedMoreCandleData
            }
            Some(SufficiencyClosureFinalRecommendation::KeepTrinity) => {
                CommitteeReferencePackRecommendation::KeepTrinity
            }
            Some(SufficiencyClosureFinalRecommendation::NeedMoreEvidence)
            | Some(SufficiencyClosureFinalRecommendation::Unknown)
            | None => CommitteeReferencePackRecommendation::NeedMoreEvidence,
        },
    )
}
