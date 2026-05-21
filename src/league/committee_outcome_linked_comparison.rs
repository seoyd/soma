use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::committee_outcome_linker::OutcomeLinkedCommitteeScenarioPack;
use super::committee_outcome_reference::{CommitteeBaselineAction, CommitteeOutcomeReference};
use super::committee_replay::CommitteeReplayReport;
use super::committee_risk_bridge::CommitteeFinalAction;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeOutcomeLinkedComparisonStatus {
    Comparable,
    NotEnoughOutcomeLinks,
    NoBaselineReferences,
    NoOutcomeReferences,
    ResearchOnly,
    FixtureOnly,
    CryptoOnly,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeOutcomeLinkedComparison {
    pub comparable_rows: usize,
    pub outcome_linked_rows: usize,
    pub committee_final_action_counts: BTreeMap<String, usize>,
    pub baseline_action_counts: BTreeMap<String, usize>,
    pub no_trade_baseline_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub external_action_counts: Option<BTreeMap<String, usize>>,
    #[serde(default)]
    pub committee_net_return_proxy: Option<f64>,
    #[serde(default)]
    pub baseline_net_return_proxy: Option<f64>,
    pub no_trade_value_proxy: f64,
    pub risk_denied_value_proxy: f64,
    #[serde(default)]
    pub committee_vs_baseline_delta: Option<f64>,
    #[serde(default)]
    pub committee_vs_no_trade_delta: Option<f64>,
    #[serde(default)]
    pub committee_vs_external_delta: Option<f64>,
    pub comparison_status: CommitteeOutcomeLinkedComparisonStatus,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_committee_outcome_linked_comparison(
    linked_pack: &OutcomeLinkedCommitteeScenarioPack,
    replay_report: &CommitteeReplayReport,
    min_outcome_linked_rows: usize,
) -> CommitteeOutcomeLinkedComparison {
    let linked_by_row_id = linked_pack
        .linked_rows
        .iter()
        .map(|row| (row.scenario_row.scenario_row_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let mut committee_final_action_counts = BTreeMap::new();
    let mut baseline_action_counts = BTreeMap::new();
    let mut external_action_counts = BTreeMap::new();
    let mut no_trade_baseline_counts = BTreeMap::new();
    let mut comparable_rows = 0usize;
    let mut committee_net_return_proxy = 0.0;
    let mut baseline_net_return_proxy = 0.0;
    let mut external_net_return_proxy = 0.0;
    let mut external_rows = 0usize;
    let mut risk_denied_value_proxy = 0.0;

    for record in &replay_report.records {
        let Some(linked_row) = linked_by_row_id.get(&record.scenario_row.scenario_row_id) else {
            continue;
        };
        let Some(outcome) = linked_row.outcome_reference.as_ref() else {
            continue;
        };
        comparable_rows += 1;
        *committee_final_action_counts
            .entry(format!("{:?}", record.final_action))
            .or_insert(0) += 1;
        *no_trade_baseline_counts
            .entry(
                CommitteeBaselineAction::NoTrade
                    .as_summary_str()
                    .to_string(),
            )
            .or_insert(0) += 1;
        if let Some(reference) = &linked_row.baseline_reference {
            *baseline_action_counts
                .entry(reference.baseline_action.as_summary_str().to_string())
                .or_insert(0) += 1;
            baseline_net_return_proxy +=
                scaled_return(reference.baseline_action.sizing_multiplier(), outcome);
        }
        if let Some(reference) = &linked_row.external_reference {
            if reference.prediction_schema_valid {
                let action = reference.action_as_baseline_action();
                *external_action_counts
                    .entry(action.as_summary_str().to_string())
                    .or_insert(0) += 1;
                external_net_return_proxy += scaled_return(action.sizing_multiplier(), outcome);
                external_rows += 1;
            }
        }
        committee_net_return_proxy += committee_return_proxy(record.final_action, outcome);
        if record.final_action == CommitteeFinalAction::FinalDenied
            && (linked_row.scenario_row.risk_denial_counterfactual.is_some()
                || outcome.risk_denial_counterfactual())
        {
            risk_denied_value_proxy += outcome
                .cost_adjusted_return_pct()
                .map(|value| value.min(0.0).abs())
                .unwrap_or_default();
        }
    }

    let committee_net_return_proxy = (comparable_rows > 0).then_some(committee_net_return_proxy);
    let baseline_net_return_proxy =
        (!baseline_action_counts.is_empty()).then_some(baseline_net_return_proxy);
    let external_net_return_proxy = (external_rows > 0).then_some(external_net_return_proxy);
    let no_trade_value_proxy = 0.0;
    let comparison_status = if linked_pack.pack.fixture_ratio() >= 0.999
        && linked_pack.pack.row_count() > 0
    {
        CommitteeOutcomeLinkedComparisonStatus::FixtureOnly
    } else if linked_pack.pack.research_only_ratio() >= 0.999 && linked_pack.pack.row_count() > 0 {
        CommitteeOutcomeLinkedComparisonStatus::ResearchOnly
    } else if linked_pack.pack.crypto_only_ratio() >= 0.999 && linked_pack.pack.row_count() > 0 {
        CommitteeOutcomeLinkedComparisonStatus::CryptoOnly
    } else if linked_pack.outcome_linked_count == 0 {
        CommitteeOutcomeLinkedComparisonStatus::NotEnoughOutcomeLinks
    } else if baseline_action_counts.is_empty() {
        CommitteeOutcomeLinkedComparisonStatus::NoBaselineReferences
    } else if comparable_rows < min_outcome_linked_rows {
        CommitteeOutcomeLinkedComparisonStatus::NotEnoughOutcomeLinks
    } else if comparable_rows == 0 {
        CommitteeOutcomeLinkedComparisonStatus::NoOutcomeReferences
    } else if committee_net_return_proxy.is_none() {
        CommitteeOutcomeLinkedComparisonStatus::DiagnosticOnly
    } else {
        CommitteeOutcomeLinkedComparisonStatus::Comparable
    };
    CommitteeOutcomeLinkedComparison {
        comparable_rows,
        outcome_linked_rows: linked_pack.outcome_linked_count,
        committee_final_action_counts,
        baseline_action_counts,
        no_trade_baseline_counts,
        external_action_counts: (!external_action_counts.is_empty())
            .then_some(external_action_counts),
        committee_net_return_proxy,
        baseline_net_return_proxy,
        no_trade_value_proxy,
        risk_denied_value_proxy,
        committee_vs_baseline_delta: committee_net_return_proxy
            .zip(baseline_net_return_proxy)
            .map(|(committee, baseline)| committee - baseline),
        committee_vs_no_trade_delta: committee_net_return_proxy
            .map(|committee| committee - no_trade_value_proxy),
        committee_vs_external_delta: committee_net_return_proxy
            .zip(external_net_return_proxy)
            .map(|(committee, external)| committee - external),
        comparison_status,
        reason_codes: vec![ReasonCode::CommitteeOutcomeLinkedComparisonBuilt],
    }
}

impl CommitteeOutcomeLinkedComparison {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("comparison_status={:?}", self.comparison_status),
            format!("comparable_rows={}", self.comparable_rows),
            format!("outcome_linked_rows={}", self.outcome_linked_rows),
            format!(
                "committee_net_return_proxy={:.6}",
                self.committee_net_return_proxy.unwrap_or_default()
            ),
            format!(
                "baseline_net_return_proxy={:.6}",
                self.baseline_net_return_proxy.unwrap_or_default()
            ),
            format!("no_trade_value_proxy={:.6}", self.no_trade_value_proxy),
            format!(
                "risk_denied_value_proxy={:.6}",
                self.risk_denied_value_proxy
            ),
        ];
        for (action, count) in &self.committee_final_action_counts {
            lines.push(format!("committee_action={action};count={count}"));
        }
        for (action, count) in &self.baseline_action_counts {
            lines.push(format!("baseline_action={action};count={count}"));
        }
        if let Some(external_action_counts) = &self.external_action_counts {
            for (action, count) in external_action_counts {
                lines.push(format!("external_action={action};count={count}"));
            }
        }
        lines.join("\n")
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

pub fn delta_counts(
    left: &BTreeMap<String, usize>,
    right: &BTreeMap<String, usize>,
) -> BTreeMap<String, isize> {
    let mut keys = BTreeSet::new();
    keys.extend(left.keys().cloned());
    keys.extend(right.keys().cloned());
    keys.into_iter()
        .map(|key| {
            let left_value = left.get(&key).copied().unwrap_or(0) as isize;
            let right_value = right.get(&key).copied().unwrap_or(0) as isize;
            (key, left_value - right_value)
        })
        .collect()
}
