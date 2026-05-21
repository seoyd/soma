use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{DataFreshnessTier, EvidenceSourceKind, ProviderKind, ProviderMarket};

use super::StrategyUseCase;
use super::evidence_lane::{EvidenceLaneRunReport, EvidenceLaneStatus};
use super::executable_evidence_plan::ExecutableEvidencePlan;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReadinessCellStatus {
    NotAvailable,
    MissingAuth,
    MissingApproval,
    MissingEntitlement,
    ResearchOnly,
    ReadyForCollection,
    ReadyForBenchmark,
    Evaluated,
    InsufficientOutcomes,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessCell {
    pub market: ProviderMarket,
    pub use_case: StrategyUseCase,
    #[serde(default)]
    pub provider_kind: Option<ProviderKind>,
    pub source_kind: EvidenceSourceKind,
    pub freshness_tier: DataFreshnessTier,
    pub status: ReadinessCellStatus,
    pub official_readiness_eligible: bool,
    pub benchmark_eligible: bool,
    pub outcome_count: usize,
    #[serde(default)]
    pub calibration_status: Option<String>,
    #[serde(default)]
    pub risk_status: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceReadinessMatrix {
    pub cells: Vec<ReadinessCell>,
    pub crypto_summary: String,
    pub korean_equity_summary: String,
    pub us_equity_summary: String,
    pub research_supplemental_summary: String,
    pub official_ready_count: usize,
    pub benchmark_ready_count: usize,
    pub evaluated_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

impl EvidenceReadinessMatrix {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("official_ready_count={}", self.official_ready_count),
            format!("benchmark_ready_count={}", self.benchmark_ready_count),
            format!("evaluated_count={}", self.evaluated_count),
            format!("crypto_summary={}", self.crypto_summary),
            format!("korean_equity_summary={}", self.korean_equity_summary),
            format!("us_equity_summary={}", self.us_equity_summary),
            format!(
                "research_supplemental_summary={}",
                self.research_supplemental_summary
            ),
        ];
        for cell in &self.cells {
            lines.push(format!(
                "cell={:?}/{:?}/{:?};status={:?};official={};benchmark={};outcomes={}",
                cell.market,
                cell.use_case,
                cell.provider_kind,
                cell.status,
                cell.official_readiness_eligible,
                cell.benchmark_eligible,
                cell.outcome_count
            ));
        }
        lines.join("\n")
    }
}

pub fn build_evidence_readiness_matrix(
    plan: &ExecutableEvidencePlan,
    lane_reports: &[EvidenceLaneRunReport],
) -> EvidenceReadinessMatrix {
    let mut cells = plan
        .lanes
        .iter()
        .map(|lane| {
            let report = lane_reports
                .iter()
                .find(|report| report.lane_id == lane.lane_id);
            let status = classify_cell_status(lane.lane_status, report);
            ReadinessCell {
                market: lane.market,
                use_case: lane.desired_use_case,
                provider_kind: lane.provider_kind,
                source_kind: lane.source_kind,
                freshness_tier: lane.freshness_tier,
                status,
                official_readiness_eligible: lane.official_readiness_eligible(),
                benchmark_eligible: lane.benchmark_eligible(),
                outcome_count: report
                    .map(|value| value.outcome_records)
                    .unwrap_or_default(),
                calibration_status: report.and_then(|value| value.calibration_summary.clone()),
                risk_status: report.and_then(|value| value.risk_summary.clone()),
                reason_codes: vec![ReasonCode::EvidenceReadinessMatrixBuilt],
            }
        })
        .collect::<Vec<_>>();
    cells.sort_by(|left, right| {
        (left.market, left.use_case, left.provider_kind).cmp(&(
            right.market,
            right.use_case,
            right.provider_kind,
        ))
    });
    let official_ready_count = cells
        .iter()
        .filter(|cell| {
            cell.official_readiness_eligible
                && matches!(
                    cell.status,
                    ReadinessCellStatus::ReadyForCollection
                        | ReadinessCellStatus::ReadyForBenchmark
                        | ReadinessCellStatus::Evaluated
                )
        })
        .count();
    let benchmark_ready_count = cells
        .iter()
        .filter(|cell| {
            matches!(
                cell.status,
                ReadinessCellStatus::ReadyForBenchmark | ReadinessCellStatus::Evaluated
            )
        })
        .count();
    let evaluated_count = cells
        .iter()
        .filter(|cell| cell.status == ReadinessCellStatus::Evaluated)
        .count();
    EvidenceReadinessMatrix {
        crypto_summary: summary_for_market(&cells, ProviderMarket::Crypto),
        korean_equity_summary: summary_for_market(&cells, ProviderMarket::KoreanEquity),
        us_equity_summary: summary_for_market(&cells, ProviderMarket::USEquity),
        research_supplemental_summary: format!(
            "research_only_cells={}",
            cells
                .iter()
                .filter(|cell| cell.status == ReadinessCellStatus::ResearchOnly)
                .count()
        ),
        cells,
        official_ready_count,
        benchmark_ready_count,
        evaluated_count,
        reason_codes: vec![ReasonCode::EvidenceReadinessMatrixBuilt],
    }
}

fn classify_cell_status(
    lane_status: EvidenceLaneStatus,
    report: Option<&EvidenceLaneRunReport>,
) -> ReadinessCellStatus {
    match lane_status {
        EvidenceLaneStatus::SkippedMissingAuth => ReadinessCellStatus::MissingAuth,
        EvidenceLaneStatus::SkippedMissingApproval => ReadinessCellStatus::MissingApproval,
        EvidenceLaneStatus::SkippedMissingEndpointTemplate
        | EvidenceLaneStatus::SkippedMissingEntitlement
        | EvidenceLaneStatus::SkippedIncompatibleFreshness
        | EvidenceLaneStatus::SkippedBudgetExceeded
        | EvidenceLaneStatus::SkippedCoreBlocked => ReadinessCellStatus::MissingEntitlement,
        EvidenceLaneStatus::SkippedResearchOnlyNotOfficial => ReadinessCellStatus::ResearchOnly,
        EvidenceLaneStatus::FailedCollection
        | EvidenceLaneStatus::FailedPreflight
        | EvidenceLaneStatus::FailedBenchmark => ReadinessCellStatus::Failed,
        EvidenceLaneStatus::DiagnosticOnly => ReadinessCellStatus::NotAvailable,
        EvidenceLaneStatus::ReadyToRun => match report {
            Some(value) if value.yfinance_report.is_some() => ReadinessCellStatus::ResearchOnly,
            Some(value) if value.lane_status == EvidenceLaneStatus::RanSuccessfully => {
                if value.benchmark_report.is_some() && value.outcome_records > 0 {
                    ReadinessCellStatus::Evaluated
                } else {
                    ReadinessCellStatus::ReadyForCollection
                }
            }
            Some(value) if value.preflight_report.is_some() => {
                ReadinessCellStatus::ReadyForBenchmark
            }
            _ => ReadinessCellStatus::ReadyForCollection,
        },
        EvidenceLaneStatus::RanSuccessfully => ReadinessCellStatus::Evaluated,
    }
}

fn summary_for_market(cells: &[ReadinessCell], market: ProviderMarket) -> String {
    let relevant = cells
        .iter()
        .filter(|cell| cell.market == market)
        .collect::<Vec<_>>();
    format!(
        "cells={};evaluated={};blocked={}",
        relevant.len(),
        relevant
            .iter()
            .filter(|cell| cell.status == ReadinessCellStatus::Evaluated)
            .count(),
        relevant
            .iter()
            .filter(|cell| {
                matches!(
                    cell.status,
                    ReadinessCellStatus::MissingAuth
                        | ReadinessCellStatus::MissingApproval
                        | ReadinessCellStatus::MissingEntitlement
                )
            })
            .count()
    )
}
