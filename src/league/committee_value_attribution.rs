use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CommitteeValueAttributionInputs {
    pub comparable_rows: usize,
    pub official_comparable_rows: usize,
    #[serde(default)]
    pub committee_action_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub baseline_action_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub no_trade_baseline_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub external_action_counts: Option<BTreeMap<String, usize>>,
    #[serde(default)]
    pub persona_contribution_summary: BTreeMap<String, String>,
    #[serde(default)]
    pub chair_contribution_summary: BTreeMap<String, usize>,
    #[serde(default)]
    pub risk_contribution_summary: BTreeMap<String, usize>,
    #[serde(default)]
    pub source_contribution_summary: BTreeMap<String, usize>,
    #[serde(default)]
    pub committee_vs_baseline_delta: Option<f64>,
    #[serde(default)]
    pub committee_vs_no_trade_delta: Option<f64>,
    #[serde(default)]
    pub committee_vs_external_delta: Option<f64>,
    #[serde(default)]
    pub chair_dominated: bool,
    #[serde(default)]
    pub persona_dominated: bool,
    #[serde(default)]
    pub risk_dominated: bool,
    #[serde(default)]
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteeValueAttributionStatus {
    CommitteeAddsValue,
    CommitteeNoBetterThanBaseline,
    CommitteeWorseThanBaseline,
    CommitteeMostlyNoTrade,
    CommitteeMostlyRiskDenied,
    ChairDominated,
    #[default]
    PersonaDominated,
    RiskDominated,
    InsufficientComparableRows,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeValueAttributionReport {
    pub comparable_rows: usize,
    pub committee_action_counts: BTreeMap<String, usize>,
    pub baseline_action_counts: BTreeMap<String, usize>,
    pub no_trade_baseline_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub external_action_counts: Option<BTreeMap<String, usize>>,
    pub persona_contribution_summary: BTreeMap<String, String>,
    pub chair_contribution_summary: BTreeMap<String, usize>,
    pub risk_contribution_summary: BTreeMap<String, usize>,
    pub source_contribution_summary: BTreeMap<String, usize>,
    #[serde(default)]
    pub committee_vs_baseline_delta: Option<f64>,
    #[serde(default)]
    pub committee_vs_no_trade_delta: Option<f64>,
    #[serde(default)]
    pub committee_vs_external_delta: Option<f64>,
    pub attribution_status: CommitteeValueAttributionStatus,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_committee_value_attribution_report(
    inputs: &CommitteeValueAttributionInputs,
) -> CommitteeValueAttributionReport {
    let no_trade_count = inputs
        .committee_action_counts
        .get("FinalNoTrade")
        .copied()
        .unwrap_or(0);
    let denied_count = inputs
        .committee_action_counts
        .get("FinalDenied")
        .copied()
        .unwrap_or(0);
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let status = if inputs.comparable_rows == 0 {
        blockers.push("committee rows are not comparable yet".to_string());
        CommitteeValueAttributionStatus::InsufficientComparableRows
    } else if inputs.diagnostic_only || inputs.official_comparable_rows == 0 {
        warnings.push(
            "committee value remains diagnostic without official comparable rows".to_string(),
        );
        CommitteeValueAttributionStatus::DiagnosticOnly
    } else if no_trade_count * 2 >= inputs.comparable_rows {
        CommitteeValueAttributionStatus::CommitteeMostlyNoTrade
    } else if denied_count * 2 >= inputs.comparable_rows {
        CommitteeValueAttributionStatus::CommitteeMostlyRiskDenied
    } else if inputs.chair_dominated {
        warnings.push("chair behavior dominates the committee path".to_string());
        CommitteeValueAttributionStatus::ChairDominated
    } else if inputs.risk_dominated {
        warnings.push("risk behavior dominates committee attribution".to_string());
        CommitteeValueAttributionStatus::RiskDominated
    } else if inputs.persona_dominated {
        warnings.push("one persona dominates committee attribution".to_string());
        CommitteeValueAttributionStatus::PersonaDominated
    } else if inputs.committee_vs_baseline_delta.unwrap_or_default() < 0.0 {
        CommitteeValueAttributionStatus::CommitteeWorseThanBaseline
    } else if inputs.committee_vs_baseline_delta.unwrap_or_default() > 0.0 {
        CommitteeValueAttributionStatus::CommitteeAddsValue
    } else {
        CommitteeValueAttributionStatus::CommitteeNoBetterThanBaseline
    };

    let mut reason_codes = inputs.reason_codes.clone();
    reason_codes.push(ReasonCode::CommitteeValueAttributionReportBuilt);
    if matches!(
        status,
        CommitteeValueAttributionStatus::CommitteeWorseThanBaseline
            | CommitteeValueAttributionStatus::DiagnosticOnly
            | CommitteeValueAttributionStatus::InsufficientComparableRows
    ) {
        reason_codes.push(ReasonCode::EvidenceStillInsufficient);
    }

    CommitteeValueAttributionReport {
        comparable_rows: inputs.comparable_rows,
        committee_action_counts: inputs.committee_action_counts.clone(),
        baseline_action_counts: inputs.baseline_action_counts.clone(),
        no_trade_baseline_counts: inputs.no_trade_baseline_counts.clone(),
        external_action_counts: inputs.external_action_counts.clone(),
        persona_contribution_summary: inputs.persona_contribution_summary.clone(),
        chair_contribution_summary: inputs.chair_contribution_summary.clone(),
        risk_contribution_summary: inputs.risk_contribution_summary.clone(),
        source_contribution_summary: inputs.source_contribution_summary.clone(),
        committee_vs_baseline_delta: inputs.committee_vs_baseline_delta,
        committee_vs_no_trade_delta: inputs.committee_vs_no_trade_delta,
        committee_vs_external_delta: inputs.committee_vs_external_delta,
        attribution_status: status,
        blockers,
        warnings,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

impl CommitteeValueAttributionReport {
    pub fn to_text(&self) -> String {
        let committee_counts = self
            .committee_action_counts
            .iter()
            .map(|(key, value)| format!("{key}:{value}"))
            .collect::<Vec<_>>()
            .join("|");
        let baseline_counts = self
            .baseline_action_counts
            .iter()
            .map(|(key, value)| format!("{key}:{value}"))
            .collect::<Vec<_>>()
            .join("|");
        let no_trade_counts = self
            .no_trade_baseline_counts
            .iter()
            .map(|(key, value)| format!("{key}:{value}"))
            .collect::<Vec<_>>()
            .join("|");
        let external_counts = self
            .external_action_counts
            .as_ref()
            .map(|counts| {
                counts
                    .iter()
                    .map(|(key, value)| format!("{key}:{value}"))
                    .collect::<Vec<_>>()
                    .join("|")
            })
            .unwrap_or_default();
        [
            format!("comparable_rows={}", self.comparable_rows),
            format!("committee_action_counts={committee_counts}"),
            format!("baseline_action_counts={baseline_counts}"),
            format!("no_trade_baseline_counts={no_trade_counts}"),
            format!("external_action_counts={external_counts}"),
            format!(
                "committee_vs_baseline_delta={}",
                self.committee_vs_baseline_delta
                    .map(|value| format!("{value:.6}"))
                    .unwrap_or_default()
            ),
            format!(
                "committee_vs_no_trade_delta={}",
                self.committee_vs_no_trade_delta
                    .map(|value| format!("{value:.6}"))
                    .unwrap_or_default()
            ),
            format!(
                "committee_vs_external_delta={}",
                self.committee_vs_external_delta
                    .map(|value| format!("{value:.6}"))
                    .unwrap_or_default()
            ),
            format!("attribution_status={:?}", self.attribution_status),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}
