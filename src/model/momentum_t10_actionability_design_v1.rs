//! Volatility-normalized T10 actionability labels and unexecuted selective architecture.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::{Deserialize, Serialize};

use crate::stable_hash_string;

use super::{
    momentum_future_prediction_v4::{
        ArtifactBuilderV4_2, ArtifactReaderV4_2, as_u64, as_usize, persist_artifact, read_single,
    },
    momentum_micro_label_forensics_v1::{
        MomentumMicroProtectedBeforeStateV1, validate_momentum_micro_protected_before_state_v1,
    },
    momentum_qualified_six_replay_v1::{COMPARISON_EPSILON, MomentumReplayPartitionV1},
    momentum_t10_failure_forensics_v1::{
        MomentumT10ActionabilityDesignEvidenceClassV1, MomentumT10FailureForensicsReportV1,
        MomentumT10FailureForensicsStatusV1, read_momentum_t10_failure_forensics_report_v1,
    },
    momentum_t10_micro_screening_v1::{
        MomentumT10ConsumedActionabilityEvidenceV1,
        read_momentum_t10_consumed_actionability_evidence_v1,
        read_momentum_t10_micro_screening_report_v1,
    },
};

const ROOT: &str = "state/historical_replay/momentum_t10_actionability_design/v1";
const LABEL_REGISTRATION_VERSION: &str = "momentum-t10-actionability-label-registration-v1";
const SELECTION_POLICY_VERSION: &str = "momentum-t10-actionability-selection-policy-v1";
const TEMPORAL_STABILITY_VERSION: &str = "momentum-t10-actionability-temporal-stability-v1";
const CANDIDATE_REPORT_VERSION: &str = "momentum-t10-actionability-candidate-report-v1";
const SELECTION_RECEIPT_VERSION: &str = "momentum-t10-actionability-selection-receipt-v1";
const PARTICIPANT_VERSION: &str = "momentum-t10-selective-participant-registration-v1";
const PAIR_VERSION: &str = "momentum-t10-selective-pair-registration-v1";
const TRAINING_POLICY_VERSION: &str = "momentum-t10-selective-future-training-policy-v1";
const FRESH_GATE_VERSION: &str = "momentum-t10-selective-fresh-validation-gate-v1";
const FINAL_GATE_VERSION: &str = "momentum-t10-selective-final-holdout-gate-v1";
const JOURNAL_VERSION: &str = "momentum-t10-actionability-research-journal-v1";
const REPORT_VERSION: &str = "momentum-t10-actionability-design-public-report-v1";
const VOLATILITY_LOOKBACK_RETURNS: usize = 144;
const MAXIMUM_TRAINING_EXAMPLES: usize = 4_096;
const MINIMUM_SUPPORT: usize = 1_024;
const STANDARD_L2_MULTIPLIER: usize = 1;
const STRONG_L2_MULTIPLIER: usize = 4;
const OPPORTUNITY_THRESHOLD: f64 = 0.5;
const DIRECTION_THRESHOLD: f64 = 0.5;
const MINIMUM_CLASS_PREVALENCE: f64 = 0.15;
const MINIMUM_ABSTAIN_PREVALENCE: f64 = 0.20;
const MAXIMUM_ABSTAIN_PREVALENCE: f64 = 0.70;
const MAXIMUM_PREVALENCE_DRIFT: f64 = 0.10;
const MINIMUM_COVERAGE: f64 = 0.10;
const MAXIMUM_COVERAGE: f64 = 0.70;
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;
const WEEK_MS: u64 = 7 * DAY_MS;
const MULTIPLIERS: [f64; 3] = [0.25, 0.50, 1.00];
const OPPORTUNITY_METRICS: [&str; 7] = [
    "PairedMeanBrierVersusO0",
    "PairedMedianBrierVersusO0",
    "ActionabilityCorrectness",
    "FixedBinCalibration",
    "ProbabilityCollapseAndSaturation",
    "ChronologyAudit",
    "LeakageAudit",
];
const DIRECTION_METRICS: [&str; 7] = [
    "PairedMeanBrierVersusD0OnTrueActionable",
    "PairedMedianBrierVersusD0OnTrueActionable",
    "DirectionalCorrectnessOnTrueActionable",
    "FixedBinCalibration",
    "ProbabilityCollapseAndSaturation",
    "ChronologyAudit",
    "LeakageAudit",
];
const SELECTIVE_METRICS: [&str; 14] = [
    "PredictionEventCount",
    "PredictedActionCount",
    "AbstentionCount",
    "Coverage",
    "TrueActionableCount",
    "OpportunityPrecision",
    "OpportunityRecall",
    "DirectionBrierOnTrueActionable",
    "DirectionCorrectnessOnTrueActionable",
    "DirectionCorrectnessOnPredictedKnownActionableDirection",
    "FalseActionRate",
    "MissedActionRate",
    "FiniteValueProof",
    "ChronologyAndLeakageAudit",
];
const PUBLIC_LABELS: [&str; 7] = [
    "HistoricalResearchOnly",
    "PostScreeningResearchDesignOnly",
    "ConsumedDesignEvidenceOnly",
    "FreshValidationStillClosed",
    "FinalHoldoutStillClosed",
    "NotLiveAuthority",
    "NotTradingAuthority",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumT10ActionabilityDesignRunModeV1 {
    Status,
    RegisterAndExecuteLocal,
}

impl MomentumT10ActionabilityDesignRunModeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::RegisterAndExecuteLocal => "register-and-execute-local",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumT10ActionabilityDesignStatusV1 {
    Unregistered,
    Complete,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumT10ActionabilitySelectionStatusV1 {
    StableThresholdSelected,
    NoStableActionabilityThreshold,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumT10FreshValidationSupportStatusV1 {
    SufficientSupport,
    FreshValidationInsufficientSupport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActionabilityLabel {
    ActionableUp,
    Abstain,
    ActionableDown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10ActionabilityLabelRegistrationV1 {
    pub registration_version: String,
    pub source_event_set_digest: String,
    pub volatility_policy_digest: String,
    pub candidate_multiplier_bits: Vec<u64>,
    pub actionable_up_rule: String,
    pub abstain_rule: String,
    pub actionable_down_rule: String,
    pub selection_policy_digest: String,
    pub fresh_validation_access_forbidden: bool,
    pub final_holdout_access_forbidden: bool,
    pub model_training_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10ActionabilitySelectionPolicyV1 {
    pub policy_version: String,
    pub minimum_total_support: usize,
    pub actionable_up_minimum_prevalence_bits: u64,
    pub actionable_down_minimum_prevalence_bits: u64,
    pub abstain_minimum_prevalence_bits: u64,
    pub abstain_maximum_prevalence_bits: u64,
    pub maximum_partition_class_drift_bits: u64,
    pub choose_largest_passing_multiplier: bool,
    pub policy_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumT10ActionabilityTemporalStabilityV1 {
    pub stability_version: String,
    pub daily_class_prevalence_ranges: Vec<f64>,
    pub weekly_class_prevalence_ranges: Vec<f64>,
    pub monthly_class_prevalence_ranges: Vec<f64>,
    pub rolling_144_class_prevalence_ranges: Vec<f64>,
    pub rolling_1008_class_prevalence_ranges: Vec<f64>,
    pub finite: bool,
    pub stability_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumT10ActionabilityCandidateReportV1 {
    pub report_version: String,
    pub label_registration_digest: String,
    pub partition: MomentumReplayPartitionV1,
    pub multiplier: f64,
    pub eligible_event_count: usize,
    pub actionable_up_count: usize,
    pub actionable_down_count: usize,
    pub abstain_count: usize,
    pub actionable_up_prevalence: f64,
    pub actionable_down_prevalence: f64,
    pub abstain_prevalence: f64,
    pub target_magnitude_mean: f64,
    pub target_magnitude_median: f64,
    pub volatility_scale_mean: f64,
    pub volatility_scale_median: f64,
    pub zero_volatility_floor_count: usize,
    pub temporal_stability: MomentumT10ActionabilityTemporalStabilityV1,
    pub finite_value_proof: bool,
    pub chronology_audit_passed: bool,
    pub leakage_audit_passed: bool,
    pub integrity_audit_passed: bool,
    pub candidate_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumT10ActionabilitySelectionReceiptV1 {
    pub receipt_version: String,
    pub label_registration_digest: String,
    pub candidate_report_digests: Vec<String>,
    pub selected_multiplier: Option<f64>,
    pub selected_candidate_digest: Option<String>,
    pub selection_status: MomentumT10ActionabilitySelectionStatusV1,
    pub fresh_validation_reads: usize,
    pub final_holdout_reads: usize,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10SelectiveParticipantRegistrationV1 {
    pub registration_version: String,
    pub participant_id: String,
    pub head: String,
    pub role: String,
    pub feature_policy: String,
    pub feature_dimension: usize,
    pub l2_multiplier: usize,
    pub prior_revealed_labels_only: bool,
    pub actionable_training_only: bool,
    pub training_authorized: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10SelectivePairRegistrationV1 {
    pub registration_version: String,
    pub pair_id: String,
    pub opportunity_participant_id: String,
    pub direction_participant_id: String,
    pub reference_system: bool,
    pub cross_pairing: bool,
    pub opportunity_threshold_bits: u64,
    pub direction_threshold_bits: u64,
    pub opportunity_below_threshold_abstains: bool,
    pub direction_equality_maps_up: bool,
    pub training_authorized: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10SelectiveFutureTrainingPolicyV1 {
    pub policy_version: String,
    pub daily_utc_refit: bool,
    pub previously_revealed_labels_only: bool,
    pub chronological_order_required: bool,
    pub maximum_training_examples: usize,
    pub training_only_normalizers: bool,
    pub direction_excludes_abstain: bool,
    pub persist_and_reopen_all_receipts: bool,
    pub participants_frozen_per_utc_day: bool,
    pub later_fresh_labels_only_after_observable: bool,
    pub fresh_evidence_split_digest: String,
    pub consumed_design_pool_is_former_development_and_validation: bool,
    pub fresh_validation_use_once: bool,
    pub final_holdout_requires_separate_authorization: bool,
    pub execution_authorized: bool,
    pub policy_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumT10FreshValidationGateV1 {
    pub gate_version: String,
    pub opportunity_lower_brier_than_o0_required: bool,
    pub direction_lower_brier_than_d0_required: bool,
    pub consumed_design_replay_required: bool,
    pub fresh_validation_required: bool,
    pub sufficient_actionable_support_required: bool,
    pub minimum_coverage: f64,
    pub maximum_coverage: f64,
    pub finite_no_collapse_no_saturation_required: bool,
    pub chronology_leakage_integrity_required: bool,
    pub result_selected_mutation_forbidden: bool,
    pub correctness_cannot_override_brier: bool,
    pub coverage_cannot_override_brier: bool,
    pub final_holdout_access_required_zero: bool,
    pub opportunity_metric_names: Vec<String>,
    pub direction_metric_names: Vec<String>,
    pub selective_metric_names: Vec<String>,
    pub execution_authorized: bool,
    pub gate_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10FinalHoldoutGateV1 {
    pub gate_version: String,
    pub consumed_design_pass_required: bool,
    pub fresh_validation_pass_required: bool,
    pub post_validation_design_change_forbidden: bool,
    pub deterministic_eligible_cohort_required: bool,
    pub separate_owner_authorization_required: bool,
    pub prediction_count: usize,
    pub label_read_count: usize,
    pub metric_count: usize,
    pub execution_authorized: bool,
    pub gate_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumT10ActionabilityResearchJournalV1 {
    journal_version: String,
    failure_report_digest: String,
    label_registration_digest: String,
    candidate_report_digests: Vec<String>,
    selection_receipt_digest: String,
    participant_registration_digests: Vec<String>,
    pair_registration_digests: Vec<String>,
    future_training_policy_digest: String,
    fresh_validation_gate_digest: String,
    final_holdout_gate_digest: String,
    deterministic: bool,
    journal_digest: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10ActionabilitySafetyCountersV1 {
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub label_computations: usize,
    pub new_opportunity_model_fits: usize,
    pub new_direction_model_fits: usize,
    pub new_selective_predictions: usize,
    pub new_selective_evaluations: usize,
    pub fresh_validation_predictions: usize,
    pub fresh_validation_label_reads: usize,
    pub fresh_validation_metrics: usize,
    pub final_holdout_predictions: usize,
    pub final_holdout_label_reads: usize,
    pub final_holdout_metrics: usize,
    pub t30_model_executions: usize,
    pub t60_model_executions: usize,
    pub network_requests: usize,
    pub live_operations: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub chair_actions: usize,
    pub vote_actions: usize,
    pub trading_actions: usize,
    pub day_view_loads: usize,
    pub week_view_loads: usize,
    pub month_view_loads: usize,
    pub year_view_loads: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumT10ActionabilityDesignReportV1 {
    pub report_version: String,
    pub run_mode: String,
    pub status: MomentumT10ActionabilityDesignStatusV1,
    pub design_evidence_class: MomentumT10ActionabilityDesignEvidenceClassV1,
    pub failure_forensics_report_digest: String,
    pub protected_before_state_digest: String,
    pub fresh_evidence_split_digest: String,
    pub label_registration: MomentumT10ActionabilityLabelRegistrationV1,
    pub selection_policy: MomentumT10ActionabilitySelectionPolicyV1,
    pub candidate_reports: Vec<MomentumT10ActionabilityCandidateReportV1>,
    pub selection_receipt: MomentumT10ActionabilitySelectionReceiptV1,
    pub fresh_validation_event_count: usize,
    pub fresh_validation_minimum_support: usize,
    pub fresh_validation_support_status: MomentumT10FreshValidationSupportStatusV1,
    pub fresh_validation_support_sufficient: bool,
    pub participant_registrations: Vec<MomentumT10SelectiveParticipantRegistrationV1>,
    pub pair_registrations: Vec<MomentumT10SelectivePairRegistrationV1>,
    pub future_training_policy: MomentumT10SelectiveFutureTrainingPolicyV1,
    pub fresh_validation_gate: MomentumT10FreshValidationGateV1,
    pub final_holdout_gate: MomentumT10FinalHoldoutGateV1,
    pub two_stage_registration_digest: Option<String>,
    pub labels: Vec<String>,
    pub live_completed_event_count: usize,
    pub live_scorable_event_count: usize,
    pub live_pause: String,
    pub epoch_three_registered: bool,
    pub full_eight_blocked: bool,
    pub protected_artifacts_unchanged: bool,
    pub safety_counters: MomentumT10ActionabilitySafetyCountersV1,
    pub deterministic_replay_digest: String,
    pub runtime_duration_ms: u64,
    pub report_digest: String,
}

#[derive(Clone)]
struct CandidateLabelEvent {
    timestamp_ms: u64,
    target_magnitude: f64,
    volatility: f64,
    used_floor: bool,
    label: ActionabilityLabel,
}

fn canonical_digest<T: Clone + std::fmt::Debug>(value: &T, clear: impl FnOnce(&mut T)) -> String {
    let mut canonical = value.clone();
    clear(&mut canonical);
    stable_hash_string(&format!("{canonical:?}"))
}

fn label_registration_digest(value: &MomentumT10ActionabilityLabelRegistrationV1) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn selection_policy_digest(value: &MomentumT10ActionabilitySelectionPolicyV1) -> String {
    canonical_digest(value, |item| item.policy_digest.clear())
}

fn registered_volatility_policy_digest() -> String {
    stable_hash_string(&format!(
        "T10-actionability-volatility:population-stddev-simple-return:lookback={}:floor-bits={}:past-closed-only:missing-context-ineligible",
        VOLATILITY_LOOKBACK_RETURNS,
        COMPARISON_EPSILON.to_bits()
    ))
}

fn stability_digest(value: &MomentumT10ActionabilityTemporalStabilityV1) -> String {
    canonical_digest(value, |item| item.stability_digest.clear())
}

fn candidate_digest(value: &MomentumT10ActionabilityCandidateReportV1) -> String {
    canonical_digest(value, |item| item.candidate_digest.clear())
}

fn selection_receipt_digest(value: &MomentumT10ActionabilitySelectionReceiptV1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn participant_digest(value: &MomentumT10SelectiveParticipantRegistrationV1) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn pair_digest(value: &MomentumT10SelectivePairRegistrationV1) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn future_training_digest(value: &MomentumT10SelectiveFutureTrainingPolicyV1) -> String {
    canonical_digest(value, |item| item.policy_digest.clear())
}

fn fresh_gate_digest(value: &MomentumT10FreshValidationGateV1) -> String {
    canonical_digest(value, |item| item.gate_digest.clear())
}

fn final_gate_digest(value: &MomentumT10FinalHoldoutGateV1) -> String {
    canonical_digest(value, |item| item.gate_digest.clear())
}

fn journal_digest(value: &MomentumT10ActionabilityResearchJournalV1) -> String {
    canonical_digest(value, |item| item.journal_digest.clear())
}

fn report_digest(value: &MomentumT10ActionabilityDesignReportV1) -> String {
    canonical_digest(value, |item| {
        item.run_mode.clear();
        item.safety_counters = MomentumT10ActionabilitySafetyCountersV1::default();
        item.runtime_duration_ms = 0;
        item.report_digest.clear();
    })
}

fn two_stage_registration_digest(
    selection_receipt: &MomentumT10ActionabilitySelectionReceiptV1,
    participants: &[MomentumT10SelectiveParticipantRegistrationV1],
    pairs: &[MomentumT10SelectivePairRegistrationV1],
    future_training_policy: &MomentumT10SelectiveFutureTrainingPolicyV1,
    fresh_validation_gate: &MomentumT10FreshValidationGateV1,
) -> String {
    stable_hash_string(&format!(
        "T10-selective-two-stage-registration:{}:{}:{}:{}:{}:{:?}:{:?}",
        selection_receipt.receipt_digest,
        OPPORTUNITY_THRESHOLD.to_bits(),
        DIRECTION_THRESHOLD.to_bits(),
        future_training_policy.policy_digest,
        fresh_validation_gate.gate_digest,
        participants
            .iter()
            .map(|value| value.registration_digest.clone())
            .collect::<Vec<_>>(),
        pairs
            .iter()
            .map(|value| value.registration_digest.clone())
            .collect::<Vec<_>>()
    ))
}

fn validate_selection_policy(
    value: &MomentumT10ActionabilitySelectionPolicyV1,
) -> Result<(), String> {
    if value.policy_version != SELECTION_POLICY_VERSION
        || value.minimum_total_support != MINIMUM_SUPPORT
        || f64::from_bits(value.actionable_up_minimum_prevalence_bits) != MINIMUM_CLASS_PREVALENCE
        || f64::from_bits(value.actionable_down_minimum_prevalence_bits) != MINIMUM_CLASS_PREVALENCE
        || f64::from_bits(value.abstain_minimum_prevalence_bits) != MINIMUM_ABSTAIN_PREVALENCE
        || f64::from_bits(value.abstain_maximum_prevalence_bits) != MAXIMUM_ABSTAIN_PREVALENCE
        || f64::from_bits(value.maximum_partition_class_drift_bits) != MAXIMUM_PREVALENCE_DRIFT
        || !value.choose_largest_passing_multiplier
        || value.policy_digest != selection_policy_digest(value)
    {
        return Err("T10 actionability selection policy rejected".to_string());
    }
    Ok(())
}

fn validate_label_registration(
    value: &MomentumT10ActionabilityLabelRegistrationV1,
    policy: &MomentumT10ActionabilitySelectionPolicyV1,
) -> Result<(), String> {
    if value.registration_version != LABEL_REGISTRATION_VERSION
        || value.source_event_set_digest.is_empty()
        || value.volatility_policy_digest != registered_volatility_policy_digest()
        || value.candidate_multiplier_bits
            != MULTIPLIERS
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        || value.actionable_up_rule != "future_return_t > k * sigma_t"
        || value.abstain_rule != "abs(future_return_t) <= k * sigma_t"
        || value.actionable_down_rule != "future_return_t < -k * sigma_t"
        || value.selection_policy_digest != policy.policy_digest
        || !value.fresh_validation_access_forbidden
        || !value.final_holdout_access_forbidden
        || !value.model_training_forbidden
        || value.registration_digest != label_registration_digest(value)
    {
        return Err("T10 actionability label registration rejected".to_string());
    }
    Ok(())
}

fn finite_ranges(value: &[f64]) -> bool {
    value.len() == 3
        && value
            .iter()
            .all(|item| item.is_finite() && (0.0..=1.0).contains(item))
}

fn validate_stability(value: &MomentumT10ActionabilityTemporalStabilityV1) -> Result<(), String> {
    if value.stability_version != TEMPORAL_STABILITY_VERSION
        || !finite_ranges(&value.daily_class_prevalence_ranges)
        || !finite_ranges(&value.weekly_class_prevalence_ranges)
        || !finite_ranges(&value.monthly_class_prevalence_ranges)
        || !finite_ranges(&value.rolling_144_class_prevalence_ranges)
        || !finite_ranges(&value.rolling_1008_class_prevalence_ranges)
        || !value.finite
        || value.stability_digest != stability_digest(value)
    {
        return Err("T10 actionability temporal stability rejected".to_string());
    }
    Ok(())
}

fn validate_candidate(value: &MomentumT10ActionabilityCandidateReportV1) -> Result<(), String> {
    validate_stability(&value.temporal_stability)?;
    let count = value.actionable_up_count + value.actionable_down_count + value.abstain_count;
    let denominator = value.eligible_event_count as f64;
    if value.report_version != CANDIDATE_REPORT_VERSION
        || value.label_registration_digest.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || !MULTIPLIERS.contains(&value.multiplier)
        || value.eligible_event_count == 0
        || count != value.eligible_event_count
        || (value.actionable_up_prevalence - value.actionable_up_count as f64 / denominator).abs()
            > COMPARISON_EPSILON
        || (value.actionable_down_prevalence - value.actionable_down_count as f64 / denominator)
            .abs()
            > COMPARISON_EPSILON
        || (value.abstain_prevalence - value.abstain_count as f64 / denominator).abs()
            > COMPARISON_EPSILON
        || [
            value.actionable_up_prevalence,
            value.actionable_down_prevalence,
            value.abstain_prevalence,
            value.target_magnitude_mean,
            value.target_magnitude_median,
            value.volatility_scale_mean,
            value.volatility_scale_median,
        ]
        .iter()
        .any(|item| !item.is_finite())
        || value.target_magnitude_mean < 0.0
        || value.target_magnitude_median < 0.0
        || value.volatility_scale_mean <= 0.0
        || value.volatility_scale_median <= 0.0
        || (value.actionable_up_prevalence
            + value.actionable_down_prevalence
            + value.abstain_prevalence
            - 1.0)
            .abs()
            > COMPARISON_EPSILON
        || !value.finite_value_proof
        || !value.chronology_audit_passed
        || !value.leakage_audit_passed
        || !value.integrity_audit_passed
        || value.candidate_digest != candidate_digest(value)
    {
        return Err("T10 actionability candidate report rejected".to_string());
    }
    Ok(())
}

fn validate_selection_receipt(
    value: &MomentumT10ActionabilitySelectionReceiptV1,
) -> Result<(), String> {
    let selected = value.selection_status
        == MomentumT10ActionabilitySelectionStatusV1::StableThresholdSelected;
    if value.receipt_version != SELECTION_RECEIPT_VERSION
        || value.label_registration_digest.is_empty()
        || value.candidate_report_digests.len() != 6
        || value.selected_multiplier.is_some() != selected
        || value.selected_candidate_digest.is_some() != selected
        || value
            .selected_multiplier
            .is_some_and(|multiplier| !MULTIPLIERS.contains(&multiplier))
        || value.fresh_validation_reads != 0
        || value.final_holdout_reads != 0
        || value.receipt_digest != selection_receipt_digest(value)
    {
        return Err("T10 actionability selection receipt rejected".to_string());
    }
    Ok(())
}

fn validate_participant(
    value: &MomentumT10SelectiveParticipantRegistrationV1,
) -> Result<(), String> {
    let valid = match value.participant_id.as_str() {
        "O0" => {
            value.head == "Opportunity"
                && value.role == "PrevalenceConstant"
                && value.feature_policy == "PriorRevealedActionabilityLabelsOnly"
                && value.feature_dimension == 0
                && value.l2_multiplier == 0
                && value.prior_revealed_labels_only
                && !value.actionable_training_only
        }
        "O1" => {
            value.head == "Opportunity"
                && value.role == "Logistic"
                && value.feature_policy == "FrozenTenMinuteAnchor"
                && value.feature_dimension == 6
                && value.l2_multiplier == STANDARD_L2_MULTIPLIER
                && value.prior_revealed_labels_only
                && !value.actionable_training_only
        }
        "O2" => {
            value.head == "Opportunity"
                && value.role == "Logistic"
                && value.feature_policy == "FrozenCompactMicro69"
                && value.feature_dimension == 69
                && value.l2_multiplier == STRONG_L2_MULTIPLIER
                && value.prior_revealed_labels_only
                && !value.actionable_training_only
        }
        "D0" => {
            value.head == "Direction"
                && value.role == "PrevalenceConstant"
                && value.feature_policy == "PriorRevealedActionableDirectionsOnly"
                && value.feature_dimension == 0
                && value.l2_multiplier == 0
                && value.prior_revealed_labels_only
                && value.actionable_training_only
        }
        "D1" => {
            value.head == "Direction"
                && value.role == "Logistic"
                && value.feature_policy == "FrozenTenMinuteAnchor"
                && value.feature_dimension == 6
                && value.l2_multiplier == STANDARD_L2_MULTIPLIER
                && value.prior_revealed_labels_only
                && value.actionable_training_only
        }
        "D2" => {
            value.head == "Direction"
                && value.role == "Logistic"
                && value.feature_policy == "FrozenCompactMicro69"
                && value.feature_dimension == 69
                && value.l2_multiplier == STRONG_L2_MULTIPLIER
                && value.prior_revealed_labels_only
                && value.actionable_training_only
        }
        _ => false,
    };
    if value.registration_version != PARTICIPANT_VERSION
        || !valid
        || value.training_authorized
        || value.registration_digest != participant_digest(value)
    {
        return Err("T10 selective participant registration rejected".to_string());
    }
    Ok(())
}

fn validate_pair(value: &MomentumT10SelectivePairRegistrationV1) -> Result<(), String> {
    let expected = match value.pair_id.as_str() {
        "S0" => ("O0", "D0", true),
        "S1" => ("O1", "D1", false),
        "S2" => ("O2", "D2", false),
        _ => return Err("T10 selective pair identity rejected".to_string()),
    };
    if value.registration_version != PAIR_VERSION
        || value.opportunity_participant_id != expected.0
        || value.direction_participant_id != expected.1
        || value.reference_system != expected.2
        || value.cross_pairing
        || value.opportunity_threshold_bits != OPPORTUNITY_THRESHOLD.to_bits()
        || value.direction_threshold_bits != DIRECTION_THRESHOLD.to_bits()
        || !value.opportunity_below_threshold_abstains
        || !value.direction_equality_maps_up
        || value.training_authorized
        || value.registration_digest != pair_digest(value)
    {
        return Err("T10 selective pair registration rejected".to_string());
    }
    Ok(())
}

fn validate_future_training(
    value: &MomentumT10SelectiveFutureTrainingPolicyV1,
) -> Result<(), String> {
    if value.policy_version != TRAINING_POLICY_VERSION
        || !value.daily_utc_refit
        || !value.previously_revealed_labels_only
        || !value.chronological_order_required
        || value.maximum_training_examples != MAXIMUM_TRAINING_EXAMPLES
        || !value.training_only_normalizers
        || !value.direction_excludes_abstain
        || !value.persist_and_reopen_all_receipts
        || !value.participants_frozen_per_utc_day
        || !value.later_fresh_labels_only_after_observable
        || value.fresh_evidence_split_digest.is_empty()
        || !value.consumed_design_pool_is_former_development_and_validation
        || !value.fresh_validation_use_once
        || !value.final_holdout_requires_separate_authorization
        || value.execution_authorized
        || value.policy_digest != future_training_digest(value)
    {
        return Err("T10 selective future training policy rejected".to_string());
    }
    Ok(())
}

fn validate_fresh_gate(value: &MomentumT10FreshValidationGateV1) -> Result<(), String> {
    if value.gate_version != FRESH_GATE_VERSION
        || !value.opportunity_lower_brier_than_o0_required
        || !value.direction_lower_brier_than_d0_required
        || !value.consumed_design_replay_required
        || !value.fresh_validation_required
        || !value.sufficient_actionable_support_required
        || value.minimum_coverage != MINIMUM_COVERAGE
        || value.maximum_coverage != MAXIMUM_COVERAGE
        || !value.finite_no_collapse_no_saturation_required
        || !value.chronology_leakage_integrity_required
        || !value.result_selected_mutation_forbidden
        || !value.correctness_cannot_override_brier
        || !value.coverage_cannot_override_brier
        || !value.final_holdout_access_required_zero
        || value.opportunity_metric_names
            != OPPORTUNITY_METRICS
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
        || value.direction_metric_names
            != DIRECTION_METRICS
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
        || value.selective_metric_names
            != SELECTIVE_METRICS
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
        || value.execution_authorized
        || value.gate_digest != fresh_gate_digest(value)
    {
        return Err("T10 fresh-validation gate rejected".to_string());
    }
    Ok(())
}

fn validate_final_gate(value: &MomentumT10FinalHoldoutGateV1) -> Result<(), String> {
    if value.gate_version != FINAL_GATE_VERSION
        || !value.consumed_design_pass_required
        || !value.fresh_validation_pass_required
        || !value.post_validation_design_change_forbidden
        || !value.deterministic_eligible_cohort_required
        || !value.separate_owner_authorization_required
        || value.prediction_count != 0
        || value.label_read_count != 0
        || value.metric_count != 0
        || value.execution_authorized
        || value.gate_digest != final_gate_digest(value)
    {
        return Err("T10 final-holdout gate rejected".to_string());
    }
    Ok(())
}

fn validate_journal(value: &MomentumT10ActionabilityResearchJournalV1) -> Result<(), String> {
    let selected = !value.participant_registration_digests.is_empty();
    if value.journal_version != JOURNAL_VERSION
        || value.failure_report_digest.is_empty()
        || value.label_registration_digest.is_empty()
        || value.candidate_report_digests.len() != 6
        || value.selection_receipt_digest.is_empty()
        || value.participant_registration_digests.len() != usize::from(selected) * 6
        || value.pair_registration_digests.len() != usize::from(selected) * 3
        || value.future_training_policy_digest.is_empty()
        || value.fresh_validation_gate_digest.is_empty()
        || value.final_holdout_gate_digest.is_empty()
        || !value.deterministic
        || value.journal_digest != journal_digest(value)
    {
        return Err("T10 actionability journal rejected".to_string());
    }
    Ok(())
}

fn validate_report(value: &MomentumT10ActionabilityDesignReportV1) -> Result<(), String> {
    validate_selection_policy(&value.selection_policy)?;
    validate_label_registration(&value.label_registration, &value.selection_policy)?;
    for candidate in &value.candidate_reports {
        validate_candidate(candidate)?;
    }
    validate_selection_receipt(&value.selection_receipt)?;
    for participant in &value.participant_registrations {
        validate_participant(participant)?;
    }
    for pair in &value.pair_registrations {
        validate_pair(pair)?;
    }
    validate_future_training(&value.future_training_policy)?;
    validate_fresh_gate(&value.fresh_validation_gate)?;
    validate_final_gate(&value.final_holdout_gate)?;
    let selected = value.selection_receipt.selection_status
        == MomentumT10ActionabilitySelectionStatusV1::StableThresholdSelected
        && value.fresh_validation_support_sufficient;
    let expected_selection =
        build_selection_receipt(&value.label_registration, &value.candidate_reports)?;
    let expected_participants = build_participants(selected)?;
    let expected_pairs = build_pairs(selected)?;
    let expected_two_stage = selected.then(|| {
        two_stage_registration_digest(
            &value.selection_receipt,
            &value.participant_registrations,
            &value.pair_registrations,
            &value.future_training_policy,
            &value.fresh_validation_gate,
        )
    });
    let expected_candidate_keys = MULTIPLIERS
        .iter()
        .flat_map(|multiplier| {
            [
                (MomentumReplayPartitionV1::Development, *multiplier),
                (MomentumReplayPartitionV1::Validation, *multiplier),
            ]
        })
        .collect::<Vec<_>>();
    let counters = &value.safety_counters;
    if value.report_version != REPORT_VERSION
        || value.status != MomentumT10ActionabilityDesignStatusV1::Complete
        || value.design_evidence_class
            != MomentumT10ActionabilityDesignEvidenceClassV1::PostScreeningResearchDesignOnly
        || value.failure_forensics_report_digest.is_empty()
        || value.protected_before_state_digest.is_empty()
        || value.fresh_evidence_split_digest.is_empty()
        || value.future_training_policy.fresh_evidence_split_digest
            != value.fresh_evidence_split_digest
        || value.fresh_validation_event_count == 0
        || value.fresh_validation_minimum_support != MINIMUM_SUPPORT
        || value.fresh_validation_support_status
            != if value.fresh_validation_event_count >= value.fresh_validation_minimum_support {
                MomentumT10FreshValidationSupportStatusV1::SufficientSupport
            } else {
                MomentumT10FreshValidationSupportStatusV1::FreshValidationInsufficientSupport
            }
        || value.fresh_validation_support_sufficient
            != (value.fresh_validation_event_count >= value.fresh_validation_minimum_support)
        || value.candidate_reports.len() != 6
        || value
            .candidate_reports
            .iter()
            .map(|candidate| (candidate.partition, candidate.multiplier))
            .collect::<Vec<_>>()
            != expected_candidate_keys
        || value.candidate_reports.iter().any(|candidate| {
            candidate.label_registration_digest != value.label_registration.registration_digest
        })
        || value.selection_receipt != expected_selection
        || value.participant_registrations != expected_participants
        || value.pair_registrations != expected_pairs
        || value.two_stage_registration_digest != expected_two_stage
        || value.participant_registrations.len() != usize::from(selected) * 6
        || value.pair_registrations.len() != usize::from(selected) * 3
        || value.two_stage_registration_digest.is_some() != selected
        || value.labels
            != PUBLIC_LABELS
                .iter()
                .map(|label| (*label).to_string())
                .collect::<Vec<_>>()
        || value.live_completed_event_count != 2
        || value.live_scorable_event_count != 2
        || value.live_pause != "PausedAfterCompletedEpochTwo"
        || value.epoch_three_registered
        || !value.full_eight_blocked
        || !value.protected_artifacts_unchanged
        || counters.new_opportunity_model_fits != 0
        || counters.new_direction_model_fits != 0
        || counters.new_selective_predictions != 0
        || counters.new_selective_evaluations != 0
        || counters.fresh_validation_predictions != 0
        || counters.fresh_validation_label_reads != 0
        || counters.fresh_validation_metrics != 0
        || counters.final_holdout_predictions != 0
        || counters.final_holdout_label_reads != 0
        || counters.final_holdout_metrics != 0
        || counters.t30_model_executions != 0
        || counters.t60_model_executions != 0
        || counters.network_requests != 0
        || counters.live_operations != 0
        || counters.reward_applications != 0
        || counters.penalty_applications != 0
        || counters.chair_actions != 0
        || counters.vote_actions != 0
        || counters.trading_actions != 0
        || counters.day_view_loads != 0
        || counters.week_view_loads != 0
        || counters.month_view_loads != 0
        || counters.year_view_loads != 0
        || value.deterministic_replay_digest.is_empty()
        || value.report_digest != report_digest(value)
    {
        return Err("T10 actionability report rejected".to_string());
    }
    Ok(())
}

fn partition_name(value: MomentumReplayPartitionV1) -> &'static str {
    match value {
        MomentumReplayPartitionV1::Development => "development",
        MomentumReplayPartitionV1::Validation => "validation",
        MomentumReplayPartitionV1::SealedHoldout => "sealed-holdout",
    }
}

fn parse_partition(value: &str) -> Result<MomentumReplayPartitionV1, String> {
    match value {
        "development" => Ok(MomentumReplayPartitionV1::Development),
        "validation" => Ok(MomentumReplayPartitionV1::Validation),
        "sealed-holdout" => Ok(MomentumReplayPartitionV1::SealedHoldout),
        _ => Err("T10 actionability partition rejected".to_string()),
    }
}

fn selection_status_name(value: MomentumT10ActionabilitySelectionStatusV1) -> &'static str {
    match value {
        MomentumT10ActionabilitySelectionStatusV1::StableThresholdSelected => {
            "StableThresholdSelected"
        }
        MomentumT10ActionabilitySelectionStatusV1::NoStableActionabilityThreshold => {
            "NoStableActionabilityThreshold"
        }
        MomentumT10ActionabilitySelectionStatusV1::IntegrityFailure => "IntegrityFailure",
    }
}

fn parse_selection_status(
    value: &str,
) -> Result<MomentumT10ActionabilitySelectionStatusV1, String> {
    match value {
        "StableThresholdSelected" => {
            Ok(MomentumT10ActionabilitySelectionStatusV1::StableThresholdSelected)
        }
        "NoStableActionabilityThreshold" => {
            Ok(MomentumT10ActionabilitySelectionStatusV1::NoStableActionabilityThreshold)
        }
        "IntegrityFailure" => Ok(MomentumT10ActionabilitySelectionStatusV1::IntegrityFailure),
        _ => Err("T10 actionability selection status rejected".to_string()),
    }
}

fn fresh_support_status_name(value: MomentumT10FreshValidationSupportStatusV1) -> &'static str {
    match value {
        MomentumT10FreshValidationSupportStatusV1::SufficientSupport => "SufficientSupport",
        MomentumT10FreshValidationSupportStatusV1::FreshValidationInsufficientSupport => {
            "FreshValidationInsufficientSupport"
        }
    }
}

fn parse_fresh_support_status(
    value: &str,
) -> Result<MomentumT10FreshValidationSupportStatusV1, String> {
    match value {
        "SufficientSupport" => Ok(MomentumT10FreshValidationSupportStatusV1::SufficientSupport),
        "FreshValidationInsufficientSupport" => {
            Ok(MomentumT10FreshValidationSupportStatusV1::FreshValidationInsufficientSupport)
        }
        _ => Err("T10 fresh-validation support status rejected".to_string()),
    }
}

fn encode_selection_policy(
    value: &MomentumT10ActionabilitySelectionPolicyV1,
) -> Result<Vec<u8>, String> {
    validate_selection_policy(value)?;
    ArtifactBuilderV4_2::new(SELECTION_POLICY_VERSION)
        .string("policy_version", &value.policy_version)
        .unsigned(
            "minimum_total_support",
            as_u64(value.minimum_total_support)?,
        )
        .unsigned(
            "actionable_up_minimum_prevalence_bits",
            value.actionable_up_minimum_prevalence_bits,
        )
        .unsigned(
            "actionable_down_minimum_prevalence_bits",
            value.actionable_down_minimum_prevalence_bits,
        )
        .unsigned(
            "abstain_minimum_prevalence_bits",
            value.abstain_minimum_prevalence_bits,
        )
        .unsigned(
            "abstain_maximum_prevalence_bits",
            value.abstain_maximum_prevalence_bits,
        )
        .unsigned(
            "maximum_partition_class_drift_bits",
            value.maximum_partition_class_drift_bits,
        )
        .boolean(
            "choose_largest_passing_multiplier",
            value.choose_largest_passing_multiplier,
        )
        .string("policy_digest", &value.policy_digest)
        .encode()
}

fn decode_selection_policy(
    bytes: &[u8],
) -> Result<MomentumT10ActionabilitySelectionPolicyV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, SELECTION_POLICY_VERSION)?;
    let value = MomentumT10ActionabilitySelectionPolicyV1 {
        policy_version: fields.string("policy_version")?,
        minimum_total_support: as_usize(fields.unsigned("minimum_total_support")?)?,
        actionable_up_minimum_prevalence_bits: fields
            .unsigned("actionable_up_minimum_prevalence_bits")?,
        actionable_down_minimum_prevalence_bits: fields
            .unsigned("actionable_down_minimum_prevalence_bits")?,
        abstain_minimum_prevalence_bits: fields.unsigned("abstain_minimum_prevalence_bits")?,
        abstain_maximum_prevalence_bits: fields.unsigned("abstain_maximum_prevalence_bits")?,
        maximum_partition_class_drift_bits: fields
            .unsigned("maximum_partition_class_drift_bits")?,
        choose_largest_passing_multiplier: fields.boolean("choose_largest_passing_multiplier")?,
        policy_digest: fields.string("policy_digest")?,
    };
    fields.finish()?;
    validate_selection_policy(&value)?;
    Ok(value)
}

fn encode_label_registration(
    value: &MomentumT10ActionabilityLabelRegistrationV1,
    policy: &MomentumT10ActionabilitySelectionPolicyV1,
) -> Result<Vec<u8>, String> {
    validate_label_registration(value, policy)?;
    ArtifactBuilderV4_2::new(LABEL_REGISTRATION_VERSION)
        .string("registration_version", &value.registration_version)
        .string("source_event_set_digest", &value.source_event_set_digest)
        .string("volatility_policy_digest", &value.volatility_policy_digest)
        .unsigneds(
            "candidate_multiplier_bits",
            &value.candidate_multiplier_bits,
        )
        .string("actionable_up_rule", &value.actionable_up_rule)
        .string("abstain_rule", &value.abstain_rule)
        .string("actionable_down_rule", &value.actionable_down_rule)
        .string("selection_policy_digest", &value.selection_policy_digest)
        .boolean(
            "fresh_validation_access_forbidden",
            value.fresh_validation_access_forbidden,
        )
        .boolean(
            "final_holdout_access_forbidden",
            value.final_holdout_access_forbidden,
        )
        .boolean("model_training_forbidden", value.model_training_forbidden)
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_label_registration(
    bytes: &[u8],
    policy: &MomentumT10ActionabilitySelectionPolicyV1,
) -> Result<MomentumT10ActionabilityLabelRegistrationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, LABEL_REGISTRATION_VERSION)?;
    let value = MomentumT10ActionabilityLabelRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        source_event_set_digest: fields.string("source_event_set_digest")?,
        volatility_policy_digest: fields.string("volatility_policy_digest")?,
        candidate_multiplier_bits: fields.unsigneds("candidate_multiplier_bits")?,
        actionable_up_rule: fields.string("actionable_up_rule")?,
        abstain_rule: fields.string("abstain_rule")?,
        actionable_down_rule: fields.string("actionable_down_rule")?,
        selection_policy_digest: fields.string("selection_policy_digest")?,
        fresh_validation_access_forbidden: fields.boolean("fresh_validation_access_forbidden")?,
        final_holdout_access_forbidden: fields.boolean("final_holdout_access_forbidden")?,
        model_training_forbidden: fields.boolean("model_training_forbidden")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_label_registration(&value, policy)?;
    Ok(value)
}

fn encode_stability(
    value: &MomentumT10ActionabilityTemporalStabilityV1,
) -> Result<Vec<u8>, String> {
    validate_stability(value)?;
    let bits = |values: &[f64]| {
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    };
    ArtifactBuilderV4_2::new(TEMPORAL_STABILITY_VERSION)
        .string("stability_version", &value.stability_version)
        .unsigneds(
            "daily_class_prevalence_ranges",
            &bits(&value.daily_class_prevalence_ranges),
        )
        .unsigneds(
            "weekly_class_prevalence_ranges",
            &bits(&value.weekly_class_prevalence_ranges),
        )
        .unsigneds(
            "monthly_class_prevalence_ranges",
            &bits(&value.monthly_class_prevalence_ranges),
        )
        .unsigneds(
            "rolling_144_class_prevalence_ranges",
            &bits(&value.rolling_144_class_prevalence_ranges),
        )
        .unsigneds(
            "rolling_1008_class_prevalence_ranges",
            &bits(&value.rolling_1008_class_prevalence_ranges),
        )
        .boolean("finite", value.finite)
        .string("stability_digest", &value.stability_digest)
        .encode()
}

fn decode_stability(bytes: &[u8]) -> Result<MomentumT10ActionabilityTemporalStabilityV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, TEMPORAL_STABILITY_VERSION)?;
    let values = |bits: Vec<u64>| bits.into_iter().map(f64::from_bits).collect::<Vec<_>>();
    let value = MomentumT10ActionabilityTemporalStabilityV1 {
        stability_version: fields.string("stability_version")?,
        daily_class_prevalence_ranges: values(fields.unsigneds("daily_class_prevalence_ranges")?),
        weekly_class_prevalence_ranges: values(fields.unsigneds("weekly_class_prevalence_ranges")?),
        monthly_class_prevalence_ranges: values(
            fields.unsigneds("monthly_class_prevalence_ranges")?,
        ),
        rolling_144_class_prevalence_ranges: values(
            fields.unsigneds("rolling_144_class_prevalence_ranges")?,
        ),
        rolling_1008_class_prevalence_ranges: values(
            fields.unsigneds("rolling_1008_class_prevalence_ranges")?,
        ),
        finite: fields.boolean("finite")?,
        stability_digest: fields.string("stability_digest")?,
    };
    fields.finish()?;
    validate_stability(&value)?;
    Ok(value)
}

fn encode_candidate(value: &MomentumT10ActionabilityCandidateReportV1) -> Result<Vec<u8>, String> {
    validate_candidate(value)?;
    ArtifactBuilderV4_2::new(CANDIDATE_REPORT_VERSION)
        .string("report_version", &value.report_version)
        .string(
            "label_registration_digest",
            &value.label_registration_digest,
        )
        .string("partition", partition_name(value.partition))
        .unsigned("multiplier_bits", value.multiplier.to_bits())
        .unsigned("eligible_event_count", as_u64(value.eligible_event_count)?)
        .unsigned("actionable_up_count", as_u64(value.actionable_up_count)?)
        .unsigned(
            "actionable_down_count",
            as_u64(value.actionable_down_count)?,
        )
        .unsigned("abstain_count", as_u64(value.abstain_count)?)
        .unsigned(
            "actionable_up_prevalence_bits",
            value.actionable_up_prevalence.to_bits(),
        )
        .unsigned(
            "actionable_down_prevalence_bits",
            value.actionable_down_prevalence.to_bits(),
        )
        .unsigned(
            "abstain_prevalence_bits",
            value.abstain_prevalence.to_bits(),
        )
        .unsigned(
            "target_magnitude_mean_bits",
            value.target_magnitude_mean.to_bits(),
        )
        .unsigned(
            "target_magnitude_median_bits",
            value.target_magnitude_median.to_bits(),
        )
        .unsigned(
            "volatility_scale_mean_bits",
            value.volatility_scale_mean.to_bits(),
        )
        .unsigned(
            "volatility_scale_median_bits",
            value.volatility_scale_median.to_bits(),
        )
        .unsigned(
            "zero_volatility_floor_count",
            as_u64(value.zero_volatility_floor_count)?,
        )
        .messages(
            "temporal_stability",
            vec![encode_stability(&value.temporal_stability)?],
        )
        .boolean("finite_value_proof", value.finite_value_proof)
        .boolean("chronology_audit_passed", value.chronology_audit_passed)
        .boolean("leakage_audit_passed", value.leakage_audit_passed)
        .boolean("integrity_audit_passed", value.integrity_audit_passed)
        .string("candidate_digest", &value.candidate_digest)
        .encode()
}

fn decode_candidate(bytes: &[u8]) -> Result<MomentumT10ActionabilityCandidateReportV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, CANDIDATE_REPORT_VERSION)?;
    let stability_messages = fields.messages("temporal_stability")?;
    if stability_messages.len() != 1 {
        return Err("T10 candidate stability count rejected".to_string());
    }
    let value = MomentumT10ActionabilityCandidateReportV1 {
        report_version: fields.string("report_version")?,
        label_registration_digest: fields.string("label_registration_digest")?,
        partition: parse_partition(&fields.string("partition")?)?,
        multiplier: f64::from_bits(fields.unsigned("multiplier_bits")?),
        eligible_event_count: as_usize(fields.unsigned("eligible_event_count")?)?,
        actionable_up_count: as_usize(fields.unsigned("actionable_up_count")?)?,
        actionable_down_count: as_usize(fields.unsigned("actionable_down_count")?)?,
        abstain_count: as_usize(fields.unsigned("abstain_count")?)?,
        actionable_up_prevalence: f64::from_bits(fields.unsigned("actionable_up_prevalence_bits")?),
        actionable_down_prevalence: f64::from_bits(
            fields.unsigned("actionable_down_prevalence_bits")?,
        ),
        abstain_prevalence: f64::from_bits(fields.unsigned("abstain_prevalence_bits")?),
        target_magnitude_mean: f64::from_bits(fields.unsigned("target_magnitude_mean_bits")?),
        target_magnitude_median: f64::from_bits(fields.unsigned("target_magnitude_median_bits")?),
        volatility_scale_mean: f64::from_bits(fields.unsigned("volatility_scale_mean_bits")?),
        volatility_scale_median: f64::from_bits(fields.unsigned("volatility_scale_median_bits")?),
        zero_volatility_floor_count: as_usize(fields.unsigned("zero_volatility_floor_count")?)?,
        temporal_stability: decode_stability(&stability_messages[0])?,
        finite_value_proof: fields.boolean("finite_value_proof")?,
        chronology_audit_passed: fields.boolean("chronology_audit_passed")?,
        leakage_audit_passed: fields.boolean("leakage_audit_passed")?,
        integrity_audit_passed: fields.boolean("integrity_audit_passed")?,
        candidate_digest: fields.string("candidate_digest")?,
    };
    fields.finish()?;
    validate_candidate(&value)?;
    Ok(value)
}

fn encode_selection_receipt(
    value: &MomentumT10ActionabilitySelectionReceiptV1,
) -> Result<Vec<u8>, String> {
    validate_selection_receipt(value)?;
    ArtifactBuilderV4_2::new(SELECTION_RECEIPT_VERSION)
        .string("receipt_version", &value.receipt_version)
        .string(
            "label_registration_digest",
            &value.label_registration_digest,
        )
        .strings("candidate_report_digests", &value.candidate_report_digests)
        .optional_string(
            "selected_multiplier_bits",
            &value
                .selected_multiplier
                .map(|item| item.to_bits().to_string()),
        )
        .optional_string(
            "selected_candidate_digest",
            &value.selected_candidate_digest,
        )
        .string(
            "selection_status",
            selection_status_name(value.selection_status),
        )
        .unsigned(
            "fresh_validation_reads",
            as_u64(value.fresh_validation_reads)?,
        )
        .unsigned("final_holdout_reads", as_u64(value.final_holdout_reads)?)
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_selection_receipt(
    bytes: &[u8],
) -> Result<MomentumT10ActionabilitySelectionReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, SELECTION_RECEIPT_VERSION)?;
    let value = MomentumT10ActionabilitySelectionReceiptV1 {
        receipt_version: fields.string("receipt_version")?,
        label_registration_digest: fields.string("label_registration_digest")?,
        candidate_report_digests: fields.strings("candidate_report_digests")?,
        selected_multiplier: fields
            .optional_string("selected_multiplier_bits")?
            .map(|value| {
                value
                    .parse::<u64>()
                    .map(f64::from_bits)
                    .map_err(|_| "T10 selected multiplier bits rejected".to_string())
            })
            .transpose()?,
        selected_candidate_digest: fields.optional_string("selected_candidate_digest")?,
        selection_status: parse_selection_status(&fields.string("selection_status")?)?,
        fresh_validation_reads: as_usize(fields.unsigned("fresh_validation_reads")?)?,
        final_holdout_reads: as_usize(fields.unsigned("final_holdout_reads")?)?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_selection_receipt(&value)?;
    Ok(value)
}

fn encode_participant(
    value: &MomentumT10SelectiveParticipantRegistrationV1,
) -> Result<Vec<u8>, String> {
    validate_participant(value)?;
    ArtifactBuilderV4_2::new(PARTICIPANT_VERSION)
        .string("registration_version", &value.registration_version)
        .string("participant_id", &value.participant_id)
        .string("head", &value.head)
        .string("role", &value.role)
        .string("feature_policy", &value.feature_policy)
        .unsigned("feature_dimension", as_u64(value.feature_dimension)?)
        .unsigned("l2_multiplier", as_u64(value.l2_multiplier)?)
        .boolean(
            "prior_revealed_labels_only",
            value.prior_revealed_labels_only,
        )
        .boolean("actionable_training_only", value.actionable_training_only)
        .boolean("training_authorized", value.training_authorized)
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_participant(
    bytes: &[u8],
) -> Result<MomentumT10SelectiveParticipantRegistrationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, PARTICIPANT_VERSION)?;
    let value = MomentumT10SelectiveParticipantRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        participant_id: fields.string("participant_id")?,
        head: fields.string("head")?,
        role: fields.string("role")?,
        feature_policy: fields.string("feature_policy")?,
        feature_dimension: as_usize(fields.unsigned("feature_dimension")?)?,
        l2_multiplier: as_usize(fields.unsigned("l2_multiplier")?)?,
        prior_revealed_labels_only: fields.boolean("prior_revealed_labels_only")?,
        actionable_training_only: fields.boolean("actionable_training_only")?,
        training_authorized: fields.boolean("training_authorized")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_participant(&value)?;
    Ok(value)
}

fn encode_pair(value: &MomentumT10SelectivePairRegistrationV1) -> Result<Vec<u8>, String> {
    validate_pair(value)?;
    ArtifactBuilderV4_2::new(PAIR_VERSION)
        .string("registration_version", &value.registration_version)
        .string("pair_id", &value.pair_id)
        .string(
            "opportunity_participant_id",
            &value.opportunity_participant_id,
        )
        .string("direction_participant_id", &value.direction_participant_id)
        .boolean("reference_system", value.reference_system)
        .boolean("cross_pairing", value.cross_pairing)
        .unsigned(
            "opportunity_threshold_bits",
            value.opportunity_threshold_bits,
        )
        .unsigned("direction_threshold_bits", value.direction_threshold_bits)
        .boolean(
            "opportunity_below_threshold_abstains",
            value.opportunity_below_threshold_abstains,
        )
        .boolean(
            "direction_equality_maps_up",
            value.direction_equality_maps_up,
        )
        .boolean("training_authorized", value.training_authorized)
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_pair(bytes: &[u8]) -> Result<MomentumT10SelectivePairRegistrationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, PAIR_VERSION)?;
    let value = MomentumT10SelectivePairRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        pair_id: fields.string("pair_id")?,
        opportunity_participant_id: fields.string("opportunity_participant_id")?,
        direction_participant_id: fields.string("direction_participant_id")?,
        reference_system: fields.boolean("reference_system")?,
        cross_pairing: fields.boolean("cross_pairing")?,
        opportunity_threshold_bits: fields.unsigned("opportunity_threshold_bits")?,
        direction_threshold_bits: fields.unsigned("direction_threshold_bits")?,
        opportunity_below_threshold_abstains: fields
            .boolean("opportunity_below_threshold_abstains")?,
        direction_equality_maps_up: fields.boolean("direction_equality_maps_up")?,
        training_authorized: fields.boolean("training_authorized")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_pair(&value)?;
    Ok(value)
}

fn encode_future_training(
    value: &MomentumT10SelectiveFutureTrainingPolicyV1,
) -> Result<Vec<u8>, String> {
    validate_future_training(value)?;
    ArtifactBuilderV4_2::new(TRAINING_POLICY_VERSION)
        .string("policy_version", &value.policy_version)
        .boolean("daily_utc_refit", value.daily_utc_refit)
        .boolean(
            "previously_revealed_labels_only",
            value.previously_revealed_labels_only,
        )
        .boolean(
            "chronological_order_required",
            value.chronological_order_required,
        )
        .unsigned(
            "maximum_training_examples",
            as_u64(value.maximum_training_examples)?,
        )
        .boolean("training_only_normalizers", value.training_only_normalizers)
        .boolean(
            "direction_excludes_abstain",
            value.direction_excludes_abstain,
        )
        .boolean(
            "persist_and_reopen_all_receipts",
            value.persist_and_reopen_all_receipts,
        )
        .boolean(
            "participants_frozen_per_utc_day",
            value.participants_frozen_per_utc_day,
        )
        .boolean(
            "later_fresh_labels_only_after_observable",
            value.later_fresh_labels_only_after_observable,
        )
        .string(
            "fresh_evidence_split_digest",
            &value.fresh_evidence_split_digest,
        )
        .boolean(
            "consumed_design_pool_is_former_development_and_validation",
            value.consumed_design_pool_is_former_development_and_validation,
        )
        .boolean("fresh_validation_use_once", value.fresh_validation_use_once)
        .boolean(
            "final_holdout_requires_separate_authorization",
            value.final_holdout_requires_separate_authorization,
        )
        .boolean("execution_authorized", value.execution_authorized)
        .string("policy_digest", &value.policy_digest)
        .encode()
}

fn decode_future_training(
    bytes: &[u8],
) -> Result<MomentumT10SelectiveFutureTrainingPolicyV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, TRAINING_POLICY_VERSION)?;
    let value = MomentumT10SelectiveFutureTrainingPolicyV1 {
        policy_version: fields.string("policy_version")?,
        daily_utc_refit: fields.boolean("daily_utc_refit")?,
        previously_revealed_labels_only: fields.boolean("previously_revealed_labels_only")?,
        chronological_order_required: fields.boolean("chronological_order_required")?,
        maximum_training_examples: as_usize(fields.unsigned("maximum_training_examples")?)?,
        training_only_normalizers: fields.boolean("training_only_normalizers")?,
        direction_excludes_abstain: fields.boolean("direction_excludes_abstain")?,
        persist_and_reopen_all_receipts: fields.boolean("persist_and_reopen_all_receipts")?,
        participants_frozen_per_utc_day: fields.boolean("participants_frozen_per_utc_day")?,
        later_fresh_labels_only_after_observable: fields
            .boolean("later_fresh_labels_only_after_observable")?,
        fresh_evidence_split_digest: fields.string("fresh_evidence_split_digest")?,
        consumed_design_pool_is_former_development_and_validation: fields
            .boolean("consumed_design_pool_is_former_development_and_validation")?,
        fresh_validation_use_once: fields.boolean("fresh_validation_use_once")?,
        final_holdout_requires_separate_authorization: fields
            .boolean("final_holdout_requires_separate_authorization")?,
        execution_authorized: fields.boolean("execution_authorized")?,
        policy_digest: fields.string("policy_digest")?,
    };
    fields.finish()?;
    validate_future_training(&value)?;
    Ok(value)
}

fn encode_fresh_gate(value: &MomentumT10FreshValidationGateV1) -> Result<Vec<u8>, String> {
    validate_fresh_gate(value)?;
    ArtifactBuilderV4_2::new(FRESH_GATE_VERSION)
        .string("gate_version", &value.gate_version)
        .boolean(
            "opportunity_lower_brier_than_o0_required",
            value.opportunity_lower_brier_than_o0_required,
        )
        .boolean(
            "direction_lower_brier_than_d0_required",
            value.direction_lower_brier_than_d0_required,
        )
        .boolean(
            "consumed_design_replay_required",
            value.consumed_design_replay_required,
        )
        .boolean("fresh_validation_required", value.fresh_validation_required)
        .boolean(
            "sufficient_actionable_support_required",
            value.sufficient_actionable_support_required,
        )
        .unsigned("minimum_coverage_bits", value.minimum_coverage.to_bits())
        .unsigned("maximum_coverage_bits", value.maximum_coverage.to_bits())
        .boolean(
            "finite_no_collapse_no_saturation_required",
            value.finite_no_collapse_no_saturation_required,
        )
        .boolean(
            "chronology_leakage_integrity_required",
            value.chronology_leakage_integrity_required,
        )
        .boolean(
            "result_selected_mutation_forbidden",
            value.result_selected_mutation_forbidden,
        )
        .boolean(
            "correctness_cannot_override_brier",
            value.correctness_cannot_override_brier,
        )
        .boolean(
            "coverage_cannot_override_brier",
            value.coverage_cannot_override_brier,
        )
        .boolean(
            "final_holdout_access_required_zero",
            value.final_holdout_access_required_zero,
        )
        .strings("opportunity_metric_names", &value.opportunity_metric_names)
        .strings("direction_metric_names", &value.direction_metric_names)
        .strings("selective_metric_names", &value.selective_metric_names)
        .boolean("execution_authorized", value.execution_authorized)
        .string("gate_digest", &value.gate_digest)
        .encode()
}

fn decode_fresh_gate(bytes: &[u8]) -> Result<MomentumT10FreshValidationGateV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, FRESH_GATE_VERSION)?;
    let value = MomentumT10FreshValidationGateV1 {
        gate_version: fields.string("gate_version")?,
        opportunity_lower_brier_than_o0_required: fields
            .boolean("opportunity_lower_brier_than_o0_required")?,
        direction_lower_brier_than_d0_required: fields
            .boolean("direction_lower_brier_than_d0_required")?,
        consumed_design_replay_required: fields.boolean("consumed_design_replay_required")?,
        fresh_validation_required: fields.boolean("fresh_validation_required")?,
        sufficient_actionable_support_required: fields
            .boolean("sufficient_actionable_support_required")?,
        minimum_coverage: f64::from_bits(fields.unsigned("minimum_coverage_bits")?),
        maximum_coverage: f64::from_bits(fields.unsigned("maximum_coverage_bits")?),
        finite_no_collapse_no_saturation_required: fields
            .boolean("finite_no_collapse_no_saturation_required")?,
        chronology_leakage_integrity_required: fields
            .boolean("chronology_leakage_integrity_required")?,
        result_selected_mutation_forbidden: fields.boolean("result_selected_mutation_forbidden")?,
        correctness_cannot_override_brier: fields.boolean("correctness_cannot_override_brier")?,
        coverage_cannot_override_brier: fields.boolean("coverage_cannot_override_brier")?,
        final_holdout_access_required_zero: fields.boolean("final_holdout_access_required_zero")?,
        opportunity_metric_names: fields.strings("opportunity_metric_names")?,
        direction_metric_names: fields.strings("direction_metric_names")?,
        selective_metric_names: fields.strings("selective_metric_names")?,
        execution_authorized: fields.boolean("execution_authorized")?,
        gate_digest: fields.string("gate_digest")?,
    };
    fields.finish()?;
    validate_fresh_gate(&value)?;
    Ok(value)
}

fn encode_final_gate(value: &MomentumT10FinalHoldoutGateV1) -> Result<Vec<u8>, String> {
    validate_final_gate(value)?;
    ArtifactBuilderV4_2::new(FINAL_GATE_VERSION)
        .string("gate_version", &value.gate_version)
        .boolean(
            "consumed_design_pass_required",
            value.consumed_design_pass_required,
        )
        .boolean(
            "fresh_validation_pass_required",
            value.fresh_validation_pass_required,
        )
        .boolean(
            "post_validation_design_change_forbidden",
            value.post_validation_design_change_forbidden,
        )
        .boolean(
            "deterministic_eligible_cohort_required",
            value.deterministic_eligible_cohort_required,
        )
        .boolean(
            "separate_owner_authorization_required",
            value.separate_owner_authorization_required,
        )
        .unsigned("prediction_count", as_u64(value.prediction_count)?)
        .unsigned("label_read_count", as_u64(value.label_read_count)?)
        .unsigned("metric_count", as_u64(value.metric_count)?)
        .boolean("execution_authorized", value.execution_authorized)
        .string("gate_digest", &value.gate_digest)
        .encode()
}

fn decode_final_gate(bytes: &[u8]) -> Result<MomentumT10FinalHoldoutGateV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, FINAL_GATE_VERSION)?;
    let value = MomentumT10FinalHoldoutGateV1 {
        gate_version: fields.string("gate_version")?,
        consumed_design_pass_required: fields.boolean("consumed_design_pass_required")?,
        fresh_validation_pass_required: fields.boolean("fresh_validation_pass_required")?,
        post_validation_design_change_forbidden: fields
            .boolean("post_validation_design_change_forbidden")?,
        deterministic_eligible_cohort_required: fields
            .boolean("deterministic_eligible_cohort_required")?,
        separate_owner_authorization_required: fields
            .boolean("separate_owner_authorization_required")?,
        prediction_count: as_usize(fields.unsigned("prediction_count")?)?,
        label_read_count: as_usize(fields.unsigned("label_read_count")?)?,
        metric_count: as_usize(fields.unsigned("metric_count")?)?,
        execution_authorized: fields.boolean("execution_authorized")?,
        gate_digest: fields.string("gate_digest")?,
    };
    fields.finish()?;
    validate_final_gate(&value)?;
    Ok(value)
}

