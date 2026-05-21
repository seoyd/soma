use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::candle_alignment::CandleAlignmentOverallStatus;
use super::committee_reference_pack::{
    CommitteeReferencePackConfig, GeneratedCommitteeReferencePack,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReferencePackQualityStatus {
    HealthyReferencePack,
    NeedMoreOutcomeReferences,
    NeedMoreBaselineReferences,
    NeedMoreNoTradeCounterfactuals,
    NeedMoreRiskDeniedCounterfactuals,
    NeedMoreCandleData,
    NeedBetterTimestampAlignment,
    TooManyDiagnosticOnlyReferences,
    ResearchOnlyReferences,
    FixtureOnlyReferences,
    InsufficientReferenceQuality,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferencePackQualityReport {
    pub reference_pack_id: String,
    pub scenario_count: usize,
    pub alignment_status: CandleAlignmentOverallStatus,
    pub outcome_reference_count: usize,
    pub baseline_reference_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denied_counterfactual_count: usize,
    pub official_ready_reference_count: usize,
    pub research_only_reference_count: usize,
    pub fixture_reference_count: usize,
    pub diagnostic_only_count: usize,
    pub rejected_count: usize,
    pub no_lookahead_safe_count: usize,
    pub quality_status: ReferencePackQualityStatus,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_reference_pack_quality_report(
    config: &CommitteeReferencePackConfig,
    pack: &GeneratedCommitteeReferencePack,
) -> ReferencePackQualityReport {
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let official_ready_reference_count = pack.official_ready_reference_count();
    let research_only_reference_count = pack.research_only_reference_count();
    let fixture_reference_count = pack.fixture_reference_count();
    let no_lookahead_safe_count = pack.no_lookahead_safe_count();
    let quality_status = if pack.scenario_count == 0 || pack.generated_outcome_count == 0 {
        blockers.push("outcome references remain missing".to_string());
        ReferencePackQualityStatus::NeedMoreOutcomeReferences
    } else if pack.generated_baseline_count == 0 {
        blockers.push("baseline references remain missing".to_string());
        ReferencePackQualityStatus::NeedMoreBaselineReferences
    } else if pack.generated_no_trade_count == 0 {
        blockers.push("no-trade counterfactuals remain missing".to_string());
        ReferencePackQualityStatus::NeedMoreNoTradeCounterfactuals
    } else if pack.generated_risk_denied_count == 0 {
        blockers.push("risk-denied counterfactuals remain missing".to_string());
        ReferencePackQualityStatus::NeedMoreRiskDeniedCounterfactuals
    } else if matches!(
        pack.alignment_report.alignment_status,
        CandleAlignmentOverallStatus::NeedMoreCandleData
            | CandleAlignmentOverallStatus::NeedLongerFutureWindows
    ) {
        blockers.push("local candle coverage remains incomplete".to_string());
        ReferencePackQualityStatus::NeedMoreCandleData
    } else if matches!(
        pack.alignment_report.alignment_status,
        CandleAlignmentOverallStatus::NeedBetterTimestampAlignment
    ) {
        blockers.push("timestamp alignment remains conservative".to_string());
        ReferencePackQualityStatus::NeedBetterTimestampAlignment
    } else if research_only_reference_count > 0
        && research_only_reference_count == pack.generated_references.len()
    {
        warnings.push("all generated references remain research-only".to_string());
        ReferencePackQualityStatus::ResearchOnlyReferences
    } else if fixture_reference_count > 0
        && fixture_reference_count == pack.generated_references.len()
        && !config.allow_controlled_fixture_references
    {
        warnings.push("all generated references remain fixture-only".to_string());
        ReferencePackQualityStatus::FixtureOnlyReferences
    } else if pack.diagnostic_only_count
        > pack
            .generated_references
            .len()
            .saturating_sub(pack.diagnostic_only_count)
    {
        warnings.push("diagnostic-only references dominate the pack".to_string());
        ReferencePackQualityStatus::TooManyDiagnosticOnlyReferences
    } else if pack.rejected_count > 0 && no_lookahead_safe_count == 0 {
        blockers.push("rejected references block conservative reuse".to_string());
        ReferencePackQualityStatus::InsufficientReferenceQuality
    } else {
        if research_only_reference_count > 0 {
            warnings.push("yfinance-backed references remain research-only".to_string());
        }
        if fixture_reference_count > 0 {
            if config.allow_controlled_fixture_references {
                warnings.push(
                    "controlled fixture references can improve controlled evidence but do not create official readiness"
                        .to_string(),
                );
            } else {
                warnings.push("fixture references stay fixture-only".to_string());
            }
        }
        if official_ready_reference_count == 0 {
            warnings.push(
                "official readiness still requires true OfficialApiCollected evidence".to_string(),
            );
        }
        ReferencePackQualityStatus::HealthyReferencePack
    };
    ReferencePackQualityReport {
        reference_pack_id: pack.reference_pack_id.clone(),
        scenario_count: pack.scenario_count,
        alignment_status: pack.alignment_report.alignment_status,
        outcome_reference_count: pack.generated_outcome_count,
        baseline_reference_count: pack.generated_baseline_count,
        no_trade_counterfactual_count: pack.generated_no_trade_count,
        risk_denied_counterfactual_count: pack.generated_risk_denied_count,
        official_ready_reference_count,
        research_only_reference_count,
        fixture_reference_count,
        diagnostic_only_count: pack.diagnostic_only_count,
        rejected_count: pack.rejected_count,
        no_lookahead_safe_count,
        quality_status,
        blockers,
        warnings,
        reason_codes: stable_reason_codes(&[
            ReasonCode::ReferencePackQualityBuilt,
            ReasonCode::CommitteeReferencePackBuilt,
        ]),
    }
}

impl ReferencePackQualityReport {
    pub fn to_text(&self) -> String {
        [
            format!("reference_pack_id={}", self.reference_pack_id),
            format!("scenario_count={}", self.scenario_count),
            format!("alignment_status={:?}", self.alignment_status),
            format!("outcome_reference_count={}", self.outcome_reference_count),
            format!("baseline_reference_count={}", self.baseline_reference_count),
            format!(
                "no_trade_counterfactual_count={}",
                self.no_trade_counterfactual_count
            ),
            format!(
                "risk_denied_counterfactual_count={}",
                self.risk_denied_counterfactual_count
            ),
            format!(
                "official_ready_reference_count={}",
                self.official_ready_reference_count
            ),
            format!(
                "research_only_reference_count={}",
                self.research_only_reference_count
            ),
            format!("fixture_reference_count={}", self.fixture_reference_count),
            format!("diagnostic_only_count={}", self.diagnostic_only_count),
            format!("rejected_count={}", self.rejected_count),
            format!("no_lookahead_safe_count={}", self.no_lookahead_safe_count),
            format!("quality_status={:?}", self.quality_status),
            format!("blockers={}", self.blockers.join("|")),
            format!("warnings={}", self.warnings.join("|")),
        ]
        .join("\n")
    }
}
