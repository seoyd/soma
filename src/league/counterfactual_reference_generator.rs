use serde::{Deserialize, Serialize};

use crate::CandleSeries;
use crate::core::{ReasonCode, stable_reason_codes};

use super::candle_alignment::{CandleAlignmentRecord, CandleAlignmentStatus};
use super::committee_counterfactual_builder::{
    CommitteeCounterfactualRecord, CommitteeCounterfactualType, CounterfactualBuildStatus,
};
use super::committee_outcome_reference::CommitteeTripleBarrierLabel;
use super::committee_scenario_loader::CommitteeScenarioRow;
use super::triple_barrier_reference_builder::{
    TripleBarrierReferenceBuildResult, TripleBarrierReferenceBuilder, TripleBarrierReferenceConfig,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CounterfactualReferencePolicy {
    #[serde(default = "default_true")]
    pub build_no_trade: bool,
    #[serde(default = "default_true")]
    pub build_risk_denied: bool,
    #[serde(default = "default_credit_avoided_loss_factor")]
    pub credit_avoided_loss_factor: f64,
    #[serde(default = "default_penalize_missed_gain_factor")]
    pub penalize_missed_gain_factor: f64,
    #[serde(default = "default_max_missed_gain_penalty")]
    pub max_missed_gain_penalty: f64,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CounterfactualReferenceGenerator;

impl Default for CounterfactualReferencePolicy {
    fn default() -> Self {
        Self {
            build_no_trade: true,
            build_risk_denied: true,
            credit_avoided_loss_factor: default_credit_avoided_loss_factor(),
            penalize_missed_gain_factor: default_penalize_missed_gain_factor(),
            max_missed_gain_penalty: default_max_missed_gain_penalty(),
            reason_codes: vec![ReasonCode::CommitteeCounterfactualBuilderBuilt],
        }
    }
}

impl CounterfactualReferenceGenerator {
    pub fn generate_no_trade(
        &self,
        row: &CommitteeScenarioRow,
        alignment: &CandleAlignmentRecord,
        series: &CandleSeries,
        config: &TripleBarrierReferenceConfig,
        policy: &CounterfactualReferencePolicy,
        diagnostic_only: bool,
    ) -> CommitteeCounterfactualRecord {
        self.generate(
            row,
            alignment,
            series,
            config,
            policy,
            diagnostic_only,
            CommitteeCounterfactualType::NoTrade,
        )
    }

    pub fn generate_risk_denied(
        &self,
        row: &CommitteeScenarioRow,
        alignment: &CandleAlignmentRecord,
        series: &CandleSeries,
        config: &TripleBarrierReferenceConfig,
        policy: &CounterfactualReferencePolicy,
        diagnostic_only: bool,
    ) -> CommitteeCounterfactualRecord {
        self.generate(
            row,
            alignment,
            series,
            config,
            policy,
            diagnostic_only,
            CommitteeCounterfactualType::RiskDenied,
        )
    }

    fn generate(
        &self,
        row: &CommitteeScenarioRow,
        alignment: &CandleAlignmentRecord,
        series: &CandleSeries,
        config: &TripleBarrierReferenceConfig,
        policy: &CounterfactualReferencePolicy,
        diagnostic_only: bool,
        counterfactual_type: CommitteeCounterfactualType,
    ) -> CommitteeCounterfactualRecord {
        if !matches!(
            alignment.status,
            CandleAlignmentStatus::MatchedExact | CandleAlignmentStatus::MatchedWithTolerance
        ) {
            return unavailable_record(
                row,
                config,
                counterfactual_type,
                map_alignment_status(alignment.status),
            );
        }
        if !alignment.no_lookahead_safe {
            return unavailable_record(
                row,
                config,
                counterfactual_type,
                CounterfactualBuildStatus::RejectedNoLookahead,
            );
        }
        let mut built = match TripleBarrierReferenceBuilder::default().build(
            row,
            alignment,
            series,
            config,
            diagnostic_only || !row.evidence_source_kind.readiness_eligible(),
        ) {
            Ok(result) => result,
            Err(_) => {
                return unavailable_record(
                    row,
                    config,
                    counterfactual_type,
                    CounterfactualBuildStatus::UnavailableWrongHorizon,
                );
            }
        };
        built.reference.triple_barrier_label = match counterfactual_type {
            CommitteeCounterfactualType::NoTrade => {
                CommitteeTripleBarrierLabel::NoTradeCounterfactual
            }
            CommitteeCounterfactualType::RiskDenied => {
                CommitteeTripleBarrierLabel::RiskDeniedCounterfactual
            }
            CommitteeCounterfactualType::BaselineAction => built.reference.triple_barrier_label,
            CommitteeCounterfactualType::ExternalAction => built.reference.triple_barrier_label,
        };
        record_from_build(row, built, policy, counterfactual_type)
    }
}

fn record_from_build(
    row: &CommitteeScenarioRow,
    built: TripleBarrierReferenceBuildResult,
    policy: &CounterfactualReferencePolicy,
    counterfactual_type: CommitteeCounterfactualType,
) -> CommitteeCounterfactualRecord {
    let outcome_reference = built.reference;
    let net_return_pct = outcome_reference.net_return_pct.unwrap_or_default();
    let avoided_loss_value = if net_return_pct < 0.0 {
        Some((net_return_pct.abs() * policy.credit_avoided_loss_factor).max(0.0))
    } else {
        None
    };
    let missed_gain_value = if net_return_pct > 0.0 {
        Some(
            (net_return_pct * policy.penalize_missed_gain_factor)
                .min(policy.max_missed_gain_penalty)
                .max(0.0),
        )
    } else {
        None
    };
    let mut reason_codes = policy.reason_codes.clone();
    reason_codes.extend(outcome_reference.reason_codes.clone());
    reason_codes.push(ReasonCode::CommitteeCounterfactualBuilt);
    reason_codes.push(ReasonCode::CounterfactualEvaluated);
    reason_codes.push(match counterfactual_type {
        CommitteeCounterfactualType::NoTrade => ReasonCode::NoTradeCounterfactual,
        CommitteeCounterfactualType::RiskDenied => ReasonCode::RiskDeniedCounterfactual,
        CommitteeCounterfactualType::BaselineAction => ReasonCode::CounterfactualEvaluated,
        CommitteeCounterfactualType::ExternalAction => ReasonCode::CounterfactualEvaluated,
    });
    if avoided_loss_value.is_some() {
        reason_codes.push(ReasonCode::AvoidedLossRecorded);
    }
    if missed_gain_value.is_some() {
        reason_codes.push(ReasonCode::MissedGainRecorded);
        reason_codes.push(ReasonCode::OpportunityCostRecorded);
    }
    if counterfactual_type == CommitteeCounterfactualType::RiskDenied {
        reason_codes.push(ReasonCode::DefensiveAttribution);
        reason_codes.push(ReasonCode::RiskDenied);
    }
    let build_status = if built.diagnostic_only {
        CounterfactualBuildStatus::EstimatedDiagnosticOnly
    } else {
        CounterfactualBuildStatus::Built
    };
    CommitteeCounterfactualRecord {
        counterfactual_id: format!("{}-{:?}", row.scenario_row_id, counterfactual_type),
        scenario_row_id: row.scenario_row_id.clone(),
        counterfactual_type,
        build_status,
        triple_barrier_label: Some(outcome_reference.triple_barrier_label),
        net_return_pct: outcome_reference.net_return_pct,
        avoided_loss_value,
        missed_gain_value,
        max_favorable_excursion_pct: outcome_reference.max_favorable_excursion_pct,
        max_adverse_excursion_pct: outcome_reference.max_adverse_excursion_pct,
        cost_bps: outcome_reference.cost_bps,
        slippage_bps: outcome_reference.slippage_bps,
        no_lookahead_safe: outcome_reference.no_lookahead_safe,
        diagnostic_only: built.diagnostic_only,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn unavailable_record(
    row: &CommitteeScenarioRow,
    config: &TripleBarrierReferenceConfig,
    counterfactual_type: CommitteeCounterfactualType,
    build_status: CounterfactualBuildStatus,
) -> CommitteeCounterfactualRecord {
    let mut reason_codes = vec![ReasonCode::CommitteeCounterfactualUnavailable];
    if build_status == CounterfactualBuildStatus::RejectedNoLookahead {
        reason_codes.push(ReasonCode::RejectedNoLookaheadReference);
    }
    if build_status == CounterfactualBuildStatus::RejectedBadDataQuality {
        reason_codes.push(ReasonCode::DataQualityTooLow);
    }
    CommitteeCounterfactualRecord {
        counterfactual_id: format!("{}-{:?}", row.scenario_row_id, counterfactual_type),
        scenario_row_id: row.scenario_row_id.clone(),
        counterfactual_type,
        build_status,
        triple_barrier_label: None,
        net_return_pct: None,
        avoided_loss_value: None,
        missed_gain_value: None,
        max_favorable_excursion_pct: None,
        max_adverse_excursion_pct: None,
        cost_bps: config.cost_bps,
        slippage_bps: config.slippage_bps,
        no_lookahead_safe: false,
        diagnostic_only: false,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn map_alignment_status(status: CandleAlignmentStatus) -> CounterfactualBuildStatus {
    match status {
        CandleAlignmentStatus::MatchedExact | CandleAlignmentStatus::MatchedWithTolerance => {
            CounterfactualBuildStatus::Built
        }
        CandleAlignmentStatus::MissingCandleSeries | CandleAlignmentStatus::WrongSymbol => {
            CounterfactualBuildStatus::UnavailableNoCandleData
        }
        CandleAlignmentStatus::MissingTimestamp => {
            CounterfactualBuildStatus::UnavailableNoTimestampMatch
        }
        CandleAlignmentStatus::WrongHorizon | CandleAlignmentStatus::InsufficientFutureBars => {
            CounterfactualBuildStatus::UnavailableWrongHorizon
        }
        CandleAlignmentStatus::BadDataQuality
        | CandleAlignmentStatus::GapDetected
        | CandleAlignmentStatus::DuplicateTimestamp => {
            CounterfactualBuildStatus::RejectedBadDataQuality
        }
        CandleAlignmentStatus::RejectedNoLookahead => {
            CounterfactualBuildStatus::RejectedNoLookahead
        }
        CandleAlignmentStatus::Unknown => CounterfactualBuildStatus::UnavailableNoTimestampMatch,
    }
}

fn default_true() -> bool {
    true
}

fn default_credit_avoided_loss_factor() -> f64 {
    0.50
}

fn default_penalize_missed_gain_factor() -> f64 {
    0.25
}

fn default_max_missed_gain_penalty() -> f64 {
    0.05
}