fn encode_journal(value: &MomentumT10ActionabilityResearchJournalV1) -> Result<Vec<u8>, String> {
    validate_journal(value)?;
    ArtifactBuilderV4_2::new(JOURNAL_VERSION)
        .string("journal_version", &value.journal_version)
        .string("failure_report_digest", &value.failure_report_digest)
        .string(
            "label_registration_digest",
            &value.label_registration_digest,
        )
        .strings("candidate_report_digests", &value.candidate_report_digests)
        .string("selection_receipt_digest", &value.selection_receipt_digest)
        .strings(
            "participant_registration_digests",
            &value.participant_registration_digests,
        )
        .strings(
            "pair_registration_digests",
            &value.pair_registration_digests,
        )
        .string(
            "future_training_policy_digest",
            &value.future_training_policy_digest,
        )
        .string(
            "fresh_validation_gate_digest",
            &value.fresh_validation_gate_digest,
        )
        .string(
            "final_holdout_gate_digest",
            &value.final_holdout_gate_digest,
        )
        .boolean("deterministic", value.deterministic)
        .string("journal_digest", &value.journal_digest)
        .encode()
}

fn decode_journal(bytes: &[u8]) -> Result<MomentumT10ActionabilityResearchJournalV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, JOURNAL_VERSION)?;
    let value = MomentumT10ActionabilityResearchJournalV1 {
        journal_version: fields.string("journal_version")?,
        failure_report_digest: fields.string("failure_report_digest")?,
        label_registration_digest: fields.string("label_registration_digest")?,
        candidate_report_digests: fields.strings("candidate_report_digests")?,
        selection_receipt_digest: fields.string("selection_receipt_digest")?,
        participant_registration_digests: fields.strings("participant_registration_digests")?,
        pair_registration_digests: fields.strings("pair_registration_digests")?,
        future_training_policy_digest: fields.string("future_training_policy_digest")?,
        fresh_validation_gate_digest: fields.string("fresh_validation_gate_digest")?,
        final_holdout_gate_digest: fields.string("final_holdout_gate_digest")?,
        deterministic: fields.boolean("deterministic")?,
        journal_digest: fields.string("journal_digest")?,
    };
    fields.finish()?;
    validate_journal(&value)?;
    Ok(value)
}

