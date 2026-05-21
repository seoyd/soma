use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{EvidenceSourceKind, ProviderMarket};

use super::committee_counterfactual_builder::{
    CommitteeCounterfactualRecord, CommitteeCounterfactualType,
};
use super::committee_outcome_coverage::{
    CommitteeOutcomeCoverageReport, CommitteeOutcomeCoverageStatus,
};
use super::committee_outcome_linker::OutcomeLinkedCommitteeScenarioPack;
use super::committee_outcome_reference::CommitteeOutcomeReference;
use super::committee_replay::CommitteeReplayReport;
use super::committee_risk_bridge::CommitteeFinalAction;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceStrength {
    StrongOfficial,
    ModerateOfficial,
    CryptoOnly,
    ResearchOnly,
    FixtureOnly,
    DiagnosticOnly,
    Insufficient,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteePerformanceStatus {
    EvidencePositive,
    EvidenceMixed,
    EvidenceNegative,
    EvidenceInsufficient,
    ResearchOnly,
    FixtureOnly,
    CryptoOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PerformanceEvidenceCell {
    pub source_kind: String,
    pub market: ProviderMarket,
    pub symbol: String,
    pub timeframe: String,
    pub horizon_bars: usize,
    pub committee_action: String,
    #[serde(default)]
    pub baseline_action: Option<String>,
    #[serde(default)]
    pub external_action: Option<String>,
    #[serde(default)]
    pub outcome_label: Option<String>,
    #[serde(default)]
    pub net_return_pct: Option<f64>,
    #[serde(default)]
    pub no_trade_value_proxy: Option<f64>,
    #[serde(default)]
    pub risk_denied_value_proxy: Option<f64>,
    #[serde(default)]
    pub committee_vs_baseline_delta: Option<f64>,
    #[serde(default)]
    pub committee_vs_notrade_delta: Option<f64>,
    #[serde(default)]
    pub committee_vs_external_delta: Option<f64>,
    pub evidence_strength: EvidenceStrength,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteePerformanceEvidenceMatrix {
    pub matrix_id: String,
    pub cells: Vec<PerformanceEvidenceCell>,
    pub total_comparable_rows: usize,
    pub official_comparable_rows: usize,
    pub crypto_comparable_rows: usize,
    pub research_only_rows: usize,
    pub fixture_rows: usize,
    pub committee_better_than_baseline_count: usize,
    pub committee_worse_than_baseline_count: usize,
    pub committee_better_than_notrade_count: usize,
    pub risk_denied_defensive_value_total: f64,
    pub no_trade_defensive_value_total: f64,
    pub outcome_coverage_status: CommitteeOutcomeCoverageStatus,
    pub performance_status: CommitteePerformanceStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_committee_performance_evidence_matrix(
    matrix_id: &str,
    coverage_report: &CommitteeOutcomeCoverageReport,
    linked_packs: &[OutcomeLinkedCommitteeScenarioPack],
    replay_reports: &[CommitteeReplayReport],
    counterfactual_records: &[CommitteeCounterfactualRecord],
    allow_estimated_counterfactuals: bool,
) -> CommitteePerformanceEvidenceMatrix {
    let replay_by_row_id = replay_reports
        .iter()
        .flat_map(|report| {
            report
                .records
                .iter()
                .map(|record| (record.scenario_row.scenario_row_id.clone(), record))
                .collect::<Vec<_>>()
        })
        .collect::<BTreeMap<_, _>>();
    let counterfactuals_by_row_id = counterfactual_records.iter().fold(
        BTreeMap::<String, Vec<&CommitteeCounterfactualRecord>>::new(),
        |mut acc, record| {
            acc.entry(record.scenario_row_id.clone())
                .or_default()
                .push(record);
            acc
        },
    );
    let mut cells = Vec::new();
    for linked_pack in linked_packs {
        for linked_row in &linked_pack.linked_rows {
            let Some(outcome) = linked_row.outcome_reference.as_ref() else {
                continue;
            };
            let replay_record = replay_by_row_id
                .get(&linked_row.scenario_row.scenario_row_id)
                .copied();
            let counterfactuals = counterfactuals_by_row_id
                .get(&linked_row.scenario_row.scenario_row_id)
                .cloned()
                .unwrap_or_default();
            let no_trade_record = counterfactuals.iter().copied().find(|record| {
                record.counterfactual_type == CommitteeCounterfactualType::NoTrade
                    && record.built()
                    && (allow_estimated_counterfactuals || !record.diagnostic_only)
            });
            let risk_denied_record = counterfactuals.iter().copied().find(|record| {
                record.counterfactual_type == CommitteeCounterfactualType::RiskDenied
                    && record.built()
                    && (allow_estimated_counterfactuals || !record.diagnostic_only)
            });
            let committee_action = replay_record
                .map(|record| format!("{:?}", record.final_action))
                .unwrap_or_else(|| "Unknown".to_string());
            let committee_return =
                replay_record.map(|record| committee_return_proxy(record.final_action, outcome));
            let baseline_action = linked_row
                .baseline_reference
                .as_ref()
                .map(|reference| reference.baseline_action.as_summary_str().to_string());
            let baseline_return = linked_row.baseline_reference.as_ref().map(|reference| {
                scaled_return(reference.baseline_action.sizing_multiplier(), outcome)
            });
            let external_action = linked_row
                .external_reference
                .as_ref()
                .and_then(|reference| {
                    reference.prediction_schema_valid.then(|| {
                        reference
                            .action_as_baseline_action()
                            .as_summary_str()
                            .to_string()
                    })
                });
            let external_return = linked_row
                .external_reference
                .as_ref()
                .and_then(|reference| {
                    reference.prediction_schema_valid.then(|| {
                        scaled_return(
                            reference.action_as_baseline_action().sizing_multiplier(),
                            outcome,
                        )
                    })
                });
            let no_trade_value_proxy = no_trade_record
                .and_then(|record| record.avoided_loss_value)
                .or_else(|| {
                    Some(
                        outcome
                            .cost_adjusted_return_pct()
                            .unwrap_or_default()
                            .min(0.0)
                            .abs(),
                    )
                });
            let risk_denied_value_proxy =
                risk_denied_record.and_then(|record| record.avoided_loss_value);
            let diagnostic_only = no_trade_record.is_some_and(|record| record.diagnostic_only)
                || risk_denied_record.is_some_and(|record| record.diagnostic_only)
                || !outcome.no_lookahead_safe;
            let evidence_strength = evidence_strength_for_row(
                &linked_row.scenario_row,
                committee_return,
                diagnostic_only,
            );
            let mut reason_codes = linked_row.reason_codes.clone();
            reason_codes.extend(outcome.reason_codes.clone());
            if let Some(record) = no_trade_record {
                reason_codes.extend(record.reason_codes.clone());
            }
            if let Some(record) = risk_denied_record {
                reason_codes.extend(record.reason_codes.clone());
            }
            cells.push(PerformanceEvidenceCell {
                source_kind: format!("{:?}", linked_row.scenario_row.evidence_source_kind),
                market: linked_row.scenario_row.market,
                symbol: linked_row.scenario_row.symbol.clone(),
                timeframe: match linked_row.scenario_row.target_horizon {
                    super::persona_card_lite::PersonaHorizon::Intraday => "intraday".to_string(),
                    super::persona_card_lite::PersonaHorizon::Swing => "swing".to_string(),
                    super::persona_card_lite::PersonaHorizon::MultiDay => "multi_day".to_string(),
                    super::persona_card_lite::PersonaHorizon::LongTerm => "long_term".to_string(),
                },
                horizon_bars: outcome.horizon_bars,
                committee_action,
                baseline_action,
                external_action,
                outcome_label: Some(format!("{:?}", outcome.triple_barrier_label)),
                net_return_pct: committee_return,
                no_trade_value_proxy,
                risk_denied_value_proxy,
                committee_vs_baseline_delta: committee_return
                    .zip(baseline_return)
                    .map(|(a, b)| a - b),
                committee_vs_notrade_delta: committee_return,
                committee_vs_external_delta: committee_return
                    .zip(external_return)
                    .map(|(a, b)| a - b),
                evidence_strength,
                reason_codes: stable_reason_codes(&reason_codes),
            });
        }
    }
    cells.sort_by(|left, right| {
        left.symbol
            .cmp(&right.symbol)
            .then(left.timeframe.cmp(&right.timeframe))
            .then(left.horizon_bars.cmp(&right.horizon_bars))
            .then(left.committee_action.cmp(&right.committee_action))
    });

    let total_comparable_rows = cells.len();
    let official_comparable_rows = cells
        .iter()
        .filter(|cell| {
            matches!(
                cell.evidence_strength,
                EvidenceStrength::StrongOfficial | EvidenceStrength::ModerateOfficial
            )
        })
        .count();
    let crypto_comparable_rows = cells
        .iter()
        .filter(|cell| cell.market == ProviderMarket::Crypto)
        .count();
    let research_only_rows = cells
        .iter()
        .filter(|cell| cell.evidence_strength == EvidenceStrength::ResearchOnly)
        .count();
    let fixture_rows = cells
        .iter()
        .filter(|cell| cell.evidence_strength == EvidenceStrength::FixtureOnly)
        .count();
    let committee_better_than_baseline_count = cells
        .iter()
        .filter(|cell| cell.committee_vs_baseline_delta.unwrap_or_default() > 0.0)
        .count();
    let committee_worse_than_baseline_count = cells
        .iter()
        .filter(|cell| cell.committee_vs_baseline_delta.unwrap_or_default() < 0.0)
        .count();
    let committee_better_than_notrade_count = cells
        .iter()
        .filter(|cell| cell.committee_vs_notrade_delta.unwrap_or_default() > 0.0)
        .count();
    let risk_denied_defensive_value_total = cells
        .iter()
        .filter_map(|cell| cell.risk_denied_value_proxy)
        .sum::<f64>();
    let no_trade_defensive_value_total = cells
        .iter()
        .filter_map(|cell| cell.no_trade_value_proxy)
        .sum::<f64>();
    let performance_status = if total_comparable_rows == 0 {
        CommitteePerformanceStatus::EvidenceInsufficient
    } else if fixture_rows == total_comparable_rows {
        CommitteePerformanceStatus::FixtureOnly
    } else if research_only_rows == total_comparable_rows {
        CommitteePerformanceStatus::ResearchOnly
    } else if crypto_comparable_rows == total_comparable_rows {
        CommitteePerformanceStatus::CryptoOnly
    } else if official_comparable_rows == 0 {
        CommitteePerformanceStatus::EvidenceInsufficient
    } else if committee_better_than_baseline_count > committee_worse_than_baseline_count {
        CommitteePerformanceStatus::EvidencePositive
    } else if committee_worse_than_baseline_count > committee_better_than_baseline_count {
        CommitteePerformanceStatus::EvidenceNegative
    } else {
        CommitteePerformanceStatus::EvidenceMixed
    };
    CommitteePerformanceEvidenceMatrix {
        matrix_id: matrix_id.to_string(),
        cells,
        total_comparable_rows,
        official_comparable_rows,
        crypto_comparable_rows,
        research_only_rows,
        fixture_rows,
        committee_better_than_baseline_count,
        committee_worse_than_baseline_count,
        committee_better_than_notrade_count,
        risk_denied_defensive_value_total,
        no_trade_defensive_value_total,
        outcome_coverage_status: coverage_report.coverage_status,
        performance_status,
        reason_codes: stable_reason_codes(&[ReasonCode::CommitteePerformanceEvidenceMatrixBuilt]),
    }
}

impl CommitteePerformanceEvidenceMatrix {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("matrix_id={}", self.matrix_id),
            format!("performance_status={:?}", self.performance_status),
            format!("outcome_coverage_status={:?}", self.outcome_coverage_status),
            format!("total_comparable_rows={}", self.total_comparable_rows),
            format!("official_comparable_rows={}", self.official_comparable_rows),
            format!("crypto_comparable_rows={}", self.crypto_comparable_rows),
            format!("research_only_rows={}", self.research_only_rows),
            format!("fixture_rows={}", self.fixture_rows),
            format!(
                "committee_better_than_baseline_count={}",
                self.committee_better_than_baseline_count
            ),
            format!(
                "committee_worse_than_baseline_count={}",
                self.committee_worse_than_baseline_count
            ),
            format!(
                "committee_better_than_notrade_count={}",
                self.committee_better_than_notrade_count
            ),
            format!(
                "risk_denied_defensive_value_total={}",
                crate::core::deterministic_float_format(self.risk_denied_defensive_value_total)
            ),
            format!(
                "no_trade_defensive_value_total={}",
                crate::core::deterministic_float_format(self.no_trade_defensive_value_total)
            ),
        ];
        for cell in &self.cells {
            lines.push(format!(
                "cell=symbol:{};market:{:?};committee_action:{};baseline_action:{};external_action:{};net_return_pct:{};committee_vs_baseline_delta:{};committee_vs_notrade_delta:{};committee_vs_external_delta:{};evidence_strength:{:?}",
                cell.symbol,
                cell.market,
                cell.committee_action,
                cell.baseline_action.clone().unwrap_or_default(),
                cell.external_action.clone().unwrap_or_default(),
                cell.net_return_pct
                    .map(crate::core::deterministic_float_format)
                    .unwrap_or_default(),
                cell.committee_vs_baseline_delta
                    .map(crate::core::deterministic_float_format)
                    .unwrap_or_default(),
                cell.committee_vs_notrade_delta
                    .map(crate::core::deterministic_float_format)
                    .unwrap_or_default(),
                cell.committee_vs_external_delta
                    .map(crate::core::deterministic_float_format)
                    .unwrap_or_default(),
                cell.evidence_strength,
            ));
        }
        lines.join("\n")
    }
}

fn evidence_strength_for_row(
    row: &super::committee_scenario_loader::CommitteeScenarioRow,
    committee_return: Option<f64>,
    diagnostic_only: bool,
) -> EvidenceStrength {
    if diagnostic_only {
        return EvidenceStrength::DiagnosticOnly;
    }
    if row.evidence_source_kind == EvidenceSourceKind::YFinanceResearch {
        return EvidenceStrength::ResearchOnly;
    }
    if super::committee_counterfactual_builder::fixture_source_kind(row) {
        return EvidenceStrength::FixtureOnly;
    }
    if row.market == ProviderMarket::Crypto {
        return EvidenceStrength::CryptoOnly;
    }
    if row.evidence_source_kind.readiness_eligible() && committee_return.is_some() {
        EvidenceStrength::StrongOfficial
    } else if row.evidence_source_kind.readiness_eligible() {
        EvidenceStrength::ModerateOfficial
    } else {
        EvidenceStrength::Insufficient
    }
}

fn committee_return_proxy(
    final_action: CommitteeFinalAction,
    outcome: &CommitteeOutcomeReference,
) -> f64 {
    match final_action {
        CommitteeFinalAction::PaperApprove => scaled_return(1.0, outcome),
        CommitteeFinalAction::PaperReduceSize | CommitteeFinalAction::HumanConfirmRequired => {
            scaled_return(0.5, outcome)
        }
        CommitteeFinalAction::FinalNoTrade | CommitteeFinalAction::FinalDenied => 0.0,
    }
}

fn scaled_return(multiplier: f64, outcome: &CommitteeOutcomeReference) -> f64 {
    multiplier * outcome.cost_adjusted_return_pct().unwrap_or_default()
}
