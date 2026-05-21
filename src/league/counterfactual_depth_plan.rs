use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CounterfactualGapKind {
    MissingNoTradeCounterfactual,
    MissingRiskDeniedCounterfactual,
    MissingOutcomeReference,
    MissingBaselineReference,
    MissingLocalCandles,
    MissingTimestampAlignment,
    MissingFutureWindow,
    NoLookaheadViolation,
    SourceNotEligible,
    #[default]
    SummaryDerivedOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CounterfactualSuggestedBuilder {
    CommitteeReferencePackRunner,
    CommitteeCounterfactualAuditRunner,
    CommitteeOutcomeLinker,
    OfficialEvidenceReplicationRunner,
    ManualOperatorAction,
    #[default]
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CounterfactualDepthPlanItem {
    pub row_id: String,
    pub gap_kind: CounterfactualGapKind,
    pub required_artifact: String,
    pub can_build_from_available_data: bool,
    pub suggested_builder: CounterfactualSuggestedBuilder,
    #[serde(default)]
    pub operator_action: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CounterfactualDepthPlan {
    pub plan_id: String,
    pub items: Vec<CounterfactualDepthPlanItem>,
    pub rows_with_no_gaps: usize,
    pub rows_missing_outcome: usize,
    pub rows_missing_baseline: usize,
    pub rows_missing_no_trade: usize,
    pub rows_missing_risk_denied: usize,
    pub rows_missing_candles: usize,
    pub rows_not_eligible: usize,
    pub buildable_gap_count: usize,
    pub unavailable_gap_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CounterfactualDepthPlan {
    pub fn from_bundle(
        config: &ComparableCommitteeEvidenceConfig,
        bundle: &ComparableCommitteeEvidenceBundle,
    ) -> Self {
        let mut items = Vec::new();
        let mut rows_with_no_gaps = 0usize;
        let mut rows_missing_outcome = 0usize;
        let mut rows_missing_baseline = 0usize;
        let mut rows_missing_no_trade = 0usize;
        let mut rows_missing_risk_denied = 0usize;
        let mut rows_missing_candles = 0usize;
        let mut rows_not_eligible = 0usize;

        for row in &bundle.rows {
            let row_items = gap_items_for_row(config, row);
            if row_items.is_empty() {
                rows_with_no_gaps += 1;
            }
            for item in &row_items {
                match item.gap_kind {
                    CounterfactualGapKind::MissingOutcomeReference => rows_missing_outcome += 1,
                    CounterfactualGapKind::MissingBaselineReference => rows_missing_baseline += 1,
                    CounterfactualGapKind::MissingNoTradeCounterfactual => {
                        rows_missing_no_trade += 1
                    }
                    CounterfactualGapKind::MissingRiskDeniedCounterfactual => {
                        rows_missing_risk_denied += 1
                    }
                    CounterfactualGapKind::MissingLocalCandles => rows_missing_candles += 1,
                    CounterfactualGapKind::SourceNotEligible => rows_not_eligible += 1,
                    _ => {}
                }
            }
            items.extend(row_items);
        }

        items.sort_by(|left, right| {
            left.row_id
                .cmp(&right.row_id)
                .then(left.gap_kind.cmp(&right.gap_kind))
                .then(left.required_artifact.cmp(&right.required_artifact))
        });
        let buildable_gap_count = items
            .iter()
            .filter(|item| item.can_build_from_available_data)
            .count();
        let unavailable_gap_count = items.len().saturating_sub(buildable_gap_count);
        Self {
            plan_id: format!("{}-counterfactual-depth", bundle.comparable_id),
            items,
            rows_with_no_gaps,
            rows_missing_outcome,
            rows_missing_baseline,
            rows_missing_no_trade,
            rows_missing_risk_denied,
            rows_missing_candles,
            rows_not_eligible,
            buildable_gap_count,
            unavailable_gap_count,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::MinimumEvidencePlanBuilt,
                        ReasonCode::EvidenceGapDetected,
                    ])
                    .collect::<Vec<_>>(),
            ),
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("plan_id={}", self.plan_id),
            format!("item_count={}", self.items.len()),
            format!("rows_with_no_gaps={}", self.rows_with_no_gaps),
            format!("rows_missing_outcome={}", self.rows_missing_outcome),
            format!("rows_missing_baseline={}", self.rows_missing_baseline),
            format!("rows_missing_no_trade={}", self.rows_missing_no_trade),
            format!("rows_missing_risk_denied={}", self.rows_missing_risk_denied),
            format!("rows_missing_candles={}", self.rows_missing_candles),
            format!("rows_not_eligible={}", self.rows_not_eligible),
            format!("buildable_gap_count={}", self.buildable_gap_count),
            format!("unavailable_gap_count={}", self.unavailable_gap_count),
        ];
        lines.extend(self.items.iter().map(|item| {
            format!(
                "row_id={};gap_kind={:?};required_artifact={};can_build={};suggested_builder={:?};operator_action={}",
                item.row_id,
                item.gap_kind,
                item.required_artifact,
                item.can_build_from_available_data,
                item.suggested_builder,
                item.operator_action.clone().unwrap_or_default(),
            )
        }));
        lines.join("\n")
    }
}

fn gap_items_for_row(
    config: &ComparableCommitteeEvidenceConfig,
    row: &ComparableCommitteeEvidenceRow,
) -> Vec<CounterfactualDepthPlanItem> {
    let mut items = Vec::new();
    if !row.no_lookahead_safe {
        items.push(make_item(
            row,
            CounterfactualGapKind::NoLookaheadViolation,
            "no_lookahead_safe_row",
            false,
            CounterfactualSuggestedBuilder::ManualOperatorAction,
            Some("PreserveNoLookaheadGuard"),
            &[ReasonCode::RejectedNoLookaheadReference],
        ));
        return items;
    }

    if matches!(
        row.source_class,
        ComparableEvidenceSourceClass::YFinanceResearch
            | ComparableEvidenceSourceClass::FixtureArchitectureTest
            | ComparableEvidenceSourceClass::SyntheticTest
    ) {
        items.push(make_item(
            row,
            CounterfactualGapKind::SourceNotEligible,
            "eligible_official_or_controlled_source",
            false,
            CounterfactualSuggestedBuilder::ManualOperatorAction,
            Some("KeepSourceBoundary"),
            &[ReasonCode::ReadinessEvidenceExcluded],
        ));
        return items;
    }

    if row.summary_derived {
        items.push(make_item(
            row,
            CounterfactualGapKind::SummaryDerivedOnly,
            "row_level_materialization",
            false,
            CounterfactualSuggestedBuilder::ManualOperatorAction,
            Some("MaterializeRowLevelComparableEvidence"),
            &[ReasonCode::SummaryDerived],
        ));
    }

    let missing_candles = row.reason_codes.iter().any(|reason| {
        matches!(
            reason,
            ReasonCode::MissingRealLocalData | ReasonCode::MissingOfficialCandles
        )
    });
    let missing_timestamp_alignment = row
        .reason_codes
        .iter()
        .any(|reason| matches!(reason, ReasonCode::StaleTimestamp));
    let missing_future_window = row
        .reason_codes
        .iter()
        .any(|reason| matches!(reason, ReasonCode::InsufficientBars));

    if config.require_outcome_reference && !row.outcome_reference_available {
        let can_build = !missing_candles && !missing_timestamp_alignment && !missing_future_window;
        items.push(make_item(
            row,
            CounterfactualGapKind::MissingOutcomeReference,
            "outcome_reference",
            can_build,
            if can_build {
                CounterfactualSuggestedBuilder::CommitteeOutcomeLinker
            } else if missing_candles {
                CounterfactualSuggestedBuilder::OfficialEvidenceReplicationRunner
            } else {
                CounterfactualSuggestedBuilder::Unavailable
            },
            if missing_candles {
                Some("ProvideOfficialCandleSeries")
            } else if missing_timestamp_alignment {
                Some("RepairTimestampAlignment")
            } else if missing_future_window {
                Some("ProvideLongerFutureWindow")
            } else {
                None
            },
            &[ReasonCode::CommitteeOutcomeReferenceBuilt],
        ));
    }

    if config.require_baseline_reference && !row.baseline_reference_available {
        let can_build = !missing_candles && !missing_timestamp_alignment;
        items.push(make_item(
            row,
            CounterfactualGapKind::MissingBaselineReference,
            "baseline_reference",
            can_build,
            if can_build {
                CounterfactualSuggestedBuilder::CommitteeReferencePackRunner
            } else {
                CounterfactualSuggestedBuilder::Unavailable
            },
            if missing_candles {
                Some("ProvideOfficialCandleSeries")
            } else if missing_timestamp_alignment {
                Some("RepairTimestampAlignment")
            } else {
                Some("GenerateBaselineReference")
            },
            &[ReasonCode::BaselineSignalNoTradeBias],
        ));
    }

    if config.require_no_trade_counterfactual && !row.no_trade_counterfactual_available {
        let can_build = !missing_candles && row.outcome_reference_available;
        items.push(make_item(
            row,
            CounterfactualGapKind::MissingNoTradeCounterfactual,
            "no_trade_counterfactual",
            can_build,
            if can_build {
                CounterfactualSuggestedBuilder::CommitteeCounterfactualAuditRunner
            } else if missing_candles {
                CounterfactualSuggestedBuilder::OfficialEvidenceReplicationRunner
            } else {
                CounterfactualSuggestedBuilder::Unavailable
            },
            if missing_candles {
                Some("ProvideOfficialCandleSeries")
            } else if !row.outcome_reference_available {
                Some("BuildOutcomeReferenceFirst")
            } else {
                Some("GenerateNoTradeCounterfactual")
            },
            &[ReasonCode::NoTradeCounterfactual],
        ));
    }

    if config.require_risk_denied_counterfactual && !row.risk_denied_counterfactual_available {
        let can_build = !missing_candles && row.outcome_reference_available;
        items.push(make_item(
            row,
            CounterfactualGapKind::MissingRiskDeniedCounterfactual,
            "risk_denied_counterfactual",
            can_build,
            if can_build {
                CounterfactualSuggestedBuilder::CommitteeCounterfactualAuditRunner
            } else if missing_candles {
                CounterfactualSuggestedBuilder::OfficialEvidenceReplicationRunner
            } else {
                CounterfactualSuggestedBuilder::Unavailable
            },
            if missing_candles {
                Some("ProvideOfficialCandleSeries")
            } else if !row.outcome_reference_available {
                Some("BuildOutcomeReferenceFirst")
            } else {
                Some("GenerateRiskDeniedCounterfactual")
            },
            &[ReasonCode::RiskDeniedCounterfactual],
        ));
    }

    if missing_candles {
        items.push(make_item(
            row,
            CounterfactualGapKind::MissingLocalCandles,
            "local_candle_series",
            matches!(
                row.source_class,
                ComparableEvidenceSourceClass::OfficialNonCrypto
                    | ComparableEvidenceSourceClass::ControlledDiagnostic
                    | ComparableEvidenceSourceClass::OfficialCryptoOnly
            ),
            CounterfactualSuggestedBuilder::OfficialEvidenceReplicationRunner,
            Some("ProvideOfficialCandleSeries"),
            &[ReasonCode::MissingOfficialCandles],
        ));
    }
    if missing_timestamp_alignment {
        items.push(make_item(
            row,
            CounterfactualGapKind::MissingTimestampAlignment,
            "timestamp_alignment",
            false,
            CounterfactualSuggestedBuilder::ManualOperatorAction,
            Some("RepairTimestampAlignment"),
            &[ReasonCode::StaleTimestamp],
        ));
    }
    if missing_future_window {
        items.push(make_item(
            row,
            CounterfactualGapKind::MissingFutureWindow,
            "future_window",
            false,
            CounterfactualSuggestedBuilder::ManualOperatorAction,
            Some("ProvideLongerFutureWindow"),
            &[ReasonCode::InsufficientBars],
        ));
    }
    items
}

fn make_item(
    row: &ComparableCommitteeEvidenceRow,
    gap_kind: CounterfactualGapKind,
    required_artifact: &str,
    can_build_from_available_data: bool,
    suggested_builder: CounterfactualSuggestedBuilder,
    operator_action: Option<&str>,
    reason_codes: &[ReasonCode],
) -> CounterfactualDepthPlanItem {
    CounterfactualDepthPlanItem {
        row_id: row.row_id.clone(),
        gap_kind,
        required_artifact: required_artifact.to_string(),
        can_build_from_available_data,
        suggested_builder,
        operator_action: operator_action.map(str::to_string),
        reason_codes: stable_reason_codes(reason_codes),
    }
}