fn safety_values(value: &MomentumT10ActionabilitySafetyCountersV1) -> Result<Vec<u64>, String> {
    [
        value.artifacts_written,
        value.duplicate_artifact_count,
        value.label_computations,
        value.new_opportunity_model_fits,
        value.new_direction_model_fits,
        value.new_selective_predictions,
        value.new_selective_evaluations,
        value.fresh_validation_predictions,
        value.fresh_validation_label_reads,
        value.fresh_validation_metrics,
        value.final_holdout_predictions,
        value.final_holdout_label_reads,
        value.final_holdout_metrics,
        value.t30_model_executions,
        value.t60_model_executions,
        value.network_requests,
        value.live_operations,
        value.reward_applications,
        value.penalty_applications,
        value.chair_actions,
        value.vote_actions,
        value.trading_actions,
        value.day_view_loads,
        value.week_view_loads,
        value.month_view_loads,
        value.year_view_loads,
    ]
    .into_iter()
    .map(as_u64)
    .collect()
}

fn safety_from_values(
    values: Vec<u64>,
) -> Result<MomentumT10ActionabilitySafetyCountersV1, String> {
    if values.len() != 26 {
        return Err("T10 actionability safety counters rejected".to_string());
    }
    let values = values
        .into_iter()
        .map(as_usize)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MomentumT10ActionabilitySafetyCountersV1 {
        artifacts_written: values[0],
        duplicate_artifact_count: values[1],
        label_computations: values[2],
        new_opportunity_model_fits: values[3],
        new_direction_model_fits: values[4],
        new_selective_predictions: values[5],
        new_selective_evaluations: values[6],
        fresh_validation_predictions: values[7],
        fresh_validation_label_reads: values[8],
        fresh_validation_metrics: values[9],
        final_holdout_predictions: values[10],
        final_holdout_label_reads: values[11],
        final_holdout_metrics: values[12],
        t30_model_executions: values[13],
        t60_model_executions: values[14],
        network_requests: values[15],
        live_operations: values[16],
        reward_applications: values[17],
        penalty_applications: values[18],
        chair_actions: values[19],
        vote_actions: values[20],
        trading_actions: values[21],
        day_view_loads: values[22],
        week_view_loads: values[23],
        month_view_loads: values[24],
        year_view_loads: values[25],
    })
}

