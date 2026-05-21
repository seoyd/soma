use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::EvidenceSourceKind;

use super::committee_replay::CommitteeReplayReport;
use super::committee_risk_bridge::CommitteeFinalAction;
use super::committee_scenario_loader::{CommitteeScenarioSet, CommitteeScenarioSourceKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeActionabilityStatus {
    ActionableResearch,
    MostlyNoTrade,
    MostlyRiskDenied,
    ResearchOnly,
    FixtureOnly,
    NotEnoughRows,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeActionabilityReport {
    pub decision_count: usize,
    pub actionable_count: usize,
    pub paper_approve_count: usize,
    pub paper_reduce_size_count: usize,
    pub human_confirm_required_count: usize,
    pub final_no_trade_count: usize,
    pub final_denied_count: usize,
    pub research_only_count: usize,
    pub fixture_only_count: usize,
    pub official_actionable_count: usize,
    pub actionability_ratio: f64,
    pub official_actionability_ratio: f64,
    pub risk_block_ratio: f64,
    pub confirm_ratio: f64,
    pub actionability_status: CommitteeActionabilityStatus,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_committee_actionability_report(
    scenario_set: &CommitteeScenarioSet,
    replay_report: &CommitteeReplayReport,
) -> CommitteeActionabilityReport {
    let decision_count = replay_report.record_count;
    let paper_approve_count = replay_report
        .records
        .iter()
        .filter(|record| record.final_action == CommitteeFinalAction::PaperApprove)
        .count();
    let paper_reduce_size_count = replay_report
        .records
        .iter()
        .filter(|record| record.final_action == CommitteeFinalAction::PaperReduceSize)
        .count();
    let human_confirm_required_count = replay_report
        .records
        .iter()
        .filter(|record| record.final_action == CommitteeFinalAction::HumanConfirmRequired)
        .count();
    let final_no_trade_count = replay_report
        .records
        .iter()
        .filter(|record| record.final_action == CommitteeFinalAction::FinalNoTrade)
        .count();
    let final_denied_count = replay_report
        .records
        .iter()
        .filter(|record| record.final_action == CommitteeFinalAction::FinalDenied)
        .count();
    let actionable_count =
        paper_approve_count + paper_reduce_size_count + human_confirm_required_count;
    let research_only_count = replay_report
        .records
        .iter()
        .filter(|record| {
            record.scenario_row.evidence_source_kind == EvidenceSourceKind::YFinanceResearch
        })
        .count();
    let fixture_only_count = replay_report
        .records
        .iter()
        .filter(|record| {
            matches!(
                record.scenario_row.source_kind,
                CommitteeScenarioSourceKind::Fixture | CommitteeScenarioSourceKind::SyntheticTest
            )
        })
        .count();
    let official_actionable_count = replay_report
        .records
        .iter()
        .filter(|record| {
            matches!(
                record.final_action,
                CommitteeFinalAction::PaperApprove
                    | CommitteeFinalAction::PaperReduceSize
                    | CommitteeFinalAction::HumanConfirmRequired
            ) && record
                .scenario_row
                .evidence_source_kind
                .readiness_eligible()
                && !matches!(
                    record.scenario_row.source_kind,
                    CommitteeScenarioSourceKind::Fixture
                        | CommitteeScenarioSourceKind::SyntheticTest
                )
        })
        .count();
    let actionability_status = if decision_count < 3 {
        CommitteeActionabilityStatus::NotEnoughRows
    } else if fixture_only_count == decision_count && decision_count > 0 {
        CommitteeActionabilityStatus::FixtureOnly
    } else if research_only_count == decision_count && decision_count > 0 {
        CommitteeActionabilityStatus::ResearchOnly
    } else if final_denied_count * 2 >= decision_count {
        CommitteeActionabilityStatus::MostlyRiskDenied
    } else if final_no_trade_count * 2 >= decision_count {
        CommitteeActionabilityStatus::MostlyNoTrade
    } else {
        CommitteeActionabilityStatus::ActionableResearch
    };
    CommitteeActionabilityReport {
        decision_count,
        actionable_count,
        paper_approve_count,
        paper_reduce_size_count,
        human_confirm_required_count,
        final_no_trade_count,
        final_denied_count,
        research_only_count: research_only_count.min(scenario_set.research_only_row_count),
        fixture_only_count: fixture_only_count.min(scenario_set.fixture_row_count),
        official_actionable_count,
        actionability_ratio: actionable_count as f64 / decision_count.max(1) as f64,
        official_actionability_ratio: official_actionable_count as f64
            / decision_count.max(1) as f64,
        risk_block_ratio: final_denied_count as f64 / decision_count.max(1) as f64,
        confirm_ratio: human_confirm_required_count as f64 / decision_count.max(1) as f64,
        actionability_status,
        reason_codes: vec![ReasonCode::CommitteeActionabilityBuilt],
    }
}

impl CommitteeActionabilityReport {
    pub fn to_text(&self) -> String {
        [
            format!("decision_count={}", self.decision_count),
            format!("actionable_count={}", self.actionable_count),
            format!("paper_approve_count={}", self.paper_approve_count),
            format!("paper_reduce_size_count={}", self.paper_reduce_size_count),
            format!(
                "human_confirm_required_count={}",
                self.human_confirm_required_count
            ),
            format!("final_no_trade_count={}", self.final_no_trade_count),
            format!("final_denied_count={}", self.final_denied_count),
            format!("research_only_count={}", self.research_only_count),
            format!("fixture_only_count={}", self.fixture_only_count),
            format!(
                "official_actionable_count={}",
                self.official_actionable_count
            ),
            format!("actionability_ratio={:.6}", self.actionability_ratio),
            format!(
                "official_actionability_ratio={:.6}",
                self.official_actionability_ratio
            ),
            format!("risk_block_ratio={:.6}", self.risk_block_ratio),
            format!("confirm_ratio={:.6}", self.confirm_ratio),
            format!("actionability_status={:?}", self.actionability_status),
        ]
        .join("\n")
    }
}
