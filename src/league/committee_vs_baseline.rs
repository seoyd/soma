use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::committee_replay::CommitteeReplayReport;
use super::committee_scenario_loader::CommitteeScenarioSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeVsBaselineStatus {
    Comparable,
    NotEnoughComparableRows,
    NoBaselineReference,
    NoOutcomeReference,
    ResearchOnly,
    FixtureOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeVsBaselineComparison {
    pub total_comparable_rows: usize,
    pub committee_action_counts: BTreeMap<String, usize>,
    pub baseline_action_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub external_action_counts: Option<BTreeMap<String, usize>>,
    pub no_trade_baseline_counts: BTreeMap<String, usize>,
    pub committee_vs_baseline_delta: BTreeMap<String, isize>,
    #[serde(default)]
    pub committee_vs_external_delta: Option<BTreeMap<String, isize>>,
    pub committee_vs_notrade_delta: BTreeMap<String, isize>,
    pub risk_denied_value_proxy: f64,
    pub outcome_available_count: usize,
    pub comparison_status: CommitteeVsBaselineStatus,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_committee_vs_baseline_comparison(
    scenario_set: &CommitteeScenarioSet,
    replay_report: &CommitteeReplayReport,
) -> CommitteeVsBaselineComparison {
    let mut baseline_action_counts = BTreeMap::new();
    let mut external_action_counts = BTreeMap::new();
    let mut outcome_available_count = 0usize;
    let mut baseline_reference_count = 0usize;
    let mut external_reference_count = 0usize;
    let mut risk_denied_with_counterfactual = 0usize;
    for record in &replay_report.records {
        if let Some(summary) = &record.scenario_row.baseline_signal_summary {
            baseline_reference_count += 1;
            *baseline_action_counts.entry(summary.clone()).or_insert(0) += 1;
        }
        if let Some(summary) = &record.scenario_row.external_prediction_summary {
            external_reference_count += 1;
            *external_action_counts.entry(summary.clone()).or_insert(0) += 1;
        }
        if record.scenario_row.outcome_reference.is_some() {
            outcome_available_count += 1;
        }
        if record.scenario_row.risk_denial_counterfactual.is_some()
            && matches!(
                record.final_action,
                super::committee_risk_bridge::CommitteeFinalAction::FinalDenied
            )
        {
            risk_denied_with_counterfactual += 1;
        }
    }
    let no_trade_baseline_counts =
        BTreeMap::from([("NoTradeBaseline".to_string(), replay_report.record_count)]);
    let committee_vs_baseline_delta =
        delta_counts(&replay_report.final_action_counts, &baseline_action_counts);
    let committee_vs_external_delta = if external_reference_count > 0 {
        Some(delta_counts(
            &replay_report.final_action_counts,
            &external_action_counts,
        ))
    } else {
        None
    };
    let committee_vs_notrade_delta = delta_counts(
        &replay_report.final_action_counts,
        &no_trade_baseline_counts,
    );
    let comparison_status = if replay_report.record_count < 3 {
        CommitteeVsBaselineStatus::NotEnoughComparableRows
    } else if scenario_set.fixture_row_count == scenario_set.row_count && scenario_set.row_count > 0
    {
        CommitteeVsBaselineStatus::FixtureOnly
    } else if scenario_set.research_only_row_count == scenario_set.row_count
        && scenario_set.row_count > 0
    {
        CommitteeVsBaselineStatus::ResearchOnly
    } else if baseline_reference_count == 0 {
        CommitteeVsBaselineStatus::NoBaselineReference
    } else if outcome_available_count == 0 {
        CommitteeVsBaselineStatus::NoOutcomeReference
    } else {
        CommitteeVsBaselineStatus::Comparable
    };
    CommitteeVsBaselineComparison {
        total_comparable_rows: replay_report.record_count,
        committee_action_counts: replay_report.final_action_counts.clone(),
        baseline_action_counts,
        external_action_counts: (external_reference_count > 0).then_some(external_action_counts),
        no_trade_baseline_counts,
        committee_vs_baseline_delta,
        committee_vs_external_delta,
        committee_vs_notrade_delta,
        risk_denied_value_proxy: risk_denied_with_counterfactual as f64
            / replay_report.record_count.max(1) as f64,
        outcome_available_count,
        comparison_status,
        reason_codes: vec![ReasonCode::CommitteeVsBaselineBuilt],
    }
}

impl CommitteeVsBaselineComparison {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("comparison_status={:?}", self.comparison_status),
            format!("total_comparable_rows={}", self.total_comparable_rows),
            format!("outcome_available_count={}", self.outcome_available_count),
            format!(
                "risk_denied_value_proxy={:.6}",
                self.risk_denied_value_proxy
            ),
        ];
        for (action, count) in &self.committee_action_counts {
            lines.push(format!("committee_action={action};count={count}"));
        }
        for (action, count) in &self.baseline_action_counts {
            lines.push(format!("baseline_action={action};count={count}"));
        }
        for (action, delta) in &self.committee_vs_notrade_delta {
            lines.push(format!("committee_vs_notrade={action};delta={delta}"));
        }
        lines.join("\n")
    }
}

fn delta_counts(
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