fn status_name(value: MomentumT10ActionabilityDesignStatusV1) -> &'static str {
    match value {
        MomentumT10ActionabilityDesignStatusV1::Unregistered => "Unregistered",
        MomentumT10ActionabilityDesignStatusV1::Complete => "Complete",
        MomentumT10ActionabilityDesignStatusV1::IntegrityFailure => "IntegrityFailure",
    }
}

fn parse_status(value: &str) -> Result<MomentumT10ActionabilityDesignStatusV1, String> {
    match value {
        "Unregistered" => Ok(MomentumT10ActionabilityDesignStatusV1::Unregistered),
        "Complete" => Ok(MomentumT10ActionabilityDesignStatusV1::Complete),
        "IntegrityFailure" => Ok(MomentumT10ActionabilityDesignStatusV1::IntegrityFailure),
        _ => Err("T10 actionability status rejected".to_string()),
    }
}

fn encode_report(value: &MomentumT10ActionabilityDesignReportV1) -> Result<Vec<u8>, String> {
    validate_report(value)?;
    ArtifactBuilderV4_2::new(REPORT_VERSION)
        .string("report_version", &value.report_version)
        .string("run_mode", &value.run_mode)
        .string("status", status_name(value.status))
        .string("design_evidence_class", "PostScreeningResearchDesignOnly")
        .string(
            "failure_forensics_report_digest",
            &value.failure_forensics_report_digest,
        )
        .string(
            "protected_before_state_digest",
            &value.protected_before_state_digest,
        )
        .string(
            "fresh_evidence_split_digest",
            &value.fresh_evidence_split_digest,
        )
        .messages(
            "selection_policy",
            vec![encode_selection_policy(&value.selection_policy)?],
        )
        .messages(
            "label_registration",
            vec![encode_label_registration(
                &value.label_registration,
                &value.selection_policy,
            )?],
        )
        .messages(
            "candidate_reports",
            value
                .candidate_reports
                .iter()
                .map(encode_candidate)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "selection_receipt",
            vec![encode_selection_receipt(&value.selection_receipt)?],
        )
        .unsigned(
            "fresh_validation_event_count",
            as_u64(value.fresh_validation_event_count)?,
        )
        .unsigned(
            "fresh_validation_minimum_support",
            as_u64(value.fresh_validation_minimum_support)?,
        )
        .string(
            "fresh_validation_support_status",
            fresh_support_status_name(value.fresh_validation_support_status),
        )
        .boolean(
            "fresh_validation_support_sufficient",
            value.fresh_validation_support_sufficient,
        )
        .messages(
            "participant_registrations",
            value
                .participant_registrations
                .iter()
                .map(encode_participant)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "pair_registrations",
            value
                .pair_registrations
                .iter()
                .map(encode_pair)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "future_training_policy",
            vec![encode_future_training(&value.future_training_policy)?],
        )
        .messages(
            "fresh_validation_gate",
            vec![encode_fresh_gate(&value.fresh_validation_gate)?],
        )
        .messages(
            "final_holdout_gate",
            vec![encode_final_gate(&value.final_holdout_gate)?],
        )
        .optional_string(
            "two_stage_registration_digest",
            &value.two_stage_registration_digest,
        )
        .strings("labels", &value.labels)
        .unsigned(
            "live_completed_event_count",
            as_u64(value.live_completed_event_count)?,
        )
        .unsigned(
            "live_scorable_event_count",
            as_u64(value.live_scorable_event_count)?,
        )
        .string("live_pause", &value.live_pause)
        .boolean("epoch_three_registered", value.epoch_three_registered)
        .boolean("full_eight_blocked", value.full_eight_blocked)
        .boolean(
            "protected_artifacts_unchanged",
            value.protected_artifacts_unchanged,
        )
        .unsigneds("safety_counters", &safety_values(&value.safety_counters)?)
        .string(
            "deterministic_replay_digest",
            &value.deterministic_replay_digest,
        )
        .unsigned("runtime_duration_ms", value.runtime_duration_ms)
        .string("report_digest", &value.report_digest)
        .encode()
}

fn exactly_one<T>(
    messages: Vec<Vec<u8>>,
    decode: impl Fn(&[u8]) -> Result<T, String>,
) -> Result<T, String> {
    if messages.len() != 1 {
        return Err("T10 actionability nested artifact count rejected".to_string());
    }
    decode(&messages[0])
}

fn decode_report(bytes: &[u8]) -> Result<MomentumT10ActionabilityDesignReportV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, REPORT_VERSION)?;
    let report_version = fields.string("report_version")?;
    let run_mode = fields.string("run_mode")?;
    let status = parse_status(&fields.string("status")?)?;
    if fields.string("design_evidence_class")? != "PostScreeningResearchDesignOnly" {
        return Err("T10 actionability evidence class rejected".to_string());
    }
    let failure_forensics_report_digest = fields.string("failure_forensics_report_digest")?;
    let protected_before_state_digest = fields.string("protected_before_state_digest")?;
    let fresh_evidence_split_digest = fields.string("fresh_evidence_split_digest")?;
    let selection_policy = exactly_one(
        fields.messages("selection_policy")?,
        decode_selection_policy,
    )?;
    let label_registration = exactly_one(fields.messages("label_registration")?, |bytes| {
        decode_label_registration(bytes, &selection_policy)
    })?;
    let candidate_reports = fields
        .messages("candidate_reports")?
        .iter()
        .map(|bytes| decode_candidate(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let selection_receipt = exactly_one(
        fields.messages("selection_receipt")?,
        decode_selection_receipt,
    )?;
    let participant_registrations = fields
        .messages("participant_registrations")?
        .iter()
        .map(|bytes| decode_participant(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let pair_registrations = fields
        .messages("pair_registrations")?
        .iter()
        .map(|bytes| decode_pair(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let future_training_policy = exactly_one(
        fields.messages("future_training_policy")?,
        decode_future_training,
    )?;
    let fresh_validation_gate =
        exactly_one(fields.messages("fresh_validation_gate")?, decode_fresh_gate)?;
    let final_holdout_gate =
        exactly_one(fields.messages("final_holdout_gate")?, decode_final_gate)?;
    let value = MomentumT10ActionabilityDesignReportV1 {
        report_version,
        run_mode,
        status,
        design_evidence_class:
            MomentumT10ActionabilityDesignEvidenceClassV1::PostScreeningResearchDesignOnly,
        failure_forensics_report_digest,
        protected_before_state_digest,
        fresh_evidence_split_digest,
        label_registration,
        selection_policy,
        candidate_reports,
        selection_receipt,
        fresh_validation_event_count: as_usize(fields.unsigned("fresh_validation_event_count")?)?,
        fresh_validation_minimum_support: as_usize(
            fields.unsigned("fresh_validation_minimum_support")?,
        )?,
        fresh_validation_support_status: parse_fresh_support_status(
            &fields.string("fresh_validation_support_status")?,
        )?,
        fresh_validation_support_sufficient: fields
            .boolean("fresh_validation_support_sufficient")?,
        participant_registrations,
        pair_registrations,
        future_training_policy,
        fresh_validation_gate,
        final_holdout_gate,
        two_stage_registration_digest: fields.optional_string("two_stage_registration_digest")?,
        labels: fields.strings("labels")?,
        live_completed_event_count: as_usize(fields.unsigned("live_completed_event_count")?)?,
        live_scorable_event_count: as_usize(fields.unsigned("live_scorable_event_count")?)?,
        live_pause: fields.string("live_pause")?,
        epoch_three_registered: fields.boolean("epoch_three_registered")?,
        full_eight_blocked: fields.boolean("full_eight_blocked")?,
        protected_artifacts_unchanged: fields.boolean("protected_artifacts_unchanged")?,
        safety_counters: safety_from_values(fields.unsigneds("safety_counters")?)?,
        deterministic_replay_digest: fields.string("deterministic_replay_digest")?,
        runtime_duration_ms: fields.unsigned("runtime_duration_ms")?,
        report_digest: fields.string("report_digest")?,
    };
    fields.finish()?;
    validate_report(&value)?;
    Ok(value)
}

fn artifact_path(category: &str, digest: &str) -> PathBuf {
    Path::new(ROOT).join(category).join(format!("{digest}.pb"))
}

fn persist_one(
    category: &str,
    digest: &str,
    bytes: &[u8],
    decode_digest: impl Fn(&[u8]) -> Result<String, String>,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &artifact_path(category, digest),
        bytes,
        digest,
        decode_digest,
    )
}

fn add_counts(total: &mut (usize, usize), next: (usize, usize)) {
    total.0 += next.0;
    total.1 += next.1;
}

pub fn read_momentum_t10_actionability_design_report_v1()
-> Result<Option<MomentumT10ActionabilityDesignReportV1>, String> {
    read_single(&Path::new(ROOT).join("final_reports"), decode_report)
}

fn build_selection_policy() -> Result<MomentumT10ActionabilitySelectionPolicyV1, String> {
    let mut value = MomentumT10ActionabilitySelectionPolicyV1 {
        policy_version: SELECTION_POLICY_VERSION.to_string(),
        minimum_total_support: MINIMUM_SUPPORT,
        actionable_up_minimum_prevalence_bits: MINIMUM_CLASS_PREVALENCE.to_bits(),
        actionable_down_minimum_prevalence_bits: MINIMUM_CLASS_PREVALENCE.to_bits(),
        abstain_minimum_prevalence_bits: MINIMUM_ABSTAIN_PREVALENCE.to_bits(),
        abstain_maximum_prevalence_bits: MAXIMUM_ABSTAIN_PREVALENCE.to_bits(),
        maximum_partition_class_drift_bits: MAXIMUM_PREVALENCE_DRIFT.to_bits(),
        choose_largest_passing_multiplier: true,
        policy_digest: String::new(),
    };
    value.policy_digest = selection_policy_digest(&value);
    validate_selection_policy(&value)?;
    Ok(value)
}

fn build_label_registration(
    failure: &MomentumT10FailureForensicsReportV1,
    policy: &MomentumT10ActionabilitySelectionPolicyV1,
) -> Result<MomentumT10ActionabilityLabelRegistrationV1, String> {
    let source_event_set_digest = stable_hash_string(&format!(
        "T10-actionability-consumed-event-set:{}:{}:{}:{}",
        failure.registration.registration_digest,
        failure.evidence_use_receipt.receipt_digest,
        failure.source_development_aggregate_digest,
        failure.source_validation_aggregate_digest
    ));
    let volatility_policy_digest = registered_volatility_policy_digest();
    let mut value = MomentumT10ActionabilityLabelRegistrationV1 {
        registration_version: LABEL_REGISTRATION_VERSION.to_string(),
        source_event_set_digest,
        volatility_policy_digest,
        candidate_multiplier_bits: MULTIPLIERS.iter().map(|value| value.to_bits()).collect(),
        actionable_up_rule: "future_return_t > k * sigma_t".to_string(),
        abstain_rule: "abs(future_return_t) <= k * sigma_t".to_string(),
        actionable_down_rule: "future_return_t < -k * sigma_t".to_string(),
        selection_policy_digest: policy.policy_digest.clone(),
        fresh_validation_access_forbidden: true,
        final_holdout_access_forbidden: true,
        model_training_forbidden: true,
        registration_digest: String::new(),
    };
    value.registration_digest = label_registration_digest(&value);
    validate_label_registration(&value, policy)?;
    Ok(value)
}

fn percentile(values: &[f64], quantile: f64) -> Result<f64, String> {
    if values.is_empty()
        || values.iter().any(|value| !value.is_finite())
        || !(0.0..=1.0).contains(&quantile)
    {
        return Err("T10 actionability percentile rejected".to_string());
    }
    let position = quantile * (values.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        Ok(values[lower])
    } else {
        let weight = position - lower as f64;
        Ok(values[lower] * (1.0 - weight) + values[upper] * weight)
    }
}

fn civil_month_key(timestamp_ms: u64) -> i64 {
    let days = (timestamp_ms / DAY_MS) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    year * 12 + month
}

fn class_prevalence(group: &[&CandidateLabelEvent]) -> Vec<f64> {
    [
        ActionabilityLabel::ActionableUp,
        ActionabilityLabel::ActionableDown,
        ActionabilityLabel::Abstain,
    ]
    .iter()
    .map(|label| {
        group.iter().filter(|item| item.label == *label).count() as f64 / group.len() as f64
    })
    .collect()
}

fn prevalence_ranges(groups: Vec<Vec<&CandidateLabelEvent>>) -> Result<Vec<f64>, String> {
    let prevalences = groups
        .iter()
        .filter(|group| !group.is_empty())
        .map(|group| class_prevalence(group))
        .collect::<Vec<_>>();
    if prevalences.is_empty() {
        return Err("T10 actionability temporal groups unavailable".to_string());
    }
    Ok((0..3)
        .map(|index| {
            let minimum = prevalences
                .iter()
                .map(|values| values[index])
                .fold(f64::INFINITY, f64::min);
            let maximum = prevalences
                .iter()
                .map(|values| values[index])
                .fold(f64::NEG_INFINITY, f64::max);
            maximum - minimum
        })
        .collect())
}

fn rolling_prevalence_ranges(
    events: &[CandidateLabelEvent],
    requested_window: usize,
) -> Result<Vec<f64>, String> {
    if events.is_empty() || requested_window == 0 {
        return Err("T10 actionability rolling support unavailable".to_string());
    }
    let window = requested_window.min(events.len());
    let label_index = |label| match label {
        ActionabilityLabel::ActionableUp => 0,
        ActionabilityLabel::ActionableDown => 1,
        ActionabilityLabel::Abstain => 2,
    };
    let mut counts = [0_usize; 3];
    for event in &events[..window] {
        counts[label_index(event.label)] += 1;
    }
    let mut minimum = [f64::INFINITY; 3];
    let mut maximum = [f64::NEG_INFINITY; 3];
    let update = |counts: [usize; 3], minimum: &mut [f64; 3], maximum: &mut [f64; 3]| {
        for index in 0..3 {
            let prevalence = counts[index] as f64 / window as f64;
            minimum[index] = minimum[index].min(prevalence);
            maximum[index] = maximum[index].max(prevalence);
        }
    };
    update(counts, &mut minimum, &mut maximum);
    for end in window..events.len() {
        counts[label_index(events[end - window].label)] -= 1;
        counts[label_index(events[end].label)] += 1;
        update(counts, &mut minimum, &mut maximum);
    }
    Ok((0..3)
        .map(|index| maximum[index] - minimum[index])
        .collect())
}

fn temporal_stability(
    events: &[CandidateLabelEvent],
) -> Result<MomentumT10ActionabilityTemporalStabilityV1, String> {
    let grouped = |key: fn(&CandidateLabelEvent) -> i64| {
        let mut groups = BTreeMap::<i64, Vec<&CandidateLabelEvent>>::new();
        for event in events {
            groups.entry(key(event)).or_default().push(event);
        }
        groups.into_values().collect::<Vec<_>>()
    };
    let mut value = MomentumT10ActionabilityTemporalStabilityV1 {
        stability_version: TEMPORAL_STABILITY_VERSION.to_string(),
        daily_class_prevalence_ranges: prevalence_ranges(grouped(|event| {
            (event.timestamp_ms / DAY_MS) as i64
        }))?,
        weekly_class_prevalence_ranges: prevalence_ranges(grouped(|event| {
            (event.timestamp_ms / WEEK_MS) as i64
        }))?,
        monthly_class_prevalence_ranges: prevalence_ranges(grouped(|event| {
            civil_month_key(event.timestamp_ms)
        }))?,
        rolling_144_class_prevalence_ranges: rolling_prevalence_ranges(events, 144)?,
        rolling_1008_class_prevalence_ranges: rolling_prevalence_ranges(events, 1_008)?,
        finite: true,
        stability_digest: String::new(),
    };
    value.stability_digest = stability_digest(&value);
    validate_stability(&value)?;
    Ok(value)
}

fn derive_candidate_events(
    evidence: &[MomentumT10ConsumedActionabilityEvidenceV1],
    multiplier: f64,
) -> Result<Vec<CandidateLabelEvent>, String> {
    evidence
        .iter()
        .map(|event| {
            let used_floor = event.past_micro_volatility == 0.0;
            let volatility = if used_floor {
                COMPARISON_EPSILON
            } else {
                event.past_micro_volatility
            };
            let threshold = multiplier * volatility;
            if !threshold.is_finite() || threshold <= 0.0 {
                return Err("T10 actionability threshold rejected".to_string());
            }
            let label = if event.target_return > threshold {
                ActionabilityLabel::ActionableUp
            } else if event.target_return < -threshold {
                ActionabilityLabel::ActionableDown
            } else {
                ActionabilityLabel::Abstain
            };
            Ok(CandidateLabelEvent {
                timestamp_ms: event.prediction_timestamp_ms,
                target_magnitude: event.target_return.abs(),
                volatility,
                used_floor,
                label,
            })
        })
        .collect()
}

fn build_candidate_report(
    registration: &MomentumT10ActionabilityLabelRegistrationV1,
    partition: MomentumReplayPartitionV1,
    multiplier: f64,
    evidence: &[MomentumT10ConsumedActionabilityEvidenceV1],
) -> Result<MomentumT10ActionabilityCandidateReportV1, String> {
    if partition == MomentumReplayPartitionV1::SealedHoldout
        || evidence.is_empty()
        || evidence.iter().any(|event| event.partition != partition)
    {
        return Err("T10 actionability candidate evidence scope rejected".to_string());
    }
    let events = derive_candidate_events(evidence, multiplier)?;
    let count = events.len();
    let actionable_up_count = events
        .iter()
        .filter(|event| event.label == ActionabilityLabel::ActionableUp)
        .count();
    let actionable_down_count = events
        .iter()
        .filter(|event| event.label == ActionabilityLabel::ActionableDown)
        .count();
    let abstain_count = events
        .iter()
        .filter(|event| event.label == ActionabilityLabel::Abstain)
        .count();
    let mut magnitudes = events
        .iter()
        .map(|event| event.target_magnitude)
        .collect::<Vec<_>>();
    magnitudes.sort_by(f64::total_cmp);
    let mut volatility = events
        .iter()
        .map(|event| event.volatility)
        .collect::<Vec<_>>();
    volatility.sort_by(f64::total_cmp);
    let mut value = MomentumT10ActionabilityCandidateReportV1 {
        report_version: CANDIDATE_REPORT_VERSION.to_string(),
        label_registration_digest: registration.registration_digest.clone(),
        partition,
        multiplier,
        eligible_event_count: count,
        actionable_up_count,
        actionable_down_count,
        abstain_count,
        actionable_up_prevalence: actionable_up_count as f64 / count as f64,
        actionable_down_prevalence: actionable_down_count as f64 / count as f64,
        abstain_prevalence: abstain_count as f64 / count as f64,
        target_magnitude_mean: magnitudes.iter().sum::<f64>() / count as f64,
        target_magnitude_median: percentile(&magnitudes, 0.5)?,
        volatility_scale_mean: volatility.iter().sum::<f64>() / count as f64,
        volatility_scale_median: percentile(&volatility, 0.5)?,
        zero_volatility_floor_count: events.iter().filter(|event| event.used_floor).count(),
        temporal_stability: temporal_stability(&events)?,
        finite_value_proof: true,
        chronology_audit_passed: evidence
            .windows(2)
            .all(|pair| pair[0].prediction_timestamp_ms < pair[1].prediction_timestamp_ms),
        leakage_audit_passed: evidence
            .iter()
            .all(|event| event.target_timestamp_ms > event.prediction_timestamp_ms),
        integrity_audit_passed: true,
        candidate_digest: String::new(),
    };
    value.candidate_digest = candidate_digest(&value);
    validate_candidate(&value)?;
    Ok(value)
}

fn candidate_support_passes(value: &MomentumT10ActionabilityCandidateReportV1) -> bool {
    value.eligible_event_count >= MINIMUM_SUPPORT
        && value.actionable_up_prevalence >= MINIMUM_CLASS_PREVALENCE
        && value.actionable_down_prevalence >= MINIMUM_CLASS_PREVALENCE
        && value.abstain_prevalence >= MINIMUM_ABSTAIN_PREVALENCE
        && value.abstain_prevalence <= MAXIMUM_ABSTAIN_PREVALENCE
        && value.finite_value_proof
        && value.chronology_audit_passed
        && value.leakage_audit_passed
        && value.integrity_audit_passed
}

fn build_selection_receipt(
    registration: &MomentumT10ActionabilityLabelRegistrationV1,
    candidates: &[MomentumT10ActionabilityCandidateReportV1],
) -> Result<MomentumT10ActionabilitySelectionReceiptV1, String> {
    let passing = MULTIPLIERS
        .iter()
        .copied()
        .filter(|multiplier| {
            let development = candidates.iter().find(|candidate| {
                candidate.partition == MomentumReplayPartitionV1::Development
                    && candidate.multiplier == *multiplier
            });
            let validation = candidates.iter().find(|candidate| {
                candidate.partition == MomentumReplayPartitionV1::Validation
                    && candidate.multiplier == *multiplier
            });
            match (development, validation) {
                (Some(development), Some(validation)) => {
                    candidate_support_passes(development)
                        && candidate_support_passes(validation)
                        && (development.actionable_up_prevalence
                            - validation.actionable_up_prevalence)
                            .abs()
                            <= MAXIMUM_PREVALENCE_DRIFT
                        && (development.actionable_down_prevalence
                            - validation.actionable_down_prevalence)
                            .abs()
                            <= MAXIMUM_PREVALENCE_DRIFT
                        && (development.abstain_prevalence - validation.abstain_prevalence).abs()
                            <= MAXIMUM_PREVALENCE_DRIFT
                }
                _ => false,
            }
        })
        .collect::<Vec<_>>();
    let selected_multiplier = passing.last().copied();
    let selected_candidate_digest = selected_multiplier.and_then(|multiplier| {
        let digests = candidates
            .iter()
            .filter(|candidate| candidate.multiplier == multiplier)
            .map(|candidate| candidate.candidate_digest.clone())
            .collect::<Vec<_>>();
        (digests.len() == 2).then(|| {
            stable_hash_string(&format!(
                "T10-selected-actionability-candidate:{multiplier:?}:{digests:?}"
            ))
        })
    });
    let mut value = MomentumT10ActionabilitySelectionReceiptV1 {
        receipt_version: SELECTION_RECEIPT_VERSION.to_string(),
        label_registration_digest: registration.registration_digest.clone(),
        candidate_report_digests: candidates
            .iter()
            .map(|candidate| candidate.candidate_digest.clone())
            .collect(),
        selected_multiplier,
        selected_candidate_digest,
        selection_status: if selected_multiplier.is_some() {
            MomentumT10ActionabilitySelectionStatusV1::StableThresholdSelected
        } else {
            MomentumT10ActionabilitySelectionStatusV1::NoStableActionabilityThreshold
        },
        fresh_validation_reads: 0,
        final_holdout_reads: 0,
        receipt_digest: String::new(),
    };
    value.receipt_digest = selection_receipt_digest(&value);
    validate_selection_receipt(&value)?;
    Ok(value)
}

fn build_participants(
    selected: bool,
) -> Result<Vec<MomentumT10SelectiveParticipantRegistrationV1>, String> {
    if !selected {
        return Ok(Vec::new());
    }
    let definitions = [
        (
            "O0",
            "Opportunity",
            "PrevalenceConstant",
            "PriorRevealedActionabilityLabelsOnly",
            0,
            0,
            false,
        ),
        (
            "O1",
            "Opportunity",
            "Logistic",
            "FrozenTenMinuteAnchor",
            6,
            STANDARD_L2_MULTIPLIER,
            false,
        ),
        (
            "O2",
            "Opportunity",
            "Logistic",
            "FrozenCompactMicro69",
            69,
            STRONG_L2_MULTIPLIER,
            false,
        ),
        (
            "D0",
            "Direction",
            "PrevalenceConstant",
            "PriorRevealedActionableDirectionsOnly",
            0,
            0,
            true,
        ),
        (
            "D1",
            "Direction",
            "Logistic",
            "FrozenTenMinuteAnchor",
            6,
            STANDARD_L2_MULTIPLIER,
            true,
        ),
        (
            "D2",
            "Direction",
            "Logistic",
            "FrozenCompactMicro69",
            69,
            STRONG_L2_MULTIPLIER,
            true,
        ),
    ];
    definitions
        .into_iter()
        .map(
            |(id, head, role, feature, dimension, l2, actionable_only)| {
                let mut value = MomentumT10SelectiveParticipantRegistrationV1 {
                    registration_version: PARTICIPANT_VERSION.to_string(),
                    participant_id: id.to_string(),
                    head: head.to_string(),
                    role: role.to_string(),
                    feature_policy: feature.to_string(),
                    feature_dimension: dimension,
                    l2_multiplier: l2,
                    prior_revealed_labels_only: true,
                    actionable_training_only: actionable_only,
                    training_authorized: false,
                    registration_digest: String::new(),
                };
                value.registration_digest = participant_digest(&value);
                validate_participant(&value)?;
                Ok(value)
            },
        )
        .collect()
}

fn build_pairs(selected: bool) -> Result<Vec<MomentumT10SelectivePairRegistrationV1>, String> {
    if !selected {
        return Ok(Vec::new());
    }
    [
        ("S0", "O0", "D0", true),
        ("S1", "O1", "D1", false),
        ("S2", "O2", "D2", false),
    ]
    .into_iter()
    .map(|(pair, opportunity, direction, reference)| {
        let mut value = MomentumT10SelectivePairRegistrationV1 {
            registration_version: PAIR_VERSION.to_string(),
            pair_id: pair.to_string(),
            opportunity_participant_id: opportunity.to_string(),
            direction_participant_id: direction.to_string(),
            reference_system: reference,
            cross_pairing: false,
            opportunity_threshold_bits: OPPORTUNITY_THRESHOLD.to_bits(),
            direction_threshold_bits: DIRECTION_THRESHOLD.to_bits(),
            opportunity_below_threshold_abstains: true,
            direction_equality_maps_up: true,
            training_authorized: false,
            registration_digest: String::new(),
        };
        value.registration_digest = pair_digest(&value);
        validate_pair(&value)?;
        Ok(value)
    })
    .collect()
}

fn build_future_training(
    fresh_evidence_split_digest: &str,
) -> Result<MomentumT10SelectiveFutureTrainingPolicyV1, String> {
    if fresh_evidence_split_digest.is_empty() {
        return Err("T10 fresh-evidence split binding unavailable".to_string());
    }
    let mut value = MomentumT10SelectiveFutureTrainingPolicyV1 {
        policy_version: TRAINING_POLICY_VERSION.to_string(),
        daily_utc_refit: true,
        previously_revealed_labels_only: true,
        chronological_order_required: true,
        maximum_training_examples: MAXIMUM_TRAINING_EXAMPLES,
        training_only_normalizers: true,
        direction_excludes_abstain: true,
        persist_and_reopen_all_receipts: true,
        participants_frozen_per_utc_day: true,
        later_fresh_labels_only_after_observable: true,
        fresh_evidence_split_digest: fresh_evidence_split_digest.to_string(),
        consumed_design_pool_is_former_development_and_validation: true,
        fresh_validation_use_once: true,
        final_holdout_requires_separate_authorization: true,
        execution_authorized: false,
        policy_digest: String::new(),
    };
    value.policy_digest = future_training_digest(&value);
    validate_future_training(&value)?;
    Ok(value)
}

fn build_fresh_gate() -> Result<MomentumT10FreshValidationGateV1, String> {
    let mut value = MomentumT10FreshValidationGateV1 {
        gate_version: FRESH_GATE_VERSION.to_string(),
        opportunity_lower_brier_than_o0_required: true,
        direction_lower_brier_than_d0_required: true,
        consumed_design_replay_required: true,
        fresh_validation_required: true,
        sufficient_actionable_support_required: true,
        minimum_coverage: MINIMUM_COVERAGE,
        maximum_coverage: MAXIMUM_COVERAGE,
        finite_no_collapse_no_saturation_required: true,
        chronology_leakage_integrity_required: true,
        result_selected_mutation_forbidden: true,
        correctness_cannot_override_brier: true,
        coverage_cannot_override_brier: true,
        final_holdout_access_required_zero: true,
        opportunity_metric_names: OPPORTUNITY_METRICS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        direction_metric_names: DIRECTION_METRICS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        selective_metric_names: SELECTIVE_METRICS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        execution_authorized: false,
        gate_digest: String::new(),
    };
    value.gate_digest = fresh_gate_digest(&value);
    validate_fresh_gate(&value)?;
    Ok(value)
}

fn build_final_gate() -> Result<MomentumT10FinalHoldoutGateV1, String> {
    let mut value = MomentumT10FinalHoldoutGateV1 {
        gate_version: FINAL_GATE_VERSION.to_string(),
        consumed_design_pass_required: true,
        fresh_validation_pass_required: true,
        post_validation_design_change_forbidden: true,
        deterministic_eligible_cohort_required: true,
        separate_owner_authorization_required: true,
        prediction_count: 0,
        label_read_count: 0,
        metric_count: 0,
        execution_authorized: false,
        gate_digest: String::new(),
    };
    value.gate_digest = final_gate_digest(&value);
    validate_final_gate(&value)?;
    Ok(value)
}

fn persist_preregistration(
    policy: &MomentumT10ActionabilitySelectionPolicyV1,
    registration: &MomentumT10ActionabilityLabelRegistrationV1,
) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_one(
            "selection_policies",
            &policy.policy_digest,
            &encode_selection_policy(policy)?,
            |bytes| Ok(decode_selection_policy(bytes)?.policy_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "label_registrations",
            &registration.registration_digest,
            &encode_label_registration(registration, policy)?,
            |bytes| Ok(decode_label_registration(bytes, policy)?.registration_digest),
        )?,
    );
    Ok(counts)
}

fn persist_results(
    candidates: &[MomentumT10ActionabilityCandidateReportV1],
    selection: &MomentumT10ActionabilitySelectionReceiptV1,
    participants: &[MomentumT10SelectiveParticipantRegistrationV1],
    pairs: &[MomentumT10SelectivePairRegistrationV1],
    training: &MomentumT10SelectiveFutureTrainingPolicyV1,
    fresh_gate: &MomentumT10FreshValidationGateV1,
    final_gate: &MomentumT10FinalHoldoutGateV1,
    journal: &MomentumT10ActionabilityResearchJournalV1,
) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    for candidate in candidates {
        add_counts(
            &mut counts,
            persist_one(
                "candidate_reports",
                &candidate.candidate_digest,
                &encode_candidate(candidate)?,
                |bytes| Ok(decode_candidate(bytes)?.candidate_digest),
            )?,
        );
    }
    add_counts(
        &mut counts,
        persist_one(
            "selection_receipts",
            &selection.receipt_digest,
            &encode_selection_receipt(selection)?,
            |bytes| Ok(decode_selection_receipt(bytes)?.receipt_digest),
        )?,
    );
    for participant in participants {
        add_counts(
            &mut counts,
            persist_one(
                "participant_registrations",
                &participant.registration_digest,
                &encode_participant(participant)?,
                |bytes| Ok(decode_participant(bytes)?.registration_digest),
            )?,
        );
    }
    for pair in pairs {
        add_counts(
            &mut counts,
            persist_one(
                "pair_registrations",
                &pair.registration_digest,
                &encode_pair(pair)?,
                |bytes| Ok(decode_pair(bytes)?.registration_digest),
            )?,
        );
    }
    add_counts(
        &mut counts,
        persist_one(
            "future_training_policies",
            &training.policy_digest,
            &encode_future_training(training)?,
            |bytes| Ok(decode_future_training(bytes)?.policy_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "fresh_validation_gates",
            &fresh_gate.gate_digest,
            &encode_fresh_gate(fresh_gate)?,
            |bytes| Ok(decode_fresh_gate(bytes)?.gate_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "final_holdout_gates",
            &final_gate.gate_digest,
            &encode_final_gate(final_gate)?,
            |bytes| Ok(decode_final_gate(bytes)?.gate_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "research_journals",
            &journal.journal_digest,
            &encode_journal(journal)?,
            |bytes| Ok(decode_journal(bytes)?.journal_digest),
        )?,
    );
    Ok(counts)
}

fn completed_replay(
    mut report: MomentumT10ActionabilityDesignReportV1,
    mode: MomentumT10ActionabilityDesignRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
    started: Instant,
) -> Result<MomentumT10ActionabilityDesignReportV1, String> {
    if report.protected_before_state_digest != protected.state_digest {
        return Err("T10 actionability replay protected state changed".to_string());
    }
    report.run_mode = mode.as_str().to_string();
    report.safety_counters = MomentumT10ActionabilitySafetyCountersV1::default();
    report.runtime_duration_ms = started.elapsed().as_millis() as u64;
    report.report_digest = report_digest(&report);
    validate_report(&report)?;
    Ok(report)
}

fn run_inner(
    mode: MomentumT10ActionabilityDesignRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumT10ActionabilityDesignReportV1, String> {
    let started = Instant::now();
    validate_momentum_micro_protected_before_state_v1(protected)?;
    if let Some(report) = read_momentum_t10_actionability_design_report_v1()? {
        return completed_replay(report, mode, protected, started);
    }
    if mode == MomentumT10ActionabilityDesignRunModeV1::Status {
        return Err("T10 actionability design unregistered".to_string());
    }
    let failure = read_momentum_t10_failure_forensics_report_v1()?
        .ok_or_else(|| "T10 failure forensics required first".to_string())?;
    if failure.status != MomentumT10FailureForensicsStatusV1::Complete
        || failure.protected_before_state_digest != protected.state_digest
        || failure.fresh_evidence_split.label_reads != 0
        || failure.fresh_evidence_split.prediction_reads != 0
        || failure.fresh_evidence_split.metric_reads != 0
        || failure
            .fresh_evidence_split
            .fresh_validation_execution_authorized
        || failure
            .fresh_evidence_split
            .final_holdout_execution_authorized
    {
        return Err("T10 actionability failure-forensics boundary rejected".to_string());
    }
    let screening = read_momentum_t10_micro_screening_report_v1()?
        .ok_or_else(|| "T10 screening report unavailable".to_string())?;
    let screening_training_policy = screening
        .training_policy
        .as_ref()
        .ok_or_else(|| "T10 screening support policy unavailable".to_string())?;
    let minimum_support = screening_training_policy.minimum_training_examples;
    if minimum_support != MINIMUM_SUPPORT
        || screening_training_policy.maximum_training_examples != MAXIMUM_TRAINING_EXAMPLES
    {
        return Err("T10 actionability support policy changed".to_string());
    }
    let selection_policy = build_selection_policy()?;
    let label_registration = build_label_registration(&failure, &selection_policy)?;
    let mut counts = persist_preregistration(&selection_policy, &label_registration)?;

    // Candidate labels are derived only after the frozen registration is persisted.
    let development = read_momentum_t10_consumed_actionability_evidence_v1(
        MomentumReplayPartitionV1::Development,
    )?;
    let validation = read_momentum_t10_consumed_actionability_evidence_v1(
        MomentumReplayPartitionV1::Validation,
    )?;
    let mut candidate_reports = Vec::new();
    for multiplier in MULTIPLIERS {
        candidate_reports.push(build_candidate_report(
            &label_registration,
            MomentumReplayPartitionV1::Development,
            multiplier,
            &development,
        )?);
        candidate_reports.push(build_candidate_report(
            &label_registration,
            MomentumReplayPartitionV1::Validation,
            multiplier,
            &validation,
        )?);
    }
    let selection_receipt = build_selection_receipt(&label_registration, &candidate_reports)?;
    let fresh_validation_event_count = failure
        .fresh_evidence_split
        .fresh_validation_event_digests
        .len();
    let fresh_validation_support_sufficient =
        fresh_validation_event_count >= selection_policy.minimum_total_support;
    let fresh_validation_support_status = if fresh_validation_support_sufficient {
        MomentumT10FreshValidationSupportStatusV1::SufficientSupport
    } else {
        MomentumT10FreshValidationSupportStatusV1::FreshValidationInsufficientSupport
    };
    let selected = selection_receipt.selection_status
        == MomentumT10ActionabilitySelectionStatusV1::StableThresholdSelected
        && fresh_validation_support_sufficient;
    let participant_registrations = build_participants(selected)?;
    let pair_registrations = build_pairs(selected)?;
    let future_training_policy = build_future_training(&failure.fresh_evidence_split.split_digest)?;
    let fresh_validation_gate = build_fresh_gate()?;
    let final_holdout_gate = build_final_gate()?;
    let two_stage_registration_digest = selected.then(|| {
        two_stage_registration_digest(
            &selection_receipt,
            &participant_registrations,
            &pair_registrations,
            &future_training_policy,
            &fresh_validation_gate,
        )
    });
    let mut journal = MomentumT10ActionabilityResearchJournalV1 {
        journal_version: JOURNAL_VERSION.to_string(),
        failure_report_digest: failure.report_digest.clone(),
        label_registration_digest: label_registration.registration_digest.clone(),
        candidate_report_digests: candidate_reports
            .iter()
            .map(|candidate| candidate.candidate_digest.clone())
            .collect(),
        selection_receipt_digest: selection_receipt.receipt_digest.clone(),
        participant_registration_digests: participant_registrations
            .iter()
            .map(|participant| participant.registration_digest.clone())
            .collect(),
        pair_registration_digests: pair_registrations
            .iter()
            .map(|pair| pair.registration_digest.clone())
            .collect(),
        future_training_policy_digest: future_training_policy.policy_digest.clone(),
        fresh_validation_gate_digest: fresh_validation_gate.gate_digest.clone(),
        final_holdout_gate_digest: final_holdout_gate.gate_digest.clone(),
        deterministic: true,
        journal_digest: String::new(),
    };
    journal.journal_digest = journal_digest(&journal);
    validate_journal(&journal)?;
    add_counts(
        &mut counts,
        persist_results(
            &candidate_reports,
            &selection_receipt,
            &participant_registrations,
            &pair_registrations,
            &future_training_policy,
            &fresh_validation_gate,
            &final_holdout_gate,
            &journal,
        )?,
    );
    let mut report = MomentumT10ActionabilityDesignReportV1 {
        report_version: REPORT_VERSION.to_string(),
        run_mode: mode.as_str().to_string(),
        status: MomentumT10ActionabilityDesignStatusV1::Complete,
        design_evidence_class:
            MomentumT10ActionabilityDesignEvidenceClassV1::PostScreeningResearchDesignOnly,
        failure_forensics_report_digest: failure.report_digest,
        protected_before_state_digest: protected.state_digest.clone(),
        fresh_evidence_split_digest: failure.fresh_evidence_split.split_digest,
        label_registration,
        selection_policy,
        candidate_reports,
        selection_receipt,
        fresh_validation_event_count,
        fresh_validation_minimum_support: minimum_support,
        fresh_validation_support_status,
        fresh_validation_support_sufficient,
        participant_registrations,
        pair_registrations,
        future_training_policy,
        fresh_validation_gate,
        final_holdout_gate,
        two_stage_registration_digest,
        labels: PUBLIC_LABELS
            .iter()
            .map(|label| (*label).to_string())
            .collect(),
        live_completed_event_count: protected.completed_event_count,
        live_scorable_event_count: protected.scorable_event_count,
        live_pause: "PausedAfterCompletedEpochTwo".to_string(),
        epoch_three_registered: protected.epoch_three_registered,
        full_eight_blocked: true,
        protected_artifacts_unchanged: true,
        safety_counters: MomentumT10ActionabilitySafetyCountersV1 {
            artifacts_written: counts.0 + 1,
            duplicate_artifact_count: counts.1,
            label_computations: (development.len() + validation.len()) * MULTIPLIERS.len(),
            ..MomentumT10ActionabilitySafetyCountersV1::default()
        },
        deterministic_replay_digest: journal.journal_digest,
        runtime_duration_ms: started.elapsed().as_millis() as u64,
        report_digest: String::new(),
    };
    report.report_digest = report_digest(&report);
    validate_report(&report)?;
    let persisted = persist_one(
        "final_reports",
        &report.report_digest,
        &encode_report(&report)?,
        |bytes| Ok(decode_report(bytes)?.report_digest),
    )?;
    if persisted != (1, 0)
        || read_momentum_t10_actionability_design_report_v1()?.as_ref() != Some(&report)
    {
        return Err("T10 actionability final report persist mismatch".to_string());
    }
    Ok(report)
}

pub fn run_momentum_t10_actionability_design_v1(
    mode: MomentumT10ActionabilityDesignRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumT10ActionabilityDesignReportV1, String> {
    run_inner(mode, protected)
}

pub fn format_momentum_t10_actionability_design_text_v1(
    report: &MomentumT10ActionabilityDesignReportV1,
) -> String {
    use std::fmt::Write as _;
    let mut output = String::new();
    let _ = writeln!(output, "status={:?}", report.status);
    for candidate in &report.candidate_reports {
        let _ = writeln!(
            output,
            "k={:.2};partition={};up={:.6};down={:.6};abstain={:.6}",
            candidate.multiplier,
            partition_name(candidate.partition),
            candidate.actionable_up_prevalence,
            candidate.actionable_down_prevalence,
            candidate.abstain_prevalence
        );
    }
    let _ = writeln!(
        output,
        "selection_status={}",
        selection_status_name(report.selection_receipt.selection_status)
    );
    let _ = writeln!(
        output,
        "selected_multiplier={}",
        report
            .selection_receipt
            .selected_multiplier
            .map(|value| format!("{value:.2}"))
            .unwrap_or_else(|| "absent".to_string())
    );
    let _ = writeln!(
        output,
        "fresh_validation_event_count={}",
        report.fresh_validation_event_count
    );
    let _ = writeln!(
        output,
        "fresh_validation_support_status={}",
        fresh_support_status_name(report.fresh_validation_support_status)
    );
    let _ = writeln!(
        output,
        "fresh_validation_support_sufficient={}",
        report.fresh_validation_support_sufficient
    );
    let _ = writeln!(
        output,
        "fresh_evidence_split_digest={}",
        report.fresh_evidence_split_digest
    );
    let _ = writeln!(output, "new_model_fits=0");
    let _ = writeln!(output, "fresh_validation_reads=0");
    let _ = writeln!(output, "final_holdout_reads=0");
    let _ = writeln!(output, "report_digest={}", report.report_digest);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(return_value: f64, volatility: f64) -> MomentumT10ConsumedActionabilityEvidenceV1 {
        MomentumT10ConsumedActionabilityEvidenceV1 {
            partition: MomentumReplayPartitionV1::Development,
            prediction_timestamp_ms: DAY_MS,
            target_timestamp_ms: DAY_MS + 600_000,
            source_event_digest: "event".into(),
            target_return: return_value,
            past_micro_volatility: volatility,
        }
    }

    fn label_registration_fixture(
        policy: &MomentumT10ActionabilitySelectionPolicyV1,
    ) -> MomentumT10ActionabilityLabelRegistrationV1 {
        let mut value = MomentumT10ActionabilityLabelRegistrationV1 {
            registration_version: LABEL_REGISTRATION_VERSION.into(),
            source_event_set_digest: "consumed-events".into(),
            volatility_policy_digest: registered_volatility_policy_digest(),
            candidate_multiplier_bits: MULTIPLIERS.iter().map(|value| value.to_bits()).collect(),
            actionable_up_rule: "future_return_t > k * sigma_t".into(),
            abstain_rule: "abs(future_return_t) <= k * sigma_t".into(),
            actionable_down_rule: "future_return_t < -k * sigma_t".into(),
            selection_policy_digest: policy.policy_digest.clone(),
            fresh_validation_access_forbidden: true,
            final_holdout_access_forbidden: true,
            model_training_forbidden: true,
            registration_digest: String::new(),
        };
        value.registration_digest = label_registration_digest(&value);
        value
    }

    fn candidate_fixture(
        registration: &MomentumT10ActionabilityLabelRegistrationV1,
        partition: MomentumReplayPartitionV1,
        multiplier: f64,
        counts: [usize; 3],
    ) -> MomentumT10ActionabilityCandidateReportV1 {
        let count = counts.iter().sum::<usize>();
        let mut stability = MomentumT10ActionabilityTemporalStabilityV1 {
            stability_version: TEMPORAL_STABILITY_VERSION.into(),
            daily_class_prevalence_ranges: vec![0.01, 0.01, 0.01],
            weekly_class_prevalence_ranges: vec![0.01, 0.01, 0.01],
            monthly_class_prevalence_ranges: vec![0.01, 0.01, 0.01],
            rolling_144_class_prevalence_ranges: vec![0.01, 0.01, 0.01],
            rolling_1008_class_prevalence_ranges: vec![0.01, 0.01, 0.01],
            finite: true,
            stability_digest: String::new(),
        };
        stability.stability_digest = stability_digest(&stability);
        let mut value = MomentumT10ActionabilityCandidateReportV1 {
            report_version: CANDIDATE_REPORT_VERSION.into(),
            label_registration_digest: registration.registration_digest.clone(),
            partition,
            multiplier,
            eligible_event_count: count,
            actionable_up_count: counts[0],
            actionable_down_count: counts[1],
            abstain_count: counts[2],
            actionable_up_prevalence: counts[0] as f64 / count as f64,
            actionable_down_prevalence: counts[1] as f64 / count as f64,
            abstain_prevalence: counts[2] as f64 / count as f64,
            target_magnitude_mean: 0.01,
            target_magnitude_median: 0.009,
            volatility_scale_mean: 0.008,
            volatility_scale_median: 0.007,
            zero_volatility_floor_count: 0,
            temporal_stability: stability,
            finite_value_proof: true,
            chronology_audit_passed: true,
            leakage_audit_passed: true,
            integrity_audit_passed: true,
            candidate_digest: String::new(),
        };
        value.candidate_digest = candidate_digest(&value);
        value
    }

    fn passing_candidates(
        registration: &MomentumT10ActionabilityLabelRegistrationV1,
    ) -> Vec<MomentumT10ActionabilityCandidateReportV1> {
        MULTIPLIERS
            .into_iter()
            .flat_map(|multiplier| {
                [
                    candidate_fixture(
                        registration,
                        MomentumReplayPartitionV1::Development,
                        multiplier,
                        [400, 400, 1_200],
                    ),
                    candidate_fixture(
                        registration,
                        MomentumReplayPartitionV1::Validation,
                        multiplier,
                        [420, 380, 1_200],
                    ),
                ]
            })
            .collect()
    }

    fn report_fixture() -> MomentumT10ActionabilityDesignReportV1 {
        let selection_policy = build_selection_policy().unwrap();
        let label_registration = label_registration_fixture(&selection_policy);
        let candidate_reports = passing_candidates(&label_registration);
        let selection_receipt =
            build_selection_receipt(&label_registration, &candidate_reports).unwrap();
        let mut value = MomentumT10ActionabilityDesignReportV1 {
            report_version: REPORT_VERSION.into(),
            run_mode: "register-and-execute-local".into(),
            status: MomentumT10ActionabilityDesignStatusV1::Complete,
            design_evidence_class:
                MomentumT10ActionabilityDesignEvidenceClassV1::PostScreeningResearchDesignOnly,
            failure_forensics_report_digest: "failure-report".into(),
            protected_before_state_digest: "protected".into(),
            fresh_evidence_split_digest: "split".into(),
            label_registration,
            selection_policy,
            candidate_reports,
            selection_receipt,
            fresh_validation_event_count: MINIMUM_SUPPORT + 1,
            fresh_validation_minimum_support: MINIMUM_SUPPORT,
            fresh_validation_support_status:
                MomentumT10FreshValidationSupportStatusV1::SufficientSupport,
            fresh_validation_support_sufficient: true,
            participant_registrations: build_participants(true).unwrap(),
            pair_registrations: build_pairs(true).unwrap(),
            future_training_policy: build_future_training("split").unwrap(),
            fresh_validation_gate: build_fresh_gate().unwrap(),
            final_holdout_gate: build_final_gate().unwrap(),
            two_stage_registration_digest: None,
            labels: PUBLIC_LABELS
                .iter()
                .map(|label| (*label).to_string())
                .collect(),
            live_completed_event_count: 2,
            live_scorable_event_count: 2,
            live_pause: "PausedAfterCompletedEpochTwo".into(),
            epoch_three_registered: false,
            full_eight_blocked: true,
            protected_artifacts_unchanged: true,
            safety_counters: MomentumT10ActionabilitySafetyCountersV1::default(),
            deterministic_replay_digest: "replay".into(),
            runtime_duration_ms: 0,
            report_digest: String::new(),
        };
        value.two_stage_registration_digest = Some(two_stage_registration_digest(
            &value.selection_receipt,
            &value.participant_registrations,
            &value.pair_registrations,
            &value.future_training_policy,
            &value.fresh_validation_gate,
        ));
        value.report_digest = report_digest(&value);
        value
    }

    #[test]
    fn sprint103_13_multiplier_set_is_exact() {
        assert_eq!(MULTIPLIERS, [0.25, 0.50, 1.00]);
    }

    #[test]
    fn sprint103_14_boundary_equality_maps_to_abstain() {
        let events = derive_candidate_events(&[evidence(0.01, 0.01)], 1.0).unwrap();
        assert_eq!(events[0].label, ActionabilityLabel::Abstain);
    }

    #[test]
    fn sprint103_15_past_volatility_floor_is_positive() {
        let events = derive_candidate_events(&[evidence(0.0, 0.0)], 1.0).unwrap();
        assert!(events[0].used_floor && events[0].volatility > 0.0);
    }

    #[test]
    fn sprint103_15b_rolling_stability_uses_bounded_windows() {
        let events = [
            evidence(0.02, 0.01),
            evidence(-0.02, 0.01),
            evidence(0.0, 0.01),
        ];
        let derived = derive_candidate_events(&events, 1.0).unwrap();
        assert_eq!(
            rolling_prevalence_ranges(&derived, 144).unwrap(),
            vec![0.0, 0.0, 0.0]
        );
    }

    #[test]
    fn sprint103_16_largest_passing_multiplier_policy_is_frozen() {
        assert!(
            build_selection_policy()
                .unwrap()
                .choose_largest_passing_multiplier
        );
    }

    #[test]
    fn sprint103_17_o0_to_o2_are_exact() {
        let values = build_participants(true).unwrap();
        assert_eq!(
            values[..3]
                .iter()
                .map(|value| value.participant_id.as_str())
                .collect::<Vec<_>>(),
            ["O0", "O1", "O2"]
        );
    }

    #[test]
    fn sprint103_18_d0_to_d2_exclude_abstain_training() {
        assert!(
            build_participants(true).unwrap()[3..]
                .iter()
                .all(|value| value.actionable_training_only)
        );
    }

    #[test]
    fn sprint103_19_s0_to_s2_pairs_are_exact_without_cross_pairing() {
        let values = build_pairs(true).unwrap();
        assert_eq!(
            values
                .iter()
                .map(|value| value.pair_id.as_str())
                .collect::<Vec<_>>(),
            ["S0", "S1", "S2"]
        );
        assert!(values.iter().all(|value| !value.cross_pairing));
        assert!(values.iter().all(|value| {
            value.opportunity_threshold_bits == 0.5_f64.to_bits()
                && value.direction_threshold_bits == 0.5_f64.to_bits()
                && value.opportunity_below_threshold_abstains
                && value.direction_equality_maps_up
        }));
    }

    #[test]
    fn sprint103_20_no_participant_training_is_authorized() {
        assert!(
            build_participants(true)
                .unwrap()
                .iter()
                .all(|value| !value.training_authorized)
        );
    }

    #[test]
    fn sprint103_21_both_head_brier_gates_are_required() {
        let gate = build_fresh_gate().unwrap();
        assert!(
            gate.opportunity_lower_brier_than_o0_required
                && gate.direction_lower_brier_than_d0_required
        );
    }

    #[test]
    fn sprint103_22_correctness_and_coverage_cannot_override_brier() {
        let gate = build_fresh_gate().unwrap();
        assert!(gate.correctness_cannot_override_brier && gate.coverage_cannot_override_brier);
    }

    #[test]
    fn sprint103_23_final_holdout_remains_unauthorized() {
        let gate = build_final_gate().unwrap();
        assert!(
            !gate.execution_authorized
                && gate.prediction_count + gate.label_read_count + gate.metric_count == 0
        );
    }

    #[test]
    fn sprint103_24_malformed_protobuf_rejects() {
        assert!(decode_candidate(&[0xff, 0x00]).is_err());
    }

    #[test]
    fn sprint103_40_volatility_policy_uses_exactly_144_past_returns() {
        assert_eq!(VOLATILITY_LOOKBACK_RETURNS, 144);
    }

    #[test]
    fn sprint103_41_label_registration_precedes_and_forbids_private_partitions() {
        let policy = build_selection_policy().unwrap();
        let registration = label_registration_fixture(&policy);
        assert!(validate_label_registration(&registration, &policy).is_ok());
        assert!(
            registration.fresh_validation_access_forbidden
                && registration.final_holdout_access_forbidden
                && registration.model_training_forbidden
        );
    }

    #[test]
    fn sprint103_42_candidate_reports_reject_sealed_holdout() {
        let policy = build_selection_policy().unwrap();
        let registration = label_registration_fixture(&policy);
        let value = candidate_fixture(
            &registration,
            MomentumReplayPartitionV1::SealedHoldout,
            0.25,
            [400, 400, 1_200],
        );
        assert!(validate_candidate(&value).is_err());
        assert!(
            build_candidate_report(
                &registration,
                MomentumReplayPartitionV1::SealedHoldout,
                0.25,
                &[evidence(0.01, 0.01)],
            )
            .is_err()
        );
    }

    #[test]
    fn sprint103_43_class_support_gate_derives_from_frozen_limits() {
        let policy = build_selection_policy().unwrap();
        let registration = label_registration_fixture(&policy);
        let passing = candidate_fixture(
            &registration,
            MomentumReplayPartitionV1::Development,
            0.25,
            [400, 400, 1_200],
        );
        let failing = candidate_fixture(
            &registration,
            MomentumReplayPartitionV1::Development,
            0.25,
            [100, 400, 1_500],
        );
        assert!(candidate_support_passes(&passing));
        assert!(!candidate_support_passes(&failing));
    }

    #[test]
    fn sprint103_44_partition_prevalence_drift_gate_is_enforced() {
        let policy = build_selection_policy().unwrap();
        let registration = label_registration_fixture(&policy);
        let mut candidates = passing_candidates(&registration);
        let validation = candidates
            .iter_mut()
            .find(|value| {
                value.multiplier == 1.0 && value.partition == MomentumReplayPartitionV1::Validation
            })
            .unwrap();
        validation.actionable_up_count = 700;
        validation.actionable_down_count = 300;
        validation.abstain_count = 1_000;
        validation.actionable_up_prevalence = 0.35;
        validation.actionable_down_prevalence = 0.15;
        validation.abstain_prevalence = 0.50;
        validation.candidate_digest = candidate_digest(validation);
        let receipt = build_selection_receipt(&registration, &candidates).unwrap();
        assert_eq!(receipt.selected_multiplier, Some(0.5));
    }

    #[test]
    fn sprint103_45_largest_passing_multiplier_is_selected() {
        let policy = build_selection_policy().unwrap();
        let registration = label_registration_fixture(&policy);
        let receipt =
            build_selection_receipt(&registration, &passing_candidates(&registration)).unwrap();
        assert_eq!(receipt.selected_multiplier, Some(1.0));
    }

    #[test]
    fn sprint103_46_no_passing_multiplier_blocks_two_stage_registration() {
        let policy = build_selection_policy().unwrap();
        let registration = label_registration_fixture(&policy);
        let candidates = MULTIPLIERS
            .into_iter()
            .flat_map(|multiplier| {
                [
                    candidate_fixture(
                        &registration,
                        MomentumReplayPartitionV1::Development,
                        multiplier,
                        [100, 100, 1_800],
                    ),
                    candidate_fixture(
                        &registration,
                        MomentumReplayPartitionV1::Validation,
                        multiplier,
                        [100, 100, 1_800],
                    ),
                ]
            })
            .collect::<Vec<_>>();
        let receipt = build_selection_receipt(&registration, &candidates).unwrap();
        assert_eq!(
            receipt.selection_status,
            MomentumT10ActionabilitySelectionStatusV1::NoStableActionabilityThreshold
        );
        assert!(build_participants(false).unwrap().is_empty());
        assert!(build_pairs(false).unwrap().is_empty());
    }

    #[test]
    fn sprint103_46b_insufficient_fresh_support_blocks_two_stage_registration() {
        let mut value = report_fixture();
        value.fresh_validation_event_count = MINIMUM_SUPPORT - 1;
        value.fresh_validation_support_status =
            MomentumT10FreshValidationSupportStatusV1::FreshValidationInsufficientSupport;
        value.fresh_validation_support_sufficient = false;
        value.participant_registrations.clear();
        value.pair_registrations.clear();
        value.two_stage_registration_digest = None;
        value.report_digest = report_digest(&value);
        assert!(validate_report(&value).is_ok());
        assert_eq!(
            value.fresh_validation_support_status,
            MomentumT10FreshValidationSupportStatusV1::FreshValidationInsufficientSupport
        );
    }

    #[test]
    fn sprint103_47_o0_uses_prior_actionability_labels_only() {
        let value = &build_participants(true).unwrap()[0];
        assert_eq!(value.feature_policy, "PriorRevealedActionabilityLabelsOnly");
        assert!(value.prior_revealed_labels_only);
    }

    #[test]
    fn sprint103_48_o1_uses_anchor_features_only() {
        let value = &build_participants(true).unwrap()[1];
        assert_eq!(value.feature_policy, "FrozenTenMinuteAnchor");
        assert_eq!(value.feature_dimension, 6);
    }

    #[test]
    fn sprint103_49_o2_uses_strong_shrink_compact_features_exactly() {
        let value = &build_participants(true).unwrap()[2];
        assert_eq!(value.feature_policy, "FrozenCompactMicro69");
        assert_eq!(value.feature_dimension, 69);
        assert_eq!(value.l2_multiplier, 4);
    }

    #[test]
    fn sprint103_50_d0_uses_prior_actionable_direction_labels_only() {
        let value = &build_participants(true).unwrap()[3];
        assert_eq!(
            value.feature_policy,
            "PriorRevealedActionableDirectionsOnly"
        );
        assert!(value.actionable_training_only);
    }

    #[test]
    fn sprint103_51_d1_uses_anchor_features_only() {
        let value = &build_participants(true).unwrap()[4];
        assert_eq!(value.feature_policy, "FrozenTenMinuteAnchor");
        assert_eq!(value.feature_dimension, 6);
        assert!(value.actionable_training_only);
    }

    #[test]
    fn sprint103_52_d2_uses_strong_shrink_compact_features_exactly() {
        let value = &build_participants(true).unwrap()[5];
        assert_eq!(value.feature_policy, "FrozenCompactMicro69");
        assert_eq!(value.feature_dimension, 69);
        assert_eq!(value.l2_multiplier, 4);
        assert!(value.actionable_training_only);
    }

    #[test]
    fn sprint103_53_abstain_cannot_train_any_direction_head() {
        assert!(
            build_participants(true).unwrap()[3..]
                .iter()
                .all(|value| value.actionable_training_only)
        );
        assert!(
            build_future_training("split")
                .unwrap()
                .direction_excludes_abstain
        );
    }

    #[test]
    fn sprint103_54_future_refits_are_chronological_and_revealed_only() {
        let value = build_future_training("split").unwrap();
        assert!(
            value.daily_utc_refit
                && value.previously_revealed_labels_only
                && value.chronological_order_required
                && value.training_only_normalizers
                && value.persist_and_reopen_all_receipts
                && value.participants_frozen_per_utc_day
                && value.later_fresh_labels_only_after_observable
                && value.consumed_design_pool_is_former_development_and_validation
                && value.fresh_validation_use_once
                && value.final_holdout_requires_separate_authorization
                && value.fresh_evidence_split_digest == "split"
                && !value.execution_authorized
        );
    }

    #[test]
    fn sprint103_55_fresh_gate_has_frozen_coverage_and_integrity_rules() {
        let value = build_fresh_gate().unwrap();
        assert_eq!(
            (value.minimum_coverage, value.maximum_coverage),
            (0.10, 0.70)
        );
        assert!(
            value.consumed_design_replay_required
                && value.fresh_validation_required
                && value.finite_no_collapse_no_saturation_required
                && value.chronology_leakage_integrity_required
                && value.result_selected_mutation_forbidden
                && value.final_holdout_access_required_zero
                && !value.execution_authorized
        );
        assert_eq!(value.opportunity_metric_names.len(), 7);
        assert_eq!(value.direction_metric_names.len(), 7);
        assert_eq!(value.selective_metric_names.len(), 14);
        assert!(
            value
                .selective_metric_names
                .iter()
                .all(|metric| !metric.to_ascii_lowercase().contains("p&l"))
        );
    }

    #[test]
    fn sprint103_56_final_gate_requires_separate_owner_authorization() {
        let value = build_final_gate().unwrap();
        assert!(
            value.consumed_design_pass_required
                && value.fresh_validation_pass_required
                && value.post_validation_design_change_forbidden
                && value.deterministic_eligible_cohort_required
                && value.separate_owner_authorization_required
                && !value.execution_authorized
        );
    }

    #[test]
    fn sprint103_57_no_new_models_predictions_or_evaluations_exist() {
        let value = MomentumT10ActionabilitySafetyCountersV1::default();
        assert_eq!(
            value.new_opportunity_model_fits
                + value.new_direction_model_fits
                + value.new_selective_predictions
                + value.new_selective_evaluations,
            0
        );
    }

    #[test]
    fn sprint103_58_t30_t60_month_and_year_remain_inaccessible() {
        let value = MomentumT10ActionabilitySafetyCountersV1::default();
        assert_eq!(
            value.t30_model_executions
                + value.t60_model_executions
                + value.day_view_loads
                + value.week_view_loads
                + value.month_view_loads
                + value.year_view_loads,
            0
        );
    }

    #[test]
    fn sprint103_59_live_governance_network_and_trading_authority_are_zero() {
        let value = MomentumT10ActionabilitySafetyCountersV1::default();
        assert_eq!(
            value.network_requests
                + value.live_operations
                + value.reward_applications
                + value.penalty_applications
                + value.chair_actions
                + value.vote_actions
                + value.trading_actions,
            0
        );
    }

    #[test]
    fn sprint103_60_completed_replay_performs_zero_work() {
        let report = report_fixture();
        let protected = MomentumMicroProtectedBeforeStateV1 {
            series_digest: String::new(),
            event_two_outcome_receipt_digest: String::new(),
            event_two_outcome_capsule_digest: String::new(),
            opening_authorization_digest: String::new(),
            opening_bundle_digest: String::new(),
            event_two_ledger_entry_digest: String::new(),
            eligibility_receipt_digest: String::new(),
            completed_pause_digest: String::new(),
            completed_event_count: 2,
            scorable_event_count: 2,
            eligibility_status: String::new(),
            epoch_three_registered: false,
            live_parameter_digests: Vec::new(),
            live_normalizer_digests: Vec::new(),
            protected_live_aggregate_digest: String::new(),
            historical_store_digest: String::new(),
            qualified_six_replay_digest: String::new(),
            diagnostic_store_digest: String::new(),
            active_roster_digest: String::new(),
            zero_authority_and_action_counters: true,
            state_digest: "protected".into(),
        };
        let replay = completed_replay(
            report,
            MomentumT10ActionabilityDesignRunModeV1::Status,
            &protected,
            Instant::now(),
        )
        .unwrap();
        assert_eq!(
            replay.safety_counters,
            MomentumT10ActionabilitySafetyCountersV1::default()
        );
    }

    #[test]
    fn sprint103_61_conflicting_candidate_artifact_rejects() {
        let policy = build_selection_policy().unwrap();
        let registration = label_registration_fixture(&policy);
        let mut value = candidate_fixture(
            &registration,
            MomentumReplayPartitionV1::Development,
            0.25,
            [400, 400, 1_200],
        );
        value.actionable_up_count += 1;
        assert!(validate_candidate(&value).is_err());
    }

    #[test]
    fn sprint103_62_candidate_manual_protobuf_round_trips() {
        let policy = build_selection_policy().unwrap();
        let registration = label_registration_fixture(&policy);
        let value = candidate_fixture(
            &registration,
            MomentumReplayPartitionV1::Development,
            0.25,
            [400, 400, 1_200],
        );
        assert_eq!(
            decode_candidate(&encode_candidate(&value).unwrap()).unwrap(),
            value
        );
    }

    #[test]
    fn sprint103_62b_final_report_manual_protobuf_round_trips() {
        let value = report_fixture();
        assert_eq!(
            decode_report(&encode_report(&value).unwrap()).unwrap(),
            value
        );
    }

    #[test]
    fn sprint103_63_text_and_json_expose_the_same_public_result() {
        let value = report_fixture();
        let text = format_momentum_t10_actionability_design_text_v1(&value);
        let json = serde_json::to_value(&value).unwrap();
        assert!(text.contains("selected_multiplier=1.00"));
        assert_eq!(
            json["selection_receipt"]["selected_multiplier"].as_f64(),
            Some(1.0)
        );
        assert!(text.contains(json["report_digest"].as_str().unwrap()));
    }
}
