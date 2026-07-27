//! Post-result diagnostics for the completed Qualified-Six replay.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::{Deserialize, Serialize};

use crate::stable_hash_string;

use super::{
    momentum_future_prediction_v4::{
        ArtifactBuilderV4_2, ArtifactReaderV4_2, as_u64, as_usize, persist_artifact,
    },
    momentum_multitimeframe_history_v1::momentum_qualified_replay_protected_state_v1,
    momentum_qualified_six_replay_v1::{
        COLLAPSE_VARIANCE_THRESHOLD, COMPARISON_EPSILON, MomentumQualifiedContributionStatusV1,
        MomentumQualifiedDiagnosticEventEvidenceV1, MomentumQualifiedDiagnosticRefitEvidenceV1,
        MomentumQualifiedDiagnosticSourceHeaderV1, MomentumQualifiedDiagnosticSourceV1,
        MomentumQualifiedParticipantMetricsV1, MomentumQualifiedParticipantV1,
        MomentumReplayPartitionV1, load_momentum_qualified_diagnostic_source_header_v1,
        load_momentum_qualified_diagnostic_source_v1,
    },
};

#[cfg(test)]
use super::momentum_qualified_six_replay_v1::MomentumQualifiedContributionReceiptV1;

const ROOT: &str = "state/historical_replay/momentum_qualified_six_diagnostics/v1";
const REGISTRATION_VERSION: &str = "momentum-qualified-six-diagnostic-registration-v1";
const POLICY_VERSION: &str = "momentum-qualified-six-diagnostic-policy-v1";
const THRESHOLD_VERSION: &str = "momentum-qualified-six-regime-threshold-v1";
const SUITE_VERSION: &str = "momentum-qualified-six-diagnostic-suite-v1";
const REQUIREMENTS_VERSION: &str = "momentum-qualified-six-challenger-requirements-v1";
const JOURNAL_VERSION: &str = "momentum-qualified-six-diagnostic-journal-v1";
const REPORT_VERSION: &str = "momentum-qualified-six-diagnostic-public-report-v1";
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;
const ROLLING_DAY_EVENTS: usize = 144;
const ROLLING_WEEK_EVENTS: usize = 1_008;
const REGIME_MINIMUM_SUPPORT: usize = 512;
const PREDICTION_CLAMP: f64 = 1e-6;
const NEAR_HALF_THRESHOLD: f64 = 1e-3;
const MODERATE_DRIFT_THRESHOLD: f64 = 0.10;
const HIGH_DRIFT_THRESHOLD: f64 = 0.50;
const CALIBRATION_BOUNDARIES: [f64; 11] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
const PUBLIC_LABELS: [&str; 6] = [
    "HistoricalResearchOnly",
    "PostResultDiagnosticOnly",
    "QualifiedSixNotFullEight",
    "NotIndependentEvidence",
    "NotHoldoutEvidence",
    "NotTradingAuthority",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedDiagnosticEvidenceClassV1 {
    PostResultDiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedDiagnosticStatusV1 {
    Unregistered,
    Registered,
    Complete,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedDiagnosticRunModeV1 {
    Status,
    DryRun,
    RegisterAndExecuteLocal,
}

impl MomentumQualifiedDiagnosticRunModeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::DryRun => "dry-run",
            Self::RegisterAndExecuteLocal => "register-execute-local",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MomentumDiagnosticCalendarGrainV1 {
    UtcDay,
    UtcWeek,
    UtcMonth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumDiagnosticRelationV1 {
    LowerBrier,
    HigherBrier,
    NumericallyEquivalent,
    InsufficientDiagnosticSupport,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedVolatilityRegimeV1 {
    LowVolatility,
    MediumVolatility,
    HighVolatility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedDailyTrendRegimeV1 {
    DownTrend,
    Flat,
    UpTrend,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedModelDriftStatusV1 {
    StableAcrossRefits,
    ModerateDeterministicDrift,
    HighDeterministicDrift,
    PartitionBoundaryShift,
    ProbabilityCollapse,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedPartitionStabilityV1 {
    LowerBrierAcrossDevelopmentAndValidation,
    DevelopmentOnlyLowerBrier,
    ValidationOnlyLowerBrier,
    HigherBrierAcrossDevelopmentAndValidation,
    NumericallyEquivalentAcrossPartitions,
    MixedOrInsufficientEvidence,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedHoldoutEligibilityV1 {
    EligibleForFutureSealedHoldoutEvaluation,
    NotEligibleForSealedHoldout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumResearchPriorityV1 {
    PrimaryDiagnosticTarget,
    SecondaryDiagnosticTarget,
    DeprioritizedByCurrentEvidence,
    BlockedByUnresolvedEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedSaturationStatusV1 {
    NotSaturated,
    LowBoundarySaturation,
    HighBoundarySaturation,
    TwoSidedSaturation,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedProbabilityCollapseStatusV1 {
    BenchmarkExempt,
    NotCollapsed,
    ProbabilityCollapse,
    IntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumQualifiedDiagnosticPolicyV1 {
    policy_version: String,
    policy_name: String,
    frozen_values: Vec<String>,
    policy_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumQualifiedSixDiagnosticRegistrationV1 {
    pub registration_version: String,
    pub source_replay_registration_digest: String,
    pub source_replay_journal_digest: String,
    pub source_public_report_digest: String,
    pub included_participant_ids: Vec<String>,
    pub included_partitions: Vec<MomentumReplayPartitionV1>,
    pub evidence_class: MomentumQualifiedDiagnosticEvidenceClassV1,
    pub paired_brier_policy_digest: String,
    pub calendar_stability_policy_digest: String,
    pub rolling_stability_policy_digest: String,
    pub calibration_policy_digest: String,
    pub probability_distribution_policy_digest: String,
    pub prevalence_drift_policy_digest: String,
    pub regime_policy_digest: String,
    pub model_drift_policy_digest: String,
    pub holdout_gate_policy_digest: String,
    pub post_result: bool,
    pub confirmatory_claim_allowed: bool,
    pub holdout_authority: bool,
    pub live_authority: bool,
    pub trading_authority: bool,
    pub holdout_access_forbidden: bool,
    pub new_training_forbidden: bool,
    pub result_selected_slicing_forbidden: bool,
    pub live_authority_forbidden: bool,
    pub governance_authority_forbidden: bool,
    pub trading_authority_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumPairedBrierDiagnosticV1 {
    pub participant_id: String,
    pub partition: MomentumReplayPartitionV1,
    pub paired_event_count: usize,
    pub mean_delta: f64,
    pub median_delta: f64,
    pub minimum_delta: f64,
    pub maximum_delta: f64,
    pub positive_delta_count: usize,
    pub negative_delta_count: usize,
    pub equivalent_delta_count: usize,
    pub finite_value_proof: bool,
    pub diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumCalendarStabilityDiagnosticV1 {
    pub participant_id: String,
    pub partition: MomentumReplayPartitionV1,
    pub grain: MomentumDiagnosticCalendarGrainV1,
    pub group_count: usize,
    pub lower_brier_group_count: usize,
    pub higher_brier_group_count: usize,
    pub equivalent_group_count: usize,
    pub median_group_delta: f64,
    pub worst_group_delta: f64,
    pub best_group_delta: f64,
    pub longest_lower_brier_streak: usize,
    pub longest_higher_brier_streak: usize,
    pub cumulative_trajectory_digest: String,
    pub diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumRollingStabilityDiagnosticV1 {
    pub participant_id: String,
    pub partition: MomentumReplayPartitionV1,
    pub window_event_count: usize,
    pub variant: String,
    pub eligible_window_count: usize,
    pub lower_brier_window_count: usize,
    pub higher_brier_window_count: usize,
    pub equivalent_window_count: usize,
    pub minimum_window_delta: f64,
    pub maximum_window_delta: f64,
    pub median_window_delta: f64,
    pub sign_change_count: usize,
    pub minimum_timestamp_span_ms: u64,
    pub maximum_timestamp_span_ms: u64,
    pub median_timestamp_span_ms: u64,
    pub diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumCalibrationBinDiagnosticV1 {
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub upper_inclusive: bool,
    pub support: usize,
    pub mean_predicted_probability: Option<f64>,
    pub observed_positive_frequency: Option<f64>,
    pub absolute_calibration_gap: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumCalibrationDiagnosticV1 {
    pub participant_id: String,
    pub partition: MomentumReplayPartitionV1,
    pub bins: Vec<MomentumCalibrationBinDiagnosticV1>,
    pub weighted_aggregate_calibration_gap: f64,
    pub empty_bin_count: usize,
    pub finite_value_proof: bool,
    pub diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumProbabilityDistributionDiagnosticV1 {
    pub participant_id: String,
    pub partition: MomentumReplayPartitionV1,
    pub minimum: f64,
    pub percentile_01: f64,
    pub percentile_05: f64,
    pub percentile_25: f64,
    pub median: f64,
    pub percentile_75: f64,
    pub percentile_95: f64,
    pub percentile_99: f64,
    pub maximum: f64,
    pub mean: f64,
    pub standard_deviation: f64,
    pub exact_constant_value_count: usize,
    pub near_half_count: usize,
    pub extreme_low_count: usize,
    pub extreme_high_count: usize,
    pub nonfinite_count: usize,
    pub saturation_status: MomentumQualifiedSaturationStatusV1,
    pub collapse_status: MomentumQualifiedProbabilityCollapseStatusV1,
    pub diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumPrevalenceDriftDiagnosticV1 {
    pub partition: MomentumReplayPartitionV1,
    pub grain: MomentumDiagnosticCalendarGrainV1,
    pub scorable_count: usize,
    pub positive_count: usize,
    pub negative_count: usize,
    pub partition_positive_prevalence: f64,
    pub group_count: usize,
    pub minimum_group_prevalence: f64,
    pub maximum_group_prevalence: f64,
    pub minimum_deviation: f64,
    pub maximum_deviation: f64,
    pub prevalence_trajectory_digest: String,
    pub diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumQualifiedRegimeThresholdReceiptV1 {
    threshold_version: String,
    source_diagnostic_registration_digest: String,
    development_event_count: usize,
    low_volatility_upper: f64,
    medium_volatility_upper: f64,
    validation_value_access_count: usize,
    threshold_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumRegimeMetricDiagnosticV1 {
    pub participant_id: String,
    pub partition: MomentumReplayPartitionV1,
    pub regime_dimension: String,
    pub volatility_regime: Option<MomentumQualifiedVolatilityRegimeV1>,
    pub daily_trend_regime: Option<MomentumQualifiedDailyTrendRegimeV1>,
    pub event_count: usize,
    pub scorable_count: usize,
    pub neutral_count: usize,
    pub mean_brier: Option<f64>,
    pub correctness: Option<f64>,
    pub paired_brier_delta_versus_q0: Option<f64>,
    pub relation: MomentumDiagnosticRelationV1,
    pub finite_value_proof: bool,
    pub diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumModelDriftDiagnosticV1 {
    pub participant_id: String,
    pub partition: MomentumReplayPartitionV1,
    pub refit_count: usize,
    pub parameter_digest_trajectory: String,
    pub normalizer_digest_trajectory: String,
    pub parameter_finite: bool,
    pub normalizer_finite: bool,
    pub training_loss_finite: bool,
    pub minimum_training_example_count: usize,
    pub maximum_training_example_count: usize,
    pub minimum_training_prevalence: f64,
    pub maximum_training_prevalence: f64,
    pub median_parameter_norm_change: f64,
    pub maximum_parameter_norm_change: f64,
    pub median_normalizer_shift: f64,
    pub maximum_normalizer_shift: f64,
    pub median_daily_prediction_dispersion: f64,
    pub partition_boundary_shift: bool,
    pub status: MomentumQualifiedModelDriftStatusV1,
    pub diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumPartitionStabilityReceiptV1 {
    pub participant_id: String,
    pub classification: MomentumQualifiedPartitionStabilityV1,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumHoldoutEligibilityReceiptV1 {
    pub participant_id: String,
    pub lower_brier_development: bool,
    pub lower_brier_validation: bool,
    pub sufficient_paired_support: bool,
    pub finite_predictions_and_metrics: bool,
    pub probability_collapse_absent: bool,
    pub chronology_and_leakage_passed: bool,
    pub integrity_passed: bool,
    pub source_replay_unmutated: bool,
    pub eligibility: MomentumQualifiedHoldoutEligibilityV1,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumQualifiedChallengerRequirementsV1 {
    pub requirements_version: String,
    pub source_diagnostic_registration_digest: String,
    pub source_diagnostic_report_digest: String,
    pub constant_benchmark_mandatory: bool,
    pub full_eight_claim_forbidden: bool,
    pub month_year_use_forbidden: bool,
    pub complexity_escalation_allowed: bool,
    pub interaction_expansion_allowed: bool,
    pub sequence_model_allowed: bool,
    pub micro_block_research_priority: MomentumResearchPriorityV1,
    pub qualified_macro_addition_priority: MomentumResearchPriorityV1,
    pub label_forensics_required: bool,
    pub calibration_repair_required: bool,
    pub regime_stability_required: bool,
    pub two_partition_improvement_required: bool,
    pub new_model_execution_authorized: bool,
    pub holdout_execution_authorized: bool,
    pub requirements_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumQualifiedDiagnosticJournalV1 {
    journal_version: String,
    registration_digest: String,
    source_replay_journal_digest: String,
    regime_threshold_digest: String,
    paired_suite_digest: String,
    calendar_suite_digest: String,
    rolling_suite_digest: String,
    calibration_suite_digest: String,
    probability_suite_digest: String,
    prevalence_suite_digest: String,
    regime_suite_digest: String,
    model_drift_suite_digest: String,
    partition_stability_suite_digest: String,
    holdout_eligibility_suite_digest: String,
    challenger_requirements_digest: String,
    holdout_label_reads: usize,
    holdout_prediction_reads: usize,
    holdout_metric_reads: usize,
    live_outcome_requests: usize,
    live_outcome_openings: usize,
    deterministic: bool,
    journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumQualifiedSixDiagnosticReportV1 {
    pub report_version: String,
    pub run_mode: String,
    pub status: MomentumQualifiedDiagnosticStatusV1,
    pub evidence_class: MomentumQualifiedDiagnosticEvidenceClassV1,
    pub post_result: bool,
    pub confirmatory_claim_allowed: bool,
    pub holdout_authority: bool,
    pub live_authority: bool,
    pub trading_authority: bool,
    pub source_replay_registration_digest: Option<String>,
    pub source_replay_journal_digest: Option<String>,
    pub source_public_report_digest: Option<String>,
    pub diagnostic_registration_digest: Option<String>,
    pub participant_ids: Vec<String>,
    pub included_partitions: Vec<MomentumReplayPartitionV1>,
    pub paired_brier_diagnostics: Vec<MomentumPairedBrierDiagnosticV1>,
    pub calendar_stability_diagnostics: Vec<MomentumCalendarStabilityDiagnosticV1>,
    pub rolling_stability_diagnostics: Vec<MomentumRollingStabilityDiagnosticV1>,
    pub calibration_diagnostics: Vec<MomentumCalibrationDiagnosticV1>,
    pub probability_distribution_diagnostics: Vec<MomentumProbabilityDistributionDiagnosticV1>,
    pub prevalence_drift_diagnostics: Vec<MomentumPrevalenceDriftDiagnosticV1>,
    pub volatility_threshold_low_upper: Option<f64>,
    pub volatility_threshold_medium_upper: Option<f64>,
    pub regime_diagnostics: Vec<MomentumRegimeMetricDiagnosticV1>,
    pub model_drift_diagnostics: Vec<MomentumModelDriftDiagnosticV1>,
    pub partition_stability_receipts: Vec<MomentumPartitionStabilityReceiptV1>,
    pub holdout_eligibility_receipts: Vec<MomentumHoldoutEligibilityReceiptV1>,
    pub challenger_requirements: Option<MomentumQualifiedChallengerRequirementsV1>,
    pub holdout_label_reads: usize,
    pub holdout_prediction_reads: usize,
    pub holdout_metric_reads: usize,
    pub holdout_execution_modes: usize,
    pub live_outcome_requests: usize,
    pub live_outcome_openings: usize,
    pub live_participant_changes: usize,
    pub winner_selections: usize,
    pub ranking_creations: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub chair_decisions: usize,
    pub trading_actions: usize,
    pub network_request_attempts: usize,
    pub month_view_load_count: usize,
    pub year_view_load_count: usize,
    pub full_eight_a3_blocked: bool,
    pub protected_artifacts_unchanged: bool,
    pub active_roster_unchanged: bool,
    pub labels: Vec<String>,
    pub diagnostic_journal_digest: Option<String>,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub model_refit_count: usize,
    pub prediction_computation_count: usize,
    pub evaluation_computation_count: usize,
    pub diagnostic_computation_count: usize,
    pub runtime_duration_ms: u64,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumDiagnosticSuiteReceiptV1 {
    suite_version: String,
    suite_name: String,
    record_digests: Vec<String>,
    suite_digest: String,
}

fn canonical_digest<T: Clone + std::fmt::Debug>(value: &T, clear: impl FnOnce(&mut T)) -> String {
    let mut canonical = value.clone();
    clear(&mut canonical);
    stable_hash_string(&format!("{canonical:?}"))
}

fn policy_digest(value: &MomentumQualifiedDiagnosticPolicyV1) -> String {
    canonical_digest(value, |item| item.policy_digest.clear())
}

fn registration_digest(value: &MomentumQualifiedSixDiagnosticRegistrationV1) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn threshold_digest(value: &MomentumQualifiedRegimeThresholdReceiptV1) -> String {
    canonical_digest(value, |item| item.threshold_digest.clear())
}

fn paired_digest(value: &MomentumPairedBrierDiagnosticV1) -> String {
    canonical_digest(value, |item| item.diagnostic_digest.clear())
}

fn calendar_digest(value: &MomentumCalendarStabilityDiagnosticV1) -> String {
    canonical_digest(value, |item| item.diagnostic_digest.clear())
}

fn rolling_digest(value: &MomentumRollingStabilityDiagnosticV1) -> String {
    canonical_digest(value, |item| item.diagnostic_digest.clear())
}

fn calibration_digest(value: &MomentumCalibrationDiagnosticV1) -> String {
    canonical_digest(value, |item| item.diagnostic_digest.clear())
}

fn probability_digest(value: &MomentumProbabilityDistributionDiagnosticV1) -> String {
    canonical_digest(value, |item| item.diagnostic_digest.clear())
}

fn prevalence_digest(value: &MomentumPrevalenceDriftDiagnosticV1) -> String {
    canonical_digest(value, |item| item.diagnostic_digest.clear())
}

fn regime_digest(value: &MomentumRegimeMetricDiagnosticV1) -> String {
    canonical_digest(value, |item| item.diagnostic_digest.clear())
}

fn model_drift_digest(value: &MomentumModelDriftDiagnosticV1) -> String {
    canonical_digest(value, |item| item.diagnostic_digest.clear())
}

fn partition_stability_digest(value: &MomentumPartitionStabilityReceiptV1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn eligibility_digest(value: &MomentumHoldoutEligibilityReceiptV1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn requirements_digest(value: &MomentumQualifiedChallengerRequirementsV1) -> String {
    canonical_digest(value, |item| {
        item.source_diagnostic_report_digest.clear();
        item.requirements_digest.clear();
    })
}

fn suite_digest(value: &MomentumDiagnosticSuiteReceiptV1) -> String {
    canonical_digest(value, |item| item.suite_digest.clear())
}

fn journal_digest(value: &MomentumQualifiedDiagnosticJournalV1) -> String {
    canonical_digest(value, |item| item.journal_digest.clear())
}

fn report_digest(value: &MomentumQualifiedSixDiagnosticReportV1) -> String {
    canonical_digest(value, |item| {
        item.run_mode.clear();
        item.artifacts_written = 0;
        item.duplicate_artifact_count = 0;
        item.model_refit_count = 0;
        item.prediction_computation_count = 0;
        item.evaluation_computation_count = 0;
        item.diagnostic_computation_count = 0;
        item.runtime_duration_ms = 0;
        item.report_digest.clear();
        if let Some(requirements) = &mut item.challenger_requirements {
            requirements.source_diagnostic_report_digest.clear();
            requirements.requirements_digest = requirements_digest(requirements);
        }
    })
}

fn participant_ids() -> Vec<String> {
    MomentumQualifiedParticipantV1::ORDERED
        .iter()
        .map(|participant| participant.id().to_string())
        .collect()
}

fn learned_participant_ids() -> Vec<String> {
    participant_ids().into_iter().skip(1).collect()
}

fn included_partitions() -> Vec<MomentumReplayPartitionV1> {
    vec![
        MomentumReplayPartitionV1::Development,
        MomentumReplayPartitionV1::Validation,
    ]
}

fn parse_partition(value: &str) -> Result<MomentumReplayPartitionV1, String> {
    match value {
        "development" => Ok(MomentumReplayPartitionV1::Development),
        "validation" => Ok(MomentumReplayPartitionV1::Validation),
        _ => Err("qualified-six diagnostic partition rejected".to_string()),
    }
}

fn build_policies() -> Result<Vec<MomentumQualifiedDiagnosticPolicyV1>, String> {
    let definitions = [
        (
            "paired-brier",
            vec![
                "paired-scorable-events-only".to_string(),
                format!("comparison-epsilon-bits={}", COMPARISON_EPSILON.to_bits()),
            ],
        ),
        (
            "calendar-stability",
            vec![
                "utc-calendar-day".to_string(),
                "utc-calendar-week-monday".to_string(),
                "utc-calendar-month".to_string(),
                "all-periods-retained".to_string(),
            ],
        ),
        (
            "rolling-stability",
            vec![
                format!("day-scale-events={ROLLING_DAY_EVENTS}"),
                format!("week-scale-events={ROLLING_WEEK_EVENTS}"),
                "non-overlapping-and-step-one-rolling".to_string(),
            ],
        ),
        (
            "calibration",
            CALIBRATION_BOUNDARIES
                .iter()
                .map(|value| format!("boundary-bits={}", value.to_bits()))
                .collect(),
        ),
        (
            "probability-distribution",
            vec![
                format!(
                    "collapse-variance-threshold-bits={}",
                    COLLAPSE_VARIANCE_THRESHOLD.to_bits()
                ),
                format!("near-half-threshold-bits={}", NEAR_HALF_THRESHOLD.to_bits()),
                format!("extreme-clamp-bits={}", PREDICTION_CLAMP.to_bits()),
                "q0-collapse-exempt".to_string(),
            ],
        ),
        (
            "prevalence-drift",
            vec![
                "scorable-labels-only".to_string(),
                "utc-day-week-month".to_string(),
            ],
        ),
        (
            "past-only-regime",
            vec![
                "micro-volatility-population-stddev-simple-return".to_string(),
                format!("past-ten-minute-return-count={ROLLING_DAY_EVENTS}"),
                "development-terciles-only".to_string(),
                "daily-trend-last-16-closed-candles".to_string(),
                format!("flat-epsilon-bits={}", COMPARISON_EPSILON.to_bits()),
                format!("minimum-support={REGIME_MINIMUM_SUPPORT}"),
            ],
        ),
        (
            "model-drift",
            vec![
                format!(
                    "moderate-relative-threshold-bits={}",
                    MODERATE_DRIFT_THRESHOLD.to_bits()
                ),
                format!(
                    "high-relative-threshold-bits={}",
                    HIGH_DRIFT_THRESHOLD.to_bits()
                ),
                "thresholds-frozen-before-private-values".to_string(),
            ],
        ),
        (
            "holdout-gate",
            vec![
                "lower-brier-development-required".to_string(),
                "lower-brier-validation-required".to_string(),
                format!("minimum-paired-support={REGIME_MINIMUM_SUPPORT}"),
                "finite-no-collapse-no-leakage-integrity-required".to_string(),
                "diagnostics-cannot-override-two-partition-gate".to_string(),
            ],
        ),
    ];
    definitions
        .into_iter()
        .map(|(name, values)| {
            let mut policy = MomentumQualifiedDiagnosticPolicyV1 {
                policy_version: POLICY_VERSION.to_string(),
                policy_name: name.to_string(),
                frozen_values: values,
                policy_digest: String::new(),
            };
            policy.policy_digest = policy_digest(&policy);
            validate_policy(&policy)?;
            Ok(policy)
        })
        .collect()
}

fn policy_by_name<'a>(
    policies: &'a [MomentumQualifiedDiagnosticPolicyV1],
    name: &str,
) -> Result<&'a MomentumQualifiedDiagnosticPolicyV1, String> {
    policies
        .iter()
        .find(|policy| policy.policy_name == name)
        .ok_or_else(|| "qualified-six diagnostic policy unavailable".to_string())
}

fn build_registration(
    header: &MomentumQualifiedDiagnosticSourceHeaderV1,
    policies: &[MomentumQualifiedDiagnosticPolicyV1],
) -> Result<MomentumQualifiedSixDiagnosticRegistrationV1, String> {
    let mut value = MomentumQualifiedSixDiagnosticRegistrationV1 {
        registration_version: REGISTRATION_VERSION.to_string(),
        source_replay_registration_digest: header.registration_digest.clone(),
        source_replay_journal_digest: header.replay_journal_digest.clone(),
        source_public_report_digest: header.public_report_digest.clone(),
        included_participant_ids: header.participant_ids.clone(),
        included_partitions: included_partitions(),
        evidence_class: MomentumQualifiedDiagnosticEvidenceClassV1::PostResultDiagnosticOnly,
        paired_brier_policy_digest: policy_by_name(policies, "paired-brier")?
            .policy_digest
            .clone(),
        calendar_stability_policy_digest: policy_by_name(policies, "calendar-stability")?
            .policy_digest
            .clone(),
        rolling_stability_policy_digest: policy_by_name(policies, "rolling-stability")?
            .policy_digest
            .clone(),
        calibration_policy_digest: policy_by_name(policies, "calibration")?
            .policy_digest
            .clone(),
        probability_distribution_policy_digest: policy_by_name(
            policies,
            "probability-distribution",
        )?
        .policy_digest
        .clone(),
        prevalence_drift_policy_digest: policy_by_name(policies, "prevalence-drift")?
            .policy_digest
            .clone(),
        regime_policy_digest: policy_by_name(policies, "past-only-regime")?
            .policy_digest
            .clone(),
        model_drift_policy_digest: policy_by_name(policies, "model-drift")?
            .policy_digest
            .clone(),
        holdout_gate_policy_digest: policy_by_name(policies, "holdout-gate")?
            .policy_digest
            .clone(),
        post_result: true,
        confirmatory_claim_allowed: false,
        holdout_authority: false,
        live_authority: false,
        trading_authority: false,
        holdout_access_forbidden: true,
        new_training_forbidden: true,
        result_selected_slicing_forbidden: true,
        live_authority_forbidden: true,
        governance_authority_forbidden: true,
        trading_authority_forbidden: true,
        registration_digest: String::new(),
    };
    value.registration_digest = registration_digest(&value);
    validate_registration(&value)?;
    Ok(value)
}

fn validate_policy(value: &MomentumQualifiedDiagnosticPolicyV1) -> Result<(), String> {
    if value.policy_version != POLICY_VERSION
        || value.policy_name.is_empty()
        || value.frozen_values.is_empty()
        || value.frozen_values.iter().any(String::is_empty)
        || value.policy_digest != policy_digest(value)
    {
        return Err("qualified-six diagnostic policy rejected".to_string());
    }
    Ok(())
}

fn validate_registration(
    value: &MomentumQualifiedSixDiagnosticRegistrationV1,
) -> Result<(), String> {
    if value.registration_version != REGISTRATION_VERSION
        || [
            &value.source_replay_registration_digest,
            &value.source_replay_journal_digest,
            &value.source_public_report_digest,
            &value.paired_brier_policy_digest,
            &value.calendar_stability_policy_digest,
            &value.rolling_stability_policy_digest,
            &value.calibration_policy_digest,
            &value.probability_distribution_policy_digest,
            &value.prevalence_drift_policy_digest,
            &value.regime_policy_digest,
            &value.model_drift_policy_digest,
            &value.holdout_gate_policy_digest,
        ]
        .iter()
        .any(|value| value.is_empty())
        || value.included_participant_ids != participant_ids()
        || value.included_partitions != included_partitions()
        || value.evidence_class
            != MomentumQualifiedDiagnosticEvidenceClassV1::PostResultDiagnosticOnly
        || !value.post_result
        || value.confirmatory_claim_allowed
        || value.holdout_authority
        || value.live_authority
        || value.trading_authority
        || !value.holdout_access_forbidden
        || !value.new_training_forbidden
        || !value.result_selected_slicing_forbidden
        || !value.live_authority_forbidden
        || !value.governance_authority_forbidden
        || !value.trading_authority_forbidden
        || value.registration_digest != registration_digest(value)
    {
        return Err("qualified-six diagnostic registration rejected".to_string());
    }
    Ok(())
}

fn validate_threshold(value: &MomentumQualifiedRegimeThresholdReceiptV1) -> Result<(), String> {
    if value.threshold_version != THRESHOLD_VERSION
        || value.source_diagnostic_registration_digest.is_empty()
        || value.development_event_count == 0
        || !value.low_volatility_upper.is_finite()
        || !value.medium_volatility_upper.is_finite()
        || value.low_volatility_upper < 0.0
        || value.medium_volatility_upper < value.low_volatility_upper
        || value.validation_value_access_count != 0
        || value.threshold_digest != threshold_digest(value)
    {
        return Err("qualified-six diagnostic regime threshold rejected".to_string());
    }
    Ok(())
}

fn validate_header_registration(
    header: &MomentumQualifiedDiagnosticSourceHeaderV1,
    registration: &MomentumQualifiedSixDiagnosticRegistrationV1,
) -> Result<(), String> {
    if registration.source_replay_registration_digest != header.registration_digest
        || registration.source_replay_journal_digest != header.replay_journal_digest
        || registration.source_public_report_digest != header.public_report_digest
        || registration.included_participant_ids != header.participant_ids
        || header.holdout_label_reads != 0
        || header.holdout_metric_computations != 0
        || header.holdout_participant_predictions != 0
        || header.month_view_load_count != 0
        || header.year_view_load_count != 0
        || !header.full_eight_a3_blocked
        || !header.chronology_audit_passed
        || !header.leakage_audit_passed
    {
        return Err("qualified-six diagnostic source registration binding rejected".to_string());
    }
    Ok(())
}

fn sorted_finite(values: &[f64]) -> Result<Vec<f64>, String> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err("qualified-six diagnostic finite distribution required".to_string());
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    Ok(sorted)
}

fn quantile_sorted(sorted: &[f64], quantile: f64) -> Result<f64, String> {
    if sorted.is_empty() || !(0.0..=1.0).contains(&quantile) {
        return Err("qualified-six diagnostic quantile rejected".to_string());
    }
    let position = quantile * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    let weight = position - lower as f64;
    Ok(sorted[lower] * (1.0 - weight) + sorted[upper] * weight)
}

fn median(values: &[f64]) -> Result<f64, String> {
    quantile_sorted(&sorted_finite(values)?, 0.5)
}

fn median_u64(values: &[u64]) -> Result<u64, String> {
    if values.is_empty() {
        return Err("qualified-six diagnostic timestamp span unavailable".to_string());
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    Ok(sorted[(sorted.len() - 1) / 2])
}

fn relation(delta: f64) -> MomentumDiagnosticRelationV1 {
    if !delta.is_finite() {
        MomentumDiagnosticRelationV1::IntegrityFailure
    } else if delta < -COMPARISON_EPSILON {
        MomentumDiagnosticRelationV1::LowerBrier
    } else if delta > COMPARISON_EPSILON {
        MomentumDiagnosticRelationV1::HigherBrier
    } else {
        MomentumDiagnosticRelationV1::NumericallyEquivalent
    }
}

fn scorable_event(event: &MomentumQualifiedDiagnosticEventEvidenceV1) -> Result<bool, String> {
    let scorable = event.label.is_some();
    if event.partition == MomentumReplayPartitionV1::SealedHoldout
        || event.probabilities.len() != 5
        || event
            .probabilities
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        || scorable != (event.brier_values.len() == 5 && event.correctness.len() == 5)
        || event
            .brier_values
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        || event.label.is_some_and(|value| !matches!(value, 0.0 | 1.0))
    {
        return Err("qualified-six diagnostic event evidence rejected".to_string());
    }
    Ok(scorable)
}

fn events_for_partition<'a>(
    source: &'a MomentumQualifiedDiagnosticSourceV1,
    partition: MomentumReplayPartitionV1,
) -> Vec<&'a MomentumQualifiedDiagnosticEventEvidenceV1> {
    source
        .events
        .iter()
        .filter(|event| event.partition == partition)
        .collect()
}

fn paired_values(
    source: &MomentumQualifiedDiagnosticSourceV1,
    partition: MomentumReplayPartitionV1,
    participant_index: usize,
) -> Result<Vec<(u64, f64)>, String> {
    events_for_partition(source, partition)
        .into_iter()
        .filter_map(|event| match scorable_event(event) {
            Ok(true) => Some(Ok((
                event.prediction_timestamp_ms,
                event.brier_values[participant_index] - event.brier_values[0],
            ))),
            Ok(false) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn compute_paired(
    source: &MomentumQualifiedDiagnosticSourceV1,
) -> Result<Vec<MomentumPairedBrierDiagnosticV1>, String> {
    let mut output = Vec::new();
    for partition in included_partitions() {
        for participant_index in 1..5 {
            let values = paired_values(source, partition, participant_index)?
                .into_iter()
                .map(|(_, delta)| delta)
                .collect::<Vec<_>>();
            let sorted = sorted_finite(&values)?;
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let mut item = MomentumPairedBrierDiagnosticV1 {
                participant_id: participant_ids()[participant_index].clone(),
                partition,
                paired_event_count: values.len(),
                mean_delta: mean,
                median_delta: quantile_sorted(&sorted, 0.5)?,
                minimum_delta: sorted[0],
                maximum_delta: *sorted
                    .last()
                    .ok_or_else(|| "qualified-six paired maximum unavailable".to_string())?,
                positive_delta_count: values
                    .iter()
                    .filter(|value| **value > COMPARISON_EPSILON)
                    .count(),
                negative_delta_count: values
                    .iter()
                    .filter(|value| **value < -COMPARISON_EPSILON)
                    .count(),
                equivalent_delta_count: values
                    .iter()
                    .filter(|value| value.abs() <= COMPARISON_EPSILON)
                    .count(),
                finite_value_proof: mean.is_finite(),
                diagnostic_digest: String::new(),
            };
            item.diagnostic_digest = paired_digest(&item);
            output.push(item);
        }
    }
    Ok(output)
}

fn civil_year_month(days_since_epoch: i64) -> (i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month)
}

fn calendar_key(timestamp_ms: u64, grain: MomentumDiagnosticCalendarGrainV1) -> i64 {
    let day = (timestamp_ms / DAY_MS) as i64;
    match grain {
        MomentumDiagnosticCalendarGrainV1::UtcDay => day,
        MomentumDiagnosticCalendarGrainV1::UtcWeek => (day + 3).div_euclid(7),
        MomentumDiagnosticCalendarGrainV1::UtcMonth => {
            let (year, month) = civil_year_month(day);
            year * 12 + month - 1
        }
    }
}

fn longest_streak(
    relations: &[MomentumDiagnosticRelationV1],
    target: MomentumDiagnosticRelationV1,
) -> usize {
    let mut longest = 0usize;
    let mut current = 0usize;
    for value in relations {
        if *value == target {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

fn compute_calendar(
    source: &MomentumQualifiedDiagnosticSourceV1,
) -> Result<Vec<MomentumCalendarStabilityDiagnosticV1>, String> {
    let mut output = Vec::new();
    for partition in included_partitions() {
        for participant_index in 1..5 {
            let paired = paired_values(source, partition, participant_index)?;
            for grain in [
                MomentumDiagnosticCalendarGrainV1::UtcDay,
                MomentumDiagnosticCalendarGrainV1::UtcWeek,
                MomentumDiagnosticCalendarGrainV1::UtcMonth,
            ] {
                let mut grouped = BTreeMap::<i64, Vec<f64>>::new();
                for (timestamp, delta) in &paired {
                    grouped
                        .entry(calendar_key(*timestamp, grain))
                        .or_default()
                        .push(*delta);
                }
                let group_deltas = grouped
                    .values()
                    .map(|values| values.iter().sum::<f64>() / values.len() as f64)
                    .collect::<Vec<_>>();
                let group_relations = group_deltas
                    .iter()
                    .map(|delta| relation(*delta))
                    .collect::<Vec<_>>();
                let mut cumulative = 0.0;
                let trajectory = group_deltas
                    .iter()
                    .map(|delta| {
                        cumulative += delta;
                        cumulative.to_bits()
                    })
                    .collect::<Vec<_>>();
                let mut item = MomentumCalendarStabilityDiagnosticV1 {
                    participant_id: participant_ids()[participant_index].clone(),
                    partition,
                    grain,
                    group_count: group_deltas.len(),
                    lower_brier_group_count: group_relations
                        .iter()
                        .filter(|value| **value == MomentumDiagnosticRelationV1::LowerBrier)
                        .count(),
                    higher_brier_group_count: group_relations
                        .iter()
                        .filter(|value| **value == MomentumDiagnosticRelationV1::HigherBrier)
                        .count(),
                    equivalent_group_count: group_relations
                        .iter()
                        .filter(|value| {
                            **value == MomentumDiagnosticRelationV1::NumericallyEquivalent
                        })
                        .count(),
                    median_group_delta: median(&group_deltas)?,
                    worst_group_delta: group_deltas
                        .iter()
                        .copied()
                        .max_by(f64::total_cmp)
                        .ok_or_else(|| {
                            "qualified-six calendar worst group unavailable".to_string()
                        })?,
                    best_group_delta: group_deltas
                        .iter()
                        .copied()
                        .min_by(f64::total_cmp)
                        .ok_or_else(|| {
                            "qualified-six calendar best group unavailable".to_string()
                        })?,
                    longest_lower_brier_streak: longest_streak(
                        &group_relations,
                        MomentumDiagnosticRelationV1::LowerBrier,
                    ),
                    longest_higher_brier_streak: longest_streak(
                        &group_relations,
                        MomentumDiagnosticRelationV1::HigherBrier,
                    ),
                    cumulative_trajectory_digest: stable_hash_string(&format!(
                        "qualified-six-cumulative-trajectory-v1:{trajectory:?}"
                    )),
                    diagnostic_digest: String::new(),
                };
                item.diagnostic_digest = calendar_digest(&item);
                output.push(item);
            }
        }
    }
    Ok(output)
}

fn rolling_windows(values: &[(u64, f64)], window_size: usize, step: usize) -> Vec<&[(u64, f64)]> {
    if values.len() < window_size || window_size == 0 || step == 0 {
        return Vec::new();
    }
    (0..=values.len() - window_size)
        .step_by(step)
        .map(|start| &values[start..start + window_size])
        .collect()
}

fn compute_rolling(
    source: &MomentumQualifiedDiagnosticSourceV1,
) -> Result<Vec<MomentumRollingStabilityDiagnosticV1>, String> {
    let mut output = Vec::new();
    for partition in included_partitions() {
        for participant_index in 1..5 {
            let paired = paired_values(source, partition, participant_index)?;
            for window_size in [ROLLING_DAY_EVENTS, ROLLING_WEEK_EVENTS] {
                for (variant, step) in [
                    ("non-overlapping", window_size),
                    ("rolling-step-one", 1usize),
                ] {
                    let windows = rolling_windows(&paired, window_size, step);
                    let deltas = windows
                        .iter()
                        .map(|window| {
                            window.iter().map(|(_, delta)| delta).sum::<f64>() / window.len() as f64
                        })
                        .collect::<Vec<_>>();
                    let spans = windows
                        .iter()
                        .map(|window| {
                            window
                                .last()
                                .map(|(timestamp, _)| *timestamp)
                                .unwrap_or_default()
                                .saturating_sub(
                                    window
                                        .first()
                                        .map(|(timestamp, _)| *timestamp)
                                        .unwrap_or_default(),
                                )
                        })
                        .collect::<Vec<_>>();
                    let relations = deltas
                        .iter()
                        .map(|delta| relation(*delta))
                        .collect::<Vec<_>>();
                    let sign_change_count = relations
                        .windows(2)
                        .filter(|pair| {
                            matches!(
                                (pair[0], pair[1]),
                                (
                                    MomentumDiagnosticRelationV1::LowerBrier,
                                    MomentumDiagnosticRelationV1::HigherBrier
                                ) | (
                                    MomentumDiagnosticRelationV1::HigherBrier,
                                    MomentumDiagnosticRelationV1::LowerBrier
                                )
                            )
                        })
                        .count();
                    let mut item = MomentumRollingStabilityDiagnosticV1 {
                        participant_id: participant_ids()[participant_index].clone(),
                        partition,
                        window_event_count: window_size,
                        variant: variant.to_string(),
                        eligible_window_count: windows.len(),
                        lower_brier_window_count: relations
                            .iter()
                            .filter(|value| **value == MomentumDiagnosticRelationV1::LowerBrier)
                            .count(),
                        higher_brier_window_count: relations
                            .iter()
                            .filter(|value| **value == MomentumDiagnosticRelationV1::HigherBrier)
                            .count(),
                        equivalent_window_count: relations
                            .iter()
                            .filter(|value| {
                                **value == MomentumDiagnosticRelationV1::NumericallyEquivalent
                            })
                            .count(),
                        minimum_window_delta: deltas
                            .iter()
                            .copied()
                            .min_by(f64::total_cmp)
                            .ok_or_else(|| {
                                "qualified-six rolling minimum unavailable".to_string()
                            })?,
                        maximum_window_delta: deltas
                            .iter()
                            .copied()
                            .max_by(f64::total_cmp)
                            .ok_or_else(|| {
                                "qualified-six rolling maximum unavailable".to_string()
                            })?,
                        median_window_delta: median(&deltas)?,
                        sign_change_count,
                        minimum_timestamp_span_ms: spans.iter().copied().min().ok_or_else(
                            || "qualified-six rolling span minimum unavailable".to_string(),
                        )?,
                        maximum_timestamp_span_ms: spans.iter().copied().max().ok_or_else(
                            || "qualified-six rolling span maximum unavailable".to_string(),
                        )?,
                        median_timestamp_span_ms: median_u64(&spans)?,
                        diagnostic_digest: String::new(),
                    };
                    item.diagnostic_digest = rolling_digest(&item);
                    output.push(item);
                }
            }
        }
    }
    Ok(output)
}

fn calibration_bin_index(probability: f64) -> Result<usize, String> {
    if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
        return Err("qualified-six calibration probability rejected".to_string());
    }
    Ok(if probability == 1.0 {
        9
    } else {
        (probability * 10.0).floor() as usize
    })
}

fn compute_calibration(
    source: &MomentumQualifiedDiagnosticSourceV1,
) -> Result<Vec<MomentumCalibrationDiagnosticV1>, String> {
    let mut output = Vec::new();
    for partition in included_partitions() {
        let events = events_for_partition(source, partition);
        for participant_index in 0..5 {
            let mut probabilities = vec![Vec::<f64>::new(); 10];
            let mut labels = vec![Vec::<f64>::new(); 10];
            for event in &events {
                if !scorable_event(event)? {
                    continue;
                }
                let probability = event.probabilities[participant_index];
                let index = calibration_bin_index(probability)?;
                probabilities[index].push(probability);
                labels[index].push(
                    event
                        .label
                        .ok_or_else(|| "qualified-six calibration label unavailable".to_string())?,
                );
            }
            let total = probabilities.iter().map(Vec::len).sum::<usize>();
            if total == 0 {
                return Err("qualified-six calibration support unavailable".to_string());
            }
            let mut weighted_gap = 0.0;
            let bins = (0..10)
                .map(|index| {
                    let support = probabilities[index].len();
                    let mean_probability = (support > 0)
                        .then(|| probabilities[index].iter().sum::<f64>() / support as f64);
                    let positive_frequency =
                        (support > 0).then(|| labels[index].iter().sum::<f64>() / support as f64);
                    let gap = mean_probability
                        .zip(positive_frequency)
                        .map(|(prediction, observed)| (prediction - observed).abs());
                    if let Some(gap) = gap {
                        weighted_gap += gap * support as f64 / total as f64;
                    }
                    MomentumCalibrationBinDiagnosticV1 {
                        lower_bound: CALIBRATION_BOUNDARIES[index],
                        upper_bound: CALIBRATION_BOUNDARIES[index + 1],
                        upper_inclusive: index == 9,
                        support,
                        mean_predicted_probability: mean_probability,
                        observed_positive_frequency: positive_frequency,
                        absolute_calibration_gap: gap,
                    }
                })
                .collect::<Vec<_>>();
            let mut item = MomentumCalibrationDiagnosticV1 {
                participant_id: participant_ids()[participant_index].clone(),
                partition,
                empty_bin_count: bins.iter().filter(|bin| bin.support == 0).count(),
                bins,
                weighted_aggregate_calibration_gap: weighted_gap,
                finite_value_proof: weighted_gap.is_finite(),
                diagnostic_digest: String::new(),
            };
            item.diagnostic_digest = calibration_digest(&item);
            output.push(item);
        }
    }
    Ok(output)
}

fn compute_probability_distributions(
    source: &MomentumQualifiedDiagnosticSourceV1,
) -> Result<Vec<MomentumProbabilityDistributionDiagnosticV1>, String> {
    let mut output = Vec::new();
    for partition in included_partitions() {
        let events = events_for_partition(source, partition);
        for participant_index in 0..5 {
            let raw = events
                .iter()
                .map(|event| event.probabilities[participant_index])
                .collect::<Vec<_>>();
            let nonfinite_count = raw.iter().filter(|value| !value.is_finite()).count();
            let sorted = sorted_finite(&raw)?;
            let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
            let variance = sorted
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / sorted.len() as f64;
            let mut frequencies = BTreeMap::<u64, usize>::new();
            for value in &sorted {
                *frequencies.entry(value.to_bits()).or_default() += 1;
            }
            let exact_constant_value_count =
                frequencies.values().copied().max().unwrap_or_default();
            let extreme_low_count = sorted
                .iter()
                .filter(|value| **value <= PREDICTION_CLAMP)
                .count();
            let extreme_high_count = sorted
                .iter()
                .filter(|value| **value >= 1.0 - PREDICTION_CLAMP)
                .count();
            let saturation_status = match (extreme_low_count > 0, extreme_high_count > 0) {
                (false, false) => MomentumQualifiedSaturationStatusV1::NotSaturated,
                (true, false) => MomentumQualifiedSaturationStatusV1::LowBoundarySaturation,
                (false, true) => MomentumQualifiedSaturationStatusV1::HighBoundarySaturation,
                (true, true) => MomentumQualifiedSaturationStatusV1::TwoSidedSaturation,
            };
            let collapse_status = if participant_index == 0 {
                MomentumQualifiedProbabilityCollapseStatusV1::BenchmarkExempt
            } else if variance <= COLLAPSE_VARIANCE_THRESHOLD {
                MomentumQualifiedProbabilityCollapseStatusV1::ProbabilityCollapse
            } else {
                MomentumQualifiedProbabilityCollapseStatusV1::NotCollapsed
            };
            let mut item = MomentumProbabilityDistributionDiagnosticV1 {
                participant_id: participant_ids()[participant_index].clone(),
                partition,
                minimum: sorted[0],
                percentile_01: quantile_sorted(&sorted, 0.01)?,
                percentile_05: quantile_sorted(&sorted, 0.05)?,
                percentile_25: quantile_sorted(&sorted, 0.25)?,
                median: quantile_sorted(&sorted, 0.50)?,
                percentile_75: quantile_sorted(&sorted, 0.75)?,
                percentile_95: quantile_sorted(&sorted, 0.95)?,
                percentile_99: quantile_sorted(&sorted, 0.99)?,
                maximum: *sorted
                    .last()
                    .ok_or_else(|| "qualified-six probability maximum unavailable".to_string())?,
                mean,
                standard_deviation: variance.sqrt(),
                exact_constant_value_count,
                near_half_count: sorted
                    .iter()
                    .filter(|value| (*value - 0.5).abs() <= NEAR_HALF_THRESHOLD)
                    .count(),
                extreme_low_count,
                extreme_high_count,
                nonfinite_count,
                saturation_status,
                collapse_status,
                diagnostic_digest: String::new(),
            };
            item.diagnostic_digest = probability_digest(&item);
            output.push(item);
        }
    }
    Ok(output)
}

fn compute_prevalence(
    source: &MomentumQualifiedDiagnosticSourceV1,
) -> Result<Vec<MomentumPrevalenceDriftDiagnosticV1>, String> {
    let mut output = Vec::new();
    for partition in included_partitions() {
        let events = events_for_partition(source, partition);
        let scorable = events
            .iter()
            .filter_map(|event| {
                event
                    .label
                    .map(|label| (event.prediction_timestamp_ms, label))
            })
            .collect::<Vec<_>>();
        let positive_count = scorable.iter().filter(|(_, label)| *label == 1.0).count();
        let partition_prevalence = positive_count as f64 / scorable.len() as f64;
        for grain in [
            MomentumDiagnosticCalendarGrainV1::UtcDay,
            MomentumDiagnosticCalendarGrainV1::UtcWeek,
            MomentumDiagnosticCalendarGrainV1::UtcMonth,
        ] {
            let mut grouped = BTreeMap::<i64, Vec<f64>>::new();
            for (timestamp, label) in &scorable {
                grouped
                    .entry(calendar_key(*timestamp, grain))
                    .or_default()
                    .push(*label);
            }
            let prevalences = grouped
                .values()
                .map(|labels| labels.iter().sum::<f64>() / labels.len() as f64)
                .collect::<Vec<_>>();
            let deviations = prevalences
                .iter()
                .map(|value| *value - partition_prevalence)
                .collect::<Vec<_>>();
            let trajectory = prevalences
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>();
            let mut item = MomentumPrevalenceDriftDiagnosticV1 {
                partition,
                grain,
                scorable_count: scorable.len(),
                positive_count,
                negative_count: scorable.len() - positive_count,
                partition_positive_prevalence: partition_prevalence,
                group_count: prevalences.len(),
                minimum_group_prevalence: prevalences
                    .iter()
                    .copied()
                    .min_by(f64::total_cmp)
                    .ok_or_else(|| "qualified-six prevalence minimum unavailable".to_string())?,
                maximum_group_prevalence: prevalences
                    .iter()
                    .copied()
                    .max_by(f64::total_cmp)
                    .ok_or_else(|| "qualified-six prevalence maximum unavailable".to_string())?,
                minimum_deviation: deviations
                    .iter()
                    .copied()
                    .min_by(f64::total_cmp)
                    .ok_or_else(|| {
                        "qualified-six prevalence deviation minimum unavailable".to_string()
                    })?,
                maximum_deviation: deviations
                    .iter()
                    .copied()
                    .max_by(f64::total_cmp)
                    .ok_or_else(|| {
                        "qualified-six prevalence deviation maximum unavailable".to_string()
                    })?,
                prevalence_trajectory_digest: stable_hash_string(&format!(
                    "qualified-six-prevalence-trajectory-v1:{trajectory:?}"
                )),
                diagnostic_digest: String::new(),
            };
            item.diagnostic_digest = prevalence_digest(&item);
            output.push(item);
        }
    }
    Ok(output)
}

fn derive_regime_thresholds(
    source: &MomentumQualifiedDiagnosticSourceV1,
    registration: &MomentumQualifiedSixDiagnosticRegistrationV1,
) -> Result<MomentumQualifiedRegimeThresholdReceiptV1, String> {
    let values = source
        .events
        .iter()
        .filter(|event| event.partition == MomentumReplayPartitionV1::Development)
        .filter_map(|event| event.micro_volatility)
        .collect::<Vec<_>>();
    let sorted = sorted_finite(&values)?;
    let mut value = MomentumQualifiedRegimeThresholdReceiptV1 {
        threshold_version: THRESHOLD_VERSION.to_string(),
        source_diagnostic_registration_digest: registration.registration_digest.clone(),
        development_event_count: values.len(),
        low_volatility_upper: quantile_sorted(&sorted, 1.0 / 3.0)?,
        medium_volatility_upper: quantile_sorted(&sorted, 2.0 / 3.0)?,
        validation_value_access_count: 0,
        threshold_digest: String::new(),
    };
    value.threshold_digest = threshold_digest(&value);
    validate_threshold(&value)?;
    Ok(value)
}

fn volatility_regime(
    value: f64,
    thresholds: &MomentumQualifiedRegimeThresholdReceiptV1,
) -> MomentumQualifiedVolatilityRegimeV1 {
    if value <= thresholds.low_volatility_upper {
        MomentumQualifiedVolatilityRegimeV1::LowVolatility
    } else if value <= thresholds.medium_volatility_upper {
        MomentumQualifiedVolatilityRegimeV1::MediumVolatility
    } else {
        MomentumQualifiedVolatilityRegimeV1::HighVolatility
    }
}

fn trend_regime(value: f64) -> MomentumQualifiedDailyTrendRegimeV1 {
    if value < -COMPARISON_EPSILON {
        MomentumQualifiedDailyTrendRegimeV1::DownTrend
    } else if value > COMPARISON_EPSILON {
        MomentumQualifiedDailyTrendRegimeV1::UpTrend
    } else {
        MomentumQualifiedDailyTrendRegimeV1::Flat
    }
}

fn compute_one_regime(
    participant_index: usize,
    partition: MomentumReplayPartitionV1,
    regime_dimension: &str,
    volatility: Option<MomentumQualifiedVolatilityRegimeV1>,
    trend: Option<MomentumQualifiedDailyTrendRegimeV1>,
    events: &[&MomentumQualifiedDiagnosticEventEvidenceV1],
) -> Result<MomentumRegimeMetricDiagnosticV1, String> {
    let scorable = events
        .iter()
        .filter(|event| event.label.is_some())
        .copied()
        .collect::<Vec<_>>();
    let support = scorable.len();
    let brier = (support > 0).then(|| {
        scorable
            .iter()
            .map(|event| event.brier_values[participant_index])
            .sum::<f64>()
            / support as f64
    });
    let correctness = (support > 0).then(|| {
        scorable
            .iter()
            .filter(|event| event.correctness[participant_index])
            .count() as f64
            / support as f64
    });
    let delta = (support > 0).then(|| {
        scorable
            .iter()
            .map(|event| event.brier_values[participant_index] - event.brier_values[0])
            .sum::<f64>()
            / support as f64
    });
    let relation = if support < REGIME_MINIMUM_SUPPORT {
        MomentumDiagnosticRelationV1::InsufficientDiagnosticSupport
    } else {
        delta
            .map(relation)
            .unwrap_or(MomentumDiagnosticRelationV1::IntegrityFailure)
    };
    let mut item = MomentumRegimeMetricDiagnosticV1 {
        participant_id: participant_ids()[participant_index].clone(),
        partition,
        regime_dimension: regime_dimension.to_string(),
        volatility_regime: volatility,
        daily_trend_regime: trend,
        event_count: events.len(),
        scorable_count: support,
        neutral_count: events.len() - support,
        mean_brier: brier,
        correctness,
        paired_brier_delta_versus_q0: delta,
        relation,
        finite_value_proof: brier.is_none_or(f64::is_finite)
            && correctness.is_none_or(f64::is_finite)
            && delta.is_none_or(f64::is_finite),
        diagnostic_digest: String::new(),
    };
    item.diagnostic_digest = regime_digest(&item);
    Ok(item)
}

fn compute_regimes(
    source: &MomentumQualifiedDiagnosticSourceV1,
    thresholds: &MomentumQualifiedRegimeThresholdReceiptV1,
) -> Result<Vec<MomentumRegimeMetricDiagnosticV1>, String> {
    let mut output = Vec::new();
    for partition in included_partitions() {
        let events = events_for_partition(source, partition)
            .into_iter()
            .filter(|event| event.micro_volatility.is_some() && event.daily_trend_return.is_some())
            .collect::<Vec<_>>();
        for participant_index in 0..5 {
            for volatility in [
                MomentumQualifiedVolatilityRegimeV1::LowVolatility,
                MomentumQualifiedVolatilityRegimeV1::MediumVolatility,
                MomentumQualifiedVolatilityRegimeV1::HighVolatility,
            ] {
                let selected = events
                    .iter()
                    .copied()
                    .filter(|event| {
                        volatility_regime(event.micro_volatility.unwrap_or_default(), thresholds)
                            == volatility
                    })
                    .collect::<Vec<_>>();
                output.push(compute_one_regime(
                    participant_index,
                    partition,
                    "micro-volatility",
                    Some(volatility),
                    None,
                    &selected,
                )?);
            }
            for trend in [
                MomentumQualifiedDailyTrendRegimeV1::DownTrend,
                MomentumQualifiedDailyTrendRegimeV1::Flat,
                MomentumQualifiedDailyTrendRegimeV1::UpTrend,
            ] {
                let selected = events
                    .iter()
                    .copied()
                    .filter(|event| {
                        trend_regime(event.daily_trend_return.unwrap_or_default()) == trend
                    })
                    .collect::<Vec<_>>();
                output.push(compute_one_regime(
                    participant_index,
                    partition,
                    "daily-trend",
                    None,
                    Some(trend),
                    &selected,
                )?);
            }
            for volatility in [
                MomentumQualifiedVolatilityRegimeV1::LowVolatility,
                MomentumQualifiedVolatilityRegimeV1::MediumVolatility,
                MomentumQualifiedVolatilityRegimeV1::HighVolatility,
            ] {
                for trend in [
                    MomentumQualifiedDailyTrendRegimeV1::DownTrend,
                    MomentumQualifiedDailyTrendRegimeV1::Flat,
                    MomentumQualifiedDailyTrendRegimeV1::UpTrend,
                ] {
                    let selected = events
                        .iter()
                        .copied()
                        .filter(|event| {
                            volatility_regime(
                                event.micro_volatility.unwrap_or_default(),
                                thresholds,
                            ) == volatility
                                && trend_regime(event.daily_trend_return.unwrap_or_default())
                                    == trend
                        })
                        .collect::<Vec<_>>();
                    output.push(compute_one_regime(
                        participant_index,
                        partition,
                        "volatility-x-daily-trend",
                        Some(volatility),
                        Some(trend),
                        &selected,
                    )?);
                }
            }
        }
    }
    Ok(output)
}

fn participant_normalizer_indices(participant_index: usize) -> &'static [usize] {
    match participant_index {
        1 => &[3],
        2 => &[0, 1, 2, 3],
        3 => &[4, 5],
        4 => &[0, 1, 2, 3, 4, 5],
        _ => &[],
    }
}

fn profile_shift(
    left: &MomentumQualifiedDiagnosticRefitEvidenceV1,
    right: &MomentumQualifiedDiagnosticRefitEvidenceV1,
    participant_index: usize,
) -> Result<f64, String> {
    let mut squared = 0.0;
    let mut count = 0usize;
    for index in participant_normalizer_indices(participant_index) {
        let left_profile = left
            .normalizer_profiles
            .get(*index)
            .ok_or_else(|| "qualified-six normalizer profile unavailable".to_string())?;
        let right_profile = right
            .normalizer_profiles
            .get(*index)
            .ok_or_else(|| "qualified-six normalizer profile unavailable".to_string())?;
        if left_profile.len() != right_profile.len() {
            return Err("qualified-six normalizer profile mismatch".to_string());
        }
        for (left, right) in left_profile.iter().zip(right_profile) {
            let delta = right - left;
            squared += delta * delta;
            count += 1;
        }
    }
    if count == 0 {
        return Err("qualified-six normalizer profile empty".to_string());
    }
    let shift = (squared / count as f64).sqrt();
    if !shift.is_finite() {
        return Err("qualified-six normalizer shift nonfinite".to_string());
    }
    Ok(shift)
}

fn relative_change(left: f64, right: f64) -> f64 {
    (right - left).abs() / left.abs().max(COMPARISON_EPSILON)
}

fn daily_prediction_dispersion(
    source: &MomentumQualifiedDiagnosticSourceV1,
    partition: MomentumReplayPartitionV1,
    participant_index: usize,
) -> Result<BTreeMap<String, f64>, String> {
    let mut grouped = BTreeMap::<String, Vec<f64>>::new();
    for event in source
        .events
        .iter()
        .filter(|event| event.partition == partition)
    {
        grouped
            .entry(event.daily_refit_receipt_digest.clone())
            .or_default()
            .push(event.probabilities[participant_index]);
    }
    grouped
        .into_iter()
        .map(|(digest, values)| {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance = values
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / values.len() as f64;
            let dispersion = variance.sqrt();
            if !dispersion.is_finite() {
                return Err("qualified-six daily dispersion nonfinite".to_string());
            }
            Ok((digest, dispersion))
        })
        .collect()
}

fn metric_for<'a>(
    header: &'a MomentumQualifiedDiagnosticSourceHeaderV1,
    partition: MomentumReplayPartitionV1,
    participant_id: &str,
) -> Result<&'a MomentumQualifiedParticipantMetricsV1, String> {
    let metrics = match partition {
        MomentumReplayPartitionV1::Development => &header.development_metrics,
        MomentumReplayPartitionV1::Validation => &header.validation_metrics,
        MomentumReplayPartitionV1::SealedHoldout => {
            return Err("qualified-six holdout metrics forbidden".to_string());
        }
    };
    metrics
        .iter()
        .find(|metrics| metrics.participant_id == participant_id)
        .ok_or_else(|| "qualified-six participant metrics unavailable".to_string())
}

fn compute_model_drift(
    source: &MomentumQualifiedDiagnosticSourceV1,
) -> Result<Vec<MomentumModelDriftDiagnosticV1>, String> {
    let mut output = Vec::new();
    for partition in included_partitions() {
        let partition_refits = source
            .refits
            .iter()
            .filter(|refit| refit.partition == partition)
            .collect::<Vec<_>>();
        if partition_refits.is_empty() {
            return Err("qualified-six model drift refits unavailable".to_string());
        }
        for participant_index in 1..5 {
            let participant_id = participant_ids()[participant_index].clone();
            let parameter_changes = partition_refits
                .windows(2)
                .map(|pair| {
                    relative_change(
                        pair[0].parameter_norms[participant_index],
                        pair[1].parameter_norms[participant_index],
                    )
                })
                .collect::<Vec<_>>();
            let normalizer_shifts = partition_refits
                .windows(2)
                .map(|pair| profile_shift(pair[0], pair[1], participant_index))
                .collect::<Result<Vec<_>, _>>()?;
            let dispersions = daily_prediction_dispersion(source, partition, participant_index)?;
            let daily_dispersions = partition_refits
                .iter()
                .map(|refit| {
                    dispersions
                        .get(&refit.refit_digest)
                        .copied()
                        .ok_or_else(|| {
                            "qualified-six daily refit dispersion binding unavailable".to_string()
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let previous_partition_last = if partition == MomentumReplayPartitionV1::Validation {
                source
                    .refits
                    .iter()
                    .filter(|refit| refit.partition == MomentumReplayPartitionV1::Development)
                    .last()
            } else {
                None
            };
            let boundary_shift = previous_partition_last
                .zip(partition_refits.first().copied())
                .is_some_and(|(left, right)| {
                    relative_change(
                        left.parameter_norms[participant_index],
                        right.parameter_norms[participant_index],
                    ) > HIGH_DRIFT_THRESHOLD
                });
            let parameter_finite = partition_refits.iter().all(|refit| refit.parameter_finite)
                && parameter_changes.iter().all(|value| value.is_finite());
            let normalizer_finite = partition_refits.iter().all(|refit| refit.normalizer_finite)
                && normalizer_shifts.iter().all(|value| value.is_finite());
            let training_loss_finite = partition_refits
                .iter()
                .all(|refit| refit.training_loss_finite);
            let probability_collapsed =
                metric_for(&source.header, partition, &participant_id)?.probability_collapsed;
            let maximum_parameter_change = parameter_changes
                .iter()
                .copied()
                .max_by(f64::total_cmp)
                .unwrap_or(0.0);
            let maximum_normalizer_shift = normalizer_shifts
                .iter()
                .copied()
                .max_by(f64::total_cmp)
                .unwrap_or(0.0);
            let maximum_drift = maximum_parameter_change.max(maximum_normalizer_shift);
            let status = if !parameter_finite || !normalizer_finite || !training_loss_finite {
                MomentumQualifiedModelDriftStatusV1::IntegrityFailure
            } else if probability_collapsed {
                MomentumQualifiedModelDriftStatusV1::ProbabilityCollapse
            } else if boundary_shift {
                MomentumQualifiedModelDriftStatusV1::PartitionBoundaryShift
            } else if maximum_drift > HIGH_DRIFT_THRESHOLD {
                MomentumQualifiedModelDriftStatusV1::HighDeterministicDrift
            } else if maximum_drift > MODERATE_DRIFT_THRESHOLD {
                MomentumQualifiedModelDriftStatusV1::ModerateDeterministicDrift
            } else {
                MomentumQualifiedModelDriftStatusV1::StableAcrossRefits
            };
            let parameter_trajectory = partition_refits
                .iter()
                .map(|refit| refit.parameter_digests[participant_index].clone())
                .collect::<Vec<_>>();
            let normalizer_trajectory = partition_refits
                .iter()
                .map(|refit| {
                    participant_normalizer_indices(participant_index)
                        .iter()
                        .map(|index| refit.normalizer_digests[*index].clone())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let training_counts = partition_refits
                .iter()
                .map(|refit| refit.training_example_count)
                .collect::<Vec<_>>();
            let prevalences = partition_refits
                .iter()
                .map(|refit| refit.training_class_prevalence)
                .collect::<Vec<_>>();
            let mut item = MomentumModelDriftDiagnosticV1 {
                participant_id,
                partition,
                refit_count: partition_refits.len(),
                parameter_digest_trajectory: stable_hash_string(&format!(
                    "qualified-six-parameter-trajectory-v1:{parameter_trajectory:?}"
                )),
                normalizer_digest_trajectory: stable_hash_string(&format!(
                    "qualified-six-normalizer-trajectory-v1:{normalizer_trajectory:?}"
                )),
                parameter_finite,
                normalizer_finite,
                training_loss_finite,
                minimum_training_example_count: training_counts
                    .iter()
                    .copied()
                    .min()
                    .unwrap_or_default(),
                maximum_training_example_count: training_counts
                    .iter()
                    .copied()
                    .max()
                    .unwrap_or_default(),
                minimum_training_prevalence: prevalences
                    .iter()
                    .copied()
                    .min_by(f64::total_cmp)
                    .ok_or_else(|| {
                        "qualified-six training prevalence minimum unavailable".to_string()
                    })?,
                maximum_training_prevalence: prevalences
                    .iter()
                    .copied()
                    .max_by(f64::total_cmp)
                    .ok_or_else(|| {
                        "qualified-six training prevalence maximum unavailable".to_string()
                    })?,
                median_parameter_norm_change: if parameter_changes.is_empty() {
                    0.0
                } else {
                    median(&parameter_changes)?
                },
                maximum_parameter_norm_change: maximum_parameter_change,
                median_normalizer_shift: if normalizer_shifts.is_empty() {
                    0.0
                } else {
                    median(&normalizer_shifts)?
                },
                maximum_normalizer_shift,
                median_daily_prediction_dispersion: median(&daily_dispersions)?,
                partition_boundary_shift: boundary_shift,
                status,
                diagnostic_digest: String::new(),
            };
            item.diagnostic_digest = model_drift_digest(&item);
            output.push(item);
        }
    }
    Ok(output)
}

fn metric_delta(
    header: &MomentumQualifiedDiagnosticSourceHeaderV1,
    partition: MomentumReplayPartitionV1,
    participant_id: &str,
) -> Result<f64, String> {
    metric_for(header, partition, participant_id)?
        .delta_versus_constant
        .ok_or_else(|| "qualified-six aggregate delta unavailable".to_string())
}

fn classify_partition_stability(
    development_delta: f64,
    validation_delta: f64,
) -> MomentumQualifiedPartitionStabilityV1 {
    if !development_delta.is_finite() || !validation_delta.is_finite() {
        MomentumQualifiedPartitionStabilityV1::IntegrityFailure
    } else if development_delta < -COMPARISON_EPSILON && validation_delta < -COMPARISON_EPSILON {
        MomentumQualifiedPartitionStabilityV1::LowerBrierAcrossDevelopmentAndValidation
    } else if development_delta < -COMPARISON_EPSILON && validation_delta >= -COMPARISON_EPSILON {
        MomentumQualifiedPartitionStabilityV1::DevelopmentOnlyLowerBrier
    } else if validation_delta < -COMPARISON_EPSILON && development_delta >= -COMPARISON_EPSILON {
        MomentumQualifiedPartitionStabilityV1::ValidationOnlyLowerBrier
    } else if development_delta > COMPARISON_EPSILON && validation_delta > COMPARISON_EPSILON {
        MomentumQualifiedPartitionStabilityV1::HigherBrierAcrossDevelopmentAndValidation
    } else if development_delta.abs() <= COMPARISON_EPSILON
        && validation_delta.abs() <= COMPARISON_EPSILON
    {
        MomentumQualifiedPartitionStabilityV1::NumericallyEquivalentAcrossPartitions
    } else {
        MomentumQualifiedPartitionStabilityV1::MixedOrInsufficientEvidence
    }
}

fn compute_partition_stability(
    header: &MomentumQualifiedDiagnosticSourceHeaderV1,
) -> Result<Vec<MomentumPartitionStabilityReceiptV1>, String> {
    learned_participant_ids()
        .into_iter()
        .map(|participant_id| {
            let mut value = MomentumPartitionStabilityReceiptV1 {
                classification: classify_partition_stability(
                    metric_delta(
                        header,
                        MomentumReplayPartitionV1::Development,
                        &participant_id,
                    )?,
                    metric_delta(
                        header,
                        MomentumReplayPartitionV1::Validation,
                        &participant_id,
                    )?,
                ),
                participant_id,
                receipt_digest: String::new(),
            };
            value.receipt_digest = partition_stability_digest(&value);
            Ok(value)
        })
        .collect()
}

fn compute_holdout_eligibility(
    header: &MomentumQualifiedDiagnosticSourceHeaderV1,
) -> Result<Vec<MomentumHoldoutEligibilityReceiptV1>, String> {
    learned_participant_ids()
        .into_iter()
        .map(|participant_id| {
            let development = metric_for(
                header,
                MomentumReplayPartitionV1::Development,
                &participant_id,
            )?;
            let validation = metric_for(
                header,
                MomentumReplayPartitionV1::Validation,
                &participant_id,
            )?;
            let lower_brier_development = development
                .delta_versus_constant
                .is_some_and(|delta| delta < -COMPARISON_EPSILON);
            let lower_brier_validation = validation
                .delta_versus_constant
                .is_some_and(|delta| delta < -COMPARISON_EPSILON);
            let sufficient_paired_support = development.paired_scorable_count
                >= REGIME_MINIMUM_SUPPORT
                && validation.paired_scorable_count >= REGIME_MINIMUM_SUPPORT;
            let finite_predictions_and_metrics = development.finite_prediction_count
                == development.total_prediction_events
                && validation.finite_prediction_count == validation.total_prediction_events
                && development.mean_brier_score.is_some_and(f64::is_finite)
                && validation.mean_brier_score.is_some_and(f64::is_finite);
            let probability_collapse_absent =
                !development.probability_collapsed && !validation.probability_collapsed;
            let chronology_and_leakage_passed = development.chronology_audit_passed
                && development.leakage_audit_passed
                && validation.chronology_audit_passed
                && validation.leakage_audit_passed;
            let integrity_passed = header.chronology_audit_passed
                && header.leakage_audit_passed
                && header.holdout_label_reads == 0
                && header.holdout_metric_computations == 0
                && header.holdout_participant_predictions == 0;
            let source_replay_unmutated = true;
            let eligible = lower_brier_development
                && lower_brier_validation
                && sufficient_paired_support
                && finite_predictions_and_metrics
                && probability_collapse_absent
                && chronology_and_leakage_passed
                && integrity_passed
                && source_replay_unmutated;
            let mut value = MomentumHoldoutEligibilityReceiptV1 {
                participant_id,
                lower_brier_development,
                lower_brier_validation,
                sufficient_paired_support,
                finite_predictions_and_metrics,
                probability_collapse_absent,
                chronology_and_leakage_passed,
                integrity_passed,
                source_replay_unmutated,
                eligibility: if eligible {
                    MomentumQualifiedHoldoutEligibilityV1::EligibleForFutureSealedHoldoutEvaluation
                } else {
                    MomentumQualifiedHoldoutEligibilityV1::NotEligibleForSealedHoldout
                },
                receipt_digest: String::new(),
            };
            value.receipt_digest = eligibility_digest(&value);
            Ok(value)
        })
        .collect()
}

fn priority_from_stability(
    classification: MomentumQualifiedPartitionStabilityV1,
) -> MomentumResearchPriorityV1 {
    match classification {
        MomentumQualifiedPartitionStabilityV1::DevelopmentOnlyLowerBrier
        | MomentumQualifiedPartitionStabilityV1::ValidationOnlyLowerBrier => {
            MomentumResearchPriorityV1::PrimaryDiagnosticTarget
        }
        MomentumQualifiedPartitionStabilityV1::LowerBrierAcrossDevelopmentAndValidation => {
            MomentumResearchPriorityV1::SecondaryDiagnosticTarget
        }
        MomentumQualifiedPartitionStabilityV1::HigherBrierAcrossDevelopmentAndValidation => {
            MomentumResearchPriorityV1::DeprioritizedByCurrentEvidence
        }
        MomentumQualifiedPartitionStabilityV1::NumericallyEquivalentAcrossPartitions
        | MomentumQualifiedPartitionStabilityV1::MixedOrInsufficientEvidence => {
            MomentumResearchPriorityV1::SecondaryDiagnosticTarget
        }
        MomentumQualifiedPartitionStabilityV1::IntegrityFailure => {
            MomentumResearchPriorityV1::BlockedByUnresolvedEvidence
        }
    }
}

fn build_challenger_requirements(
    registration: &MomentumQualifiedSixDiagnosticRegistrationV1,
    header: &MomentumQualifiedDiagnosticSourceHeaderV1,
    stability: &[MomentumPartitionStabilityReceiptV1],
) -> Result<MomentumQualifiedChallengerRequirementsV1, String> {
    let q2 = stability
        .iter()
        .find(|receipt| {
            receipt.participant_id == MomentumQualifiedParticipantV1::Q2MicroBlockLogistic.id()
        })
        .ok_or_else(|| "qualified-six Q2 stability unavailable".to_string())?;
    let macro_addition = header
        .contribution_comparisons
        .iter()
        .find(|receipt| {
            receipt.added_participant_id
                == MomentumQualifiedParticipantV1::Q4QualifiedSixFusionLogistic.id()
                && receipt.baseline_participant_id
                    == MomentumQualifiedParticipantV1::Q2MicroBlockLogistic.id()
        })
        .map(|receipt| match receipt.status {
            MomentumQualifiedContributionStatusV1::HigherBrierWithAddedBlock => {
                MomentumResearchPriorityV1::DeprioritizedByCurrentEvidence
            }
            MomentumQualifiedContributionStatusV1::LowerBrierWithAddedBlock => {
                MomentumResearchPriorityV1::SecondaryDiagnosticTarget
            }
            MomentumQualifiedContributionStatusV1::NumericallyEquivalent
            | MomentumQualifiedContributionStatusV1::MixedAcrossPartitions
            | MomentumQualifiedContributionStatusV1::InsufficientPairedValidation => {
                MomentumResearchPriorityV1::SecondaryDiagnosticTarget
            }
            MomentumQualifiedContributionStatusV1::IntegrityFailure => {
                MomentumResearchPriorityV1::BlockedByUnresolvedEvidence
            }
        })
        .ok_or_else(|| "qualified-six macro contribution unavailable".to_string())?;
    let mut value = MomentumQualifiedChallengerRequirementsV1 {
        requirements_version: REQUIREMENTS_VERSION.to_string(),
        source_diagnostic_registration_digest: registration.registration_digest.clone(),
        source_diagnostic_report_digest: String::new(),
        constant_benchmark_mandatory: true,
        full_eight_claim_forbidden: true,
        month_year_use_forbidden: true,
        complexity_escalation_allowed: false,
        interaction_expansion_allowed: false,
        sequence_model_allowed: false,
        micro_block_research_priority: priority_from_stability(q2.classification),
        qualified_macro_addition_priority: macro_addition,
        label_forensics_required: true,
        calibration_repair_required: true,
        regime_stability_required: true,
        two_partition_improvement_required: true,
        new_model_execution_authorized: false,
        holdout_execution_authorized: false,
        requirements_digest: String::new(),
    };
    value.requirements_digest = requirements_digest(&value);
    validate_requirements(&value)?;
    Ok(value)
}

fn validate_requirements(value: &MomentumQualifiedChallengerRequirementsV1) -> Result<(), String> {
    if value.requirements_version != REQUIREMENTS_VERSION
        || value.source_diagnostic_registration_digest.is_empty()
        || !value.constant_benchmark_mandatory
        || !value.full_eight_claim_forbidden
        || !value.month_year_use_forbidden
        || value.complexity_escalation_allowed
        || value.interaction_expansion_allowed
        || value.sequence_model_allowed
        || !value.label_forensics_required
        || !value.calibration_repair_required
        || !value.regime_stability_required
        || !value.two_partition_improvement_required
        || value.new_model_execution_authorized
        || value.holdout_execution_authorized
        || value.requirements_digest != requirements_digest(value)
    {
        return Err("qualified-six challenger requirements rejected".to_string());
    }
    Ok(())
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

fn read_single<T>(
    category: &str,
    decode: impl Fn(&[u8]) -> Result<T, String>,
) -> Result<Option<T>, String> {
    let root = Path::new(ROOT).join(category);
    if !root.exists() {
        return Ok(None);
    }
    let mut paths = fs::read_dir(root)
        .map_err(|_| "qualified-six diagnostic artifact directory read failed".to_string())?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|_| "qualified-six diagnostic artifact entry read failed".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| path.extension().is_some_and(|extension| extension == "pb"));
    paths.sort();
    if paths.len() > 1 {
        return Err("qualified-six diagnostic singleton artifact conflict".to_string());
    }
    paths
        .first()
        .map(|path| {
            fs::read(path)
                .map_err(|_| "qualified-six diagnostic artifact read failed".to_string())
                .and_then(|bytes| decode(&bytes))
        })
        .transpose()
}

fn add_counts(total: &mut (usize, usize), next: (usize, usize)) {
    total.0 += next.0;
    total.1 += next.1;
}

fn encode_policy(value: &MomentumQualifiedDiagnosticPolicyV1) -> Result<Vec<u8>, String> {
    validate_policy(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedDiagnosticPolicyV1")
        .string("policy_version", &value.policy_version)
        .string("policy_name", &value.policy_name)
        .strings("frozen_values", &value.frozen_values)
        .string("policy_digest", &value.policy_digest)
        .encode()
}

fn decode_policy(bytes: &[u8]) -> Result<MomentumQualifiedDiagnosticPolicyV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedDiagnosticPolicyV1")?;
    let value = MomentumQualifiedDiagnosticPolicyV1 {
        policy_version: fields.string("policy_version")?,
        policy_name: fields.string("policy_name")?,
        frozen_values: fields.strings("frozen_values")?,
        policy_digest: fields.string("policy_digest")?,
    };
    fields.finish()?;
    validate_policy(&value)?;
    Ok(value)
}

fn encode_registration(
    value: &MomentumQualifiedSixDiagnosticRegistrationV1,
) -> Result<Vec<u8>, String> {
    validate_registration(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedSixDiagnosticRegistrationV1")
        .string("registration_version", &value.registration_version)
        .string(
            "source_replay_registration_digest",
            &value.source_replay_registration_digest,
        )
        .string(
            "source_replay_journal_digest",
            &value.source_replay_journal_digest,
        )
        .string(
            "source_public_report_digest",
            &value.source_public_report_digest,
        )
        .strings("included_participant_ids", &value.included_participant_ids)
        .strings(
            "included_partitions",
            &value
                .included_partitions
                .iter()
                .map(|partition| partition.as_str().to_string())
                .collect::<Vec<_>>(),
        )
        .string("evidence_class", "PostResultDiagnosticOnly")
        .string(
            "paired_brier_policy_digest",
            &value.paired_brier_policy_digest,
        )
        .string(
            "calendar_stability_policy_digest",
            &value.calendar_stability_policy_digest,
        )
        .string(
            "rolling_stability_policy_digest",
            &value.rolling_stability_policy_digest,
        )
        .string(
            "calibration_policy_digest",
            &value.calibration_policy_digest,
        )
        .string(
            "probability_distribution_policy_digest",
            &value.probability_distribution_policy_digest,
        )
        .string(
            "prevalence_drift_policy_digest",
            &value.prevalence_drift_policy_digest,
        )
        .string("regime_policy_digest", &value.regime_policy_digest)
        .string(
            "model_drift_policy_digest",
            &value.model_drift_policy_digest,
        )
        .string(
            "holdout_gate_policy_digest",
            &value.holdout_gate_policy_digest,
        )
        .boolean("post_result", value.post_result)
        .boolean(
            "confirmatory_claim_allowed",
            value.confirmatory_claim_allowed,
        )
        .boolean("holdout_authority", value.holdout_authority)
        .boolean("live_authority", value.live_authority)
        .boolean("trading_authority", value.trading_authority)
        .boolean("holdout_access_forbidden", value.holdout_access_forbidden)
        .boolean("new_training_forbidden", value.new_training_forbidden)
        .boolean(
            "result_selected_slicing_forbidden",
            value.result_selected_slicing_forbidden,
        )
        .boolean("live_authority_forbidden", value.live_authority_forbidden)
        .boolean(
            "governance_authority_forbidden",
            value.governance_authority_forbidden,
        )
        .boolean(
            "trading_authority_forbidden",
            value.trading_authority_forbidden,
        )
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_registration(
    bytes: &[u8],
) -> Result<MomentumQualifiedSixDiagnosticRegistrationV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedSixDiagnosticRegistrationV1")?;
    if fields.string("evidence_class")? != "PostResultDiagnosticOnly" {
        return Err("qualified-six diagnostic evidence class rejected".to_string());
    }
    let value = MomentumQualifiedSixDiagnosticRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        source_replay_registration_digest: fields.string("source_replay_registration_digest")?,
        source_replay_journal_digest: fields.string("source_replay_journal_digest")?,
        source_public_report_digest: fields.string("source_public_report_digest")?,
        included_participant_ids: fields.strings("included_participant_ids")?,
        included_partitions: fields
            .strings("included_partitions")?
            .iter()
            .map(|value| parse_partition(value))
            .collect::<Result<Vec<_>, _>>()?,
        evidence_class: MomentumQualifiedDiagnosticEvidenceClassV1::PostResultDiagnosticOnly,
        paired_brier_policy_digest: fields.string("paired_brier_policy_digest")?,
        calendar_stability_policy_digest: fields.string("calendar_stability_policy_digest")?,
        rolling_stability_policy_digest: fields.string("rolling_stability_policy_digest")?,
        calibration_policy_digest: fields.string("calibration_policy_digest")?,
        probability_distribution_policy_digest: fields
            .string("probability_distribution_policy_digest")?,
        prevalence_drift_policy_digest: fields.string("prevalence_drift_policy_digest")?,
        regime_policy_digest: fields.string("regime_policy_digest")?,
        model_drift_policy_digest: fields.string("model_drift_policy_digest")?,
        holdout_gate_policy_digest: fields.string("holdout_gate_policy_digest")?,
        post_result: fields.boolean("post_result")?,
        confirmatory_claim_allowed: fields.boolean("confirmatory_claim_allowed")?,
        holdout_authority: fields.boolean("holdout_authority")?,
        live_authority: fields.boolean("live_authority")?,
        trading_authority: fields.boolean("trading_authority")?,
        holdout_access_forbidden: fields.boolean("holdout_access_forbidden")?,
        new_training_forbidden: fields.boolean("new_training_forbidden")?,
        result_selected_slicing_forbidden: fields.boolean("result_selected_slicing_forbidden")?,
        live_authority_forbidden: fields.boolean("live_authority_forbidden")?,
        governance_authority_forbidden: fields.boolean("governance_authority_forbidden")?,
        trading_authority_forbidden: fields.boolean("trading_authority_forbidden")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_registration(&value)?;
    Ok(value)
}

fn encode_threshold(value: &MomentumQualifiedRegimeThresholdReceiptV1) -> Result<Vec<u8>, String> {
    validate_threshold(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedRegimeThresholdReceiptV1")
        .string("threshold_version", &value.threshold_version)
        .string(
            "source_diagnostic_registration_digest",
            &value.source_diagnostic_registration_digest,
        )
        .unsigned(
            "development_event_count",
            as_u64(value.development_event_count)?,
        )
        .unsigned(
            "low_volatility_upper_bits",
            value.low_volatility_upper.to_bits(),
        )
        .unsigned(
            "medium_volatility_upper_bits",
            value.medium_volatility_upper.to_bits(),
        )
        .unsigned(
            "validation_value_access_count",
            as_u64(value.validation_value_access_count)?,
        )
        .string("threshold_digest", &value.threshold_digest)
        .encode()
}

fn decode_threshold(bytes: &[u8]) -> Result<MomentumQualifiedRegimeThresholdReceiptV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedRegimeThresholdReceiptV1")?;
    let value = MomentumQualifiedRegimeThresholdReceiptV1 {
        threshold_version: fields.string("threshold_version")?,
        source_diagnostic_registration_digest: fields
            .string("source_diagnostic_registration_digest")?,
        development_event_count: as_usize(fields.unsigned("development_event_count")?)?,
        low_volatility_upper: f64::from_bits(fields.unsigned("low_volatility_upper_bits")?),
        medium_volatility_upper: f64::from_bits(fields.unsigned("medium_volatility_upper_bits")?),
        validation_value_access_count: as_usize(fields.unsigned("validation_value_access_count")?)?,
        threshold_digest: fields.string("threshold_digest")?,
    };
    fields.finish()?;
    validate_threshold(&value)?;
    Ok(value)
}

fn build_suite(name: &str, record_digests: Vec<String>) -> MomentumDiagnosticSuiteReceiptV1 {
    let mut value = MomentumDiagnosticSuiteReceiptV1 {
        suite_version: SUITE_VERSION.to_string(),
        suite_name: name.to_string(),
        record_digests,
        suite_digest: String::new(),
    };
    value.suite_digest = suite_digest(&value);
    value
}

fn validate_suite(value: &MomentumDiagnosticSuiteReceiptV1) -> Result<(), String> {
    if value.suite_version != SUITE_VERSION
        || value.suite_name.is_empty()
        || value.record_digests.is_empty()
        || value.record_digests.iter().any(String::is_empty)
        || value.suite_digest != suite_digest(value)
    {
        return Err("qualified-six diagnostic suite rejected".to_string());
    }
    Ok(())
}

fn encode_suite(value: &MomentumDiagnosticSuiteReceiptV1) -> Result<Vec<u8>, String> {
    validate_suite(value)?;
    ArtifactBuilderV4_2::new("MomentumDiagnosticSuiteReceiptV1")
        .string("suite_version", &value.suite_version)
        .string("suite_name", &value.suite_name)
        .strings("record_digests", &value.record_digests)
        .string("suite_digest", &value.suite_digest)
        .encode()
}

fn decode_suite(bytes: &[u8]) -> Result<MomentumDiagnosticSuiteReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumDiagnosticSuiteReceiptV1")?;
    let value = MomentumDiagnosticSuiteReceiptV1 {
        suite_version: fields.string("suite_version")?,
        suite_name: fields.string("suite_name")?,
        record_digests: fields.strings("record_digests")?,
        suite_digest: fields.string("suite_digest")?,
    };
    fields.finish()?;
    validate_suite(&value)?;
    Ok(value)
}

fn f64_bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn decode_f64_bits(values: Vec<u64>) -> Vec<f64> {
    values.into_iter().map(f64::from_bits).collect()
}

fn optional_f64(fields: &mut ArtifactReaderV4_2, name: &str) -> Result<Option<f64>, String> {
    fields
        .optional_string(name)?
        .map(|value| {
            value
                .parse::<u64>()
                .map(f64::from_bits)
                .map_err(|_| "qualified-six diagnostic optional float rejected".to_string())
        })
        .transpose()
}

fn parse_grain(value: &str) -> Result<MomentumDiagnosticCalendarGrainV1, String> {
    match value {
        "UtcDay" => Ok(MomentumDiagnosticCalendarGrainV1::UtcDay),
        "UtcWeek" => Ok(MomentumDiagnosticCalendarGrainV1::UtcWeek),
        "UtcMonth" => Ok(MomentumDiagnosticCalendarGrainV1::UtcMonth),
        _ => Err("qualified-six diagnostic calendar grain rejected".to_string()),
    }
}

fn parse_relation(value: &str) -> Result<MomentumDiagnosticRelationV1, String> {
    match value {
        "LowerBrier" => Ok(MomentumDiagnosticRelationV1::LowerBrier),
        "HigherBrier" => Ok(MomentumDiagnosticRelationV1::HigherBrier),
        "NumericallyEquivalent" => Ok(MomentumDiagnosticRelationV1::NumericallyEquivalent),
        "InsufficientDiagnosticSupport" => {
            Ok(MomentumDiagnosticRelationV1::InsufficientDiagnosticSupport)
        }
        "IntegrityFailure" => Ok(MomentumDiagnosticRelationV1::IntegrityFailure),
        _ => Err("qualified-six diagnostic relation rejected".to_string()),
    }
}

fn parse_volatility(value: &str) -> Result<MomentumQualifiedVolatilityRegimeV1, String> {
    match value {
        "LowVolatility" => Ok(MomentumQualifiedVolatilityRegimeV1::LowVolatility),
        "MediumVolatility" => Ok(MomentumQualifiedVolatilityRegimeV1::MediumVolatility),
        "HighVolatility" => Ok(MomentumQualifiedVolatilityRegimeV1::HighVolatility),
        _ => Err("qualified-six volatility regime rejected".to_string()),
    }
}

fn parse_trend(value: &str) -> Result<MomentumQualifiedDailyTrendRegimeV1, String> {
    match value {
        "DownTrend" => Ok(MomentumQualifiedDailyTrendRegimeV1::DownTrend),
        "Flat" => Ok(MomentumQualifiedDailyTrendRegimeV1::Flat),
        "UpTrend" => Ok(MomentumQualifiedDailyTrendRegimeV1::UpTrend),
        _ => Err("qualified-six trend regime rejected".to_string()),
    }
}

fn parse_saturation(value: &str) -> Result<MomentumQualifiedSaturationStatusV1, String> {
    match value {
        "NotSaturated" => Ok(MomentumQualifiedSaturationStatusV1::NotSaturated),
        "LowBoundarySaturation" => Ok(MomentumQualifiedSaturationStatusV1::LowBoundarySaturation),
        "HighBoundarySaturation" => Ok(MomentumQualifiedSaturationStatusV1::HighBoundarySaturation),
        "TwoSidedSaturation" => Ok(MomentumQualifiedSaturationStatusV1::TwoSidedSaturation),
        "IntegrityFailure" => Ok(MomentumQualifiedSaturationStatusV1::IntegrityFailure),
        _ => Err("qualified-six saturation status rejected".to_string()),
    }
}

fn parse_collapse(value: &str) -> Result<MomentumQualifiedProbabilityCollapseStatusV1, String> {
    match value {
        "BenchmarkExempt" => Ok(MomentumQualifiedProbabilityCollapseStatusV1::BenchmarkExempt),
        "NotCollapsed" => Ok(MomentumQualifiedProbabilityCollapseStatusV1::NotCollapsed),
        "ProbabilityCollapse" => {
            Ok(MomentumQualifiedProbabilityCollapseStatusV1::ProbabilityCollapse)
        }
        "IntegrityFailure" => Ok(MomentumQualifiedProbabilityCollapseStatusV1::IntegrityFailure),
        _ => Err("qualified-six collapse status rejected".to_string()),
    }
}

fn parse_model_drift(value: &str) -> Result<MomentumQualifiedModelDriftStatusV1, String> {
    match value {
        "StableAcrossRefits" => Ok(MomentumQualifiedModelDriftStatusV1::StableAcrossRefits),
        "ModerateDeterministicDrift" => {
            Ok(MomentumQualifiedModelDriftStatusV1::ModerateDeterministicDrift)
        }
        "HighDeterministicDrift" => Ok(MomentumQualifiedModelDriftStatusV1::HighDeterministicDrift),
        "PartitionBoundaryShift" => Ok(MomentumQualifiedModelDriftStatusV1::PartitionBoundaryShift),
        "ProbabilityCollapse" => Ok(MomentumQualifiedModelDriftStatusV1::ProbabilityCollapse),
        "IntegrityFailure" => Ok(MomentumQualifiedModelDriftStatusV1::IntegrityFailure),
        _ => Err("qualified-six model drift status rejected".to_string()),
    }
}

fn parse_partition_stability(value: &str) -> Result<MomentumQualifiedPartitionStabilityV1, String> {
    match value {
        "LowerBrierAcrossDevelopmentAndValidation" => {
            Ok(MomentumQualifiedPartitionStabilityV1::LowerBrierAcrossDevelopmentAndValidation)
        }
        "DevelopmentOnlyLowerBrier" => {
            Ok(MomentumQualifiedPartitionStabilityV1::DevelopmentOnlyLowerBrier)
        }
        "ValidationOnlyLowerBrier" => {
            Ok(MomentumQualifiedPartitionStabilityV1::ValidationOnlyLowerBrier)
        }
        "HigherBrierAcrossDevelopmentAndValidation" => {
            Ok(MomentumQualifiedPartitionStabilityV1::HigherBrierAcrossDevelopmentAndValidation)
        }
        "NumericallyEquivalentAcrossPartitions" => {
            Ok(MomentumQualifiedPartitionStabilityV1::NumericallyEquivalentAcrossPartitions)
        }
        "MixedOrInsufficientEvidence" => {
            Ok(MomentumQualifiedPartitionStabilityV1::MixedOrInsufficientEvidence)
        }
        "IntegrityFailure" => Ok(MomentumQualifiedPartitionStabilityV1::IntegrityFailure),
        _ => Err("qualified-six partition stability rejected".to_string()),
    }
}

fn parse_eligibility(value: &str) -> Result<MomentumQualifiedHoldoutEligibilityV1, String> {
    match value {
        "EligibleForFutureSealedHoldoutEvaluation" => {
            Ok(MomentumQualifiedHoldoutEligibilityV1::EligibleForFutureSealedHoldoutEvaluation)
        }
        "NotEligibleForSealedHoldout" => {
            Ok(MomentumQualifiedHoldoutEligibilityV1::NotEligibleForSealedHoldout)
        }
        _ => Err("qualified-six holdout eligibility rejected".to_string()),
    }
}

fn parse_priority(value: &str) -> Result<MomentumResearchPriorityV1, String> {
    match value {
        "PrimaryDiagnosticTarget" => Ok(MomentumResearchPriorityV1::PrimaryDiagnosticTarget),
        "SecondaryDiagnosticTarget" => Ok(MomentumResearchPriorityV1::SecondaryDiagnosticTarget),
        "DeprioritizedByCurrentEvidence" => {
            Ok(MomentumResearchPriorityV1::DeprioritizedByCurrentEvidence)
        }
        "BlockedByUnresolvedEvidence" => {
            Ok(MomentumResearchPriorityV1::BlockedByUnresolvedEvidence)
        }
        _ => Err("qualified-six research priority rejected".to_string()),
    }
}

fn encode_paired(value: &MomentumPairedBrierDiagnosticV1) -> Result<Vec<u8>, String> {
    if value.diagnostic_digest != paired_digest(value) {
        return Err("qualified-six paired diagnostic rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumPairedBrierDiagnosticV1")
        .string("participant_id", &value.participant_id)
        .string("partition", value.partition.as_str())
        .unsigned("paired_event_count", as_u64(value.paired_event_count)?)
        .unsigneds(
            "delta_bits",
            &f64_bits(&[
                value.mean_delta,
                value.median_delta,
                value.minimum_delta,
                value.maximum_delta,
            ]),
        )
        .unsigned("positive_delta_count", as_u64(value.positive_delta_count)?)
        .unsigned("negative_delta_count", as_u64(value.negative_delta_count)?)
        .unsigned(
            "equivalent_delta_count",
            as_u64(value.equivalent_delta_count)?,
        )
        .boolean("finite_value_proof", value.finite_value_proof)
        .string("diagnostic_digest", &value.diagnostic_digest)
        .encode()
}

fn decode_paired(bytes: &[u8]) -> Result<MomentumPairedBrierDiagnosticV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumPairedBrierDiagnosticV1")?;
    let bits = decode_f64_bits(fields.unsigneds("delta_bits")?);
    if bits.len() != 4 {
        return Err("qualified-six paired float cardinality rejected".to_string());
    }
    let value = MomentumPairedBrierDiagnosticV1 {
        participant_id: fields.string("participant_id")?,
        partition: parse_partition(&fields.string("partition")?)?,
        paired_event_count: as_usize(fields.unsigned("paired_event_count")?)?,
        mean_delta: bits[0],
        median_delta: bits[1],
        minimum_delta: bits[2],
        maximum_delta: bits[3],
        positive_delta_count: as_usize(fields.unsigned("positive_delta_count")?)?,
        negative_delta_count: as_usize(fields.unsigned("negative_delta_count")?)?,
        equivalent_delta_count: as_usize(fields.unsigned("equivalent_delta_count")?)?,
        finite_value_proof: fields.boolean("finite_value_proof")?,
        diagnostic_digest: fields.string("diagnostic_digest")?,
    };
    fields.finish()?;
    if value.diagnostic_digest != paired_digest(&value) {
        return Err("qualified-six paired diagnostic digest rejected".to_string());
    }
    Ok(value)
}

fn encode_calendar(value: &MomentumCalendarStabilityDiagnosticV1) -> Result<Vec<u8>, String> {
    if value.diagnostic_digest != calendar_digest(value) {
        return Err("qualified-six calendar diagnostic rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumCalendarStabilityDiagnosticV1")
        .string("participant_id", &value.participant_id)
        .string("partition", value.partition.as_str())
        .string("grain", format!("{:?}", value.grain))
        .unsigned("group_count", as_u64(value.group_count)?)
        .unsigned(
            "lower_brier_group_count",
            as_u64(value.lower_brier_group_count)?,
        )
        .unsigned(
            "higher_brier_group_count",
            as_u64(value.higher_brier_group_count)?,
        )
        .unsigned(
            "equivalent_group_count",
            as_u64(value.equivalent_group_count)?,
        )
        .unsigneds(
            "delta_bits",
            &f64_bits(&[
                value.median_group_delta,
                value.worst_group_delta,
                value.best_group_delta,
            ]),
        )
        .unsigned(
            "longest_lower_brier_streak",
            as_u64(value.longest_lower_brier_streak)?,
        )
        .unsigned(
            "longest_higher_brier_streak",
            as_u64(value.longest_higher_brier_streak)?,
        )
        .string(
            "cumulative_trajectory_digest",
            &value.cumulative_trajectory_digest,
        )
        .string("diagnostic_digest", &value.diagnostic_digest)
        .encode()
}

fn decode_calendar(bytes: &[u8]) -> Result<MomentumCalendarStabilityDiagnosticV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumCalendarStabilityDiagnosticV1")?;
    let values = decode_f64_bits(fields.unsigneds("delta_bits")?);
    if values.len() != 3 {
        return Err("qualified-six calendar float cardinality rejected".to_string());
    }
    let value = MomentumCalendarStabilityDiagnosticV1 {
        participant_id: fields.string("participant_id")?,
        partition: parse_partition(&fields.string("partition")?)?,
        grain: parse_grain(&fields.string("grain")?)?,
        group_count: as_usize(fields.unsigned("group_count")?)?,
        lower_brier_group_count: as_usize(fields.unsigned("lower_brier_group_count")?)?,
        higher_brier_group_count: as_usize(fields.unsigned("higher_brier_group_count")?)?,
        equivalent_group_count: as_usize(fields.unsigned("equivalent_group_count")?)?,
        median_group_delta: values[0],
        worst_group_delta: values[1],
        best_group_delta: values[2],
        longest_lower_brier_streak: as_usize(fields.unsigned("longest_lower_brier_streak")?)?,
        longest_higher_brier_streak: as_usize(fields.unsigned("longest_higher_brier_streak")?)?,
        cumulative_trajectory_digest: fields.string("cumulative_trajectory_digest")?,
        diagnostic_digest: fields.string("diagnostic_digest")?,
    };
    fields.finish()?;
    if value.diagnostic_digest != calendar_digest(&value) {
        return Err("qualified-six calendar diagnostic digest rejected".to_string());
    }
    Ok(value)
}

fn encode_rolling(value: &MomentumRollingStabilityDiagnosticV1) -> Result<Vec<u8>, String> {
    if value.diagnostic_digest != rolling_digest(value) {
        return Err("qualified-six rolling diagnostic rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumRollingStabilityDiagnosticV1")
        .string("participant_id", &value.participant_id)
        .string("partition", value.partition.as_str())
        .unsigned("window_event_count", as_u64(value.window_event_count)?)
        .string("variant", &value.variant)
        .unsigned(
            "eligible_window_count",
            as_u64(value.eligible_window_count)?,
        )
        .unsigned(
            "lower_brier_window_count",
            as_u64(value.lower_brier_window_count)?,
        )
        .unsigned(
            "higher_brier_window_count",
            as_u64(value.higher_brier_window_count)?,
        )
        .unsigned(
            "equivalent_window_count",
            as_u64(value.equivalent_window_count)?,
        )
        .unsigneds(
            "delta_bits",
            &f64_bits(&[
                value.minimum_window_delta,
                value.maximum_window_delta,
                value.median_window_delta,
            ]),
        )
        .unsigned("sign_change_count", as_u64(value.sign_change_count)?)
        .unsigned("minimum_timestamp_span_ms", value.minimum_timestamp_span_ms)
        .unsigned("maximum_timestamp_span_ms", value.maximum_timestamp_span_ms)
        .unsigned("median_timestamp_span_ms", value.median_timestamp_span_ms)
        .string("diagnostic_digest", &value.diagnostic_digest)
        .encode()
}

fn decode_rolling(bytes: &[u8]) -> Result<MomentumRollingStabilityDiagnosticV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumRollingStabilityDiagnosticV1")?;
    let values = decode_f64_bits(fields.unsigneds("delta_bits")?);
    if values.len() != 3 {
        return Err("qualified-six rolling float cardinality rejected".to_string());
    }
    let value = MomentumRollingStabilityDiagnosticV1 {
        participant_id: fields.string("participant_id")?,
        partition: parse_partition(&fields.string("partition")?)?,
        window_event_count: as_usize(fields.unsigned("window_event_count")?)?,
        variant: fields.string("variant")?,
        eligible_window_count: as_usize(fields.unsigned("eligible_window_count")?)?,
        lower_brier_window_count: as_usize(fields.unsigned("lower_brier_window_count")?)?,
        higher_brier_window_count: as_usize(fields.unsigned("higher_brier_window_count")?)?,
        equivalent_window_count: as_usize(fields.unsigned("equivalent_window_count")?)?,
        minimum_window_delta: values[0],
        maximum_window_delta: values[1],
        median_window_delta: values[2],
        sign_change_count: as_usize(fields.unsigned("sign_change_count")?)?,
        minimum_timestamp_span_ms: fields.unsigned("minimum_timestamp_span_ms")?,
        maximum_timestamp_span_ms: fields.unsigned("maximum_timestamp_span_ms")?,
        median_timestamp_span_ms: fields.unsigned("median_timestamp_span_ms")?,
        diagnostic_digest: fields.string("diagnostic_digest")?,
    };
    fields.finish()?;
    if value.diagnostic_digest != rolling_digest(&value) {
        return Err("qualified-six rolling diagnostic digest rejected".to_string());
    }
    Ok(value)
}

fn encode_prevalence(value: &MomentumPrevalenceDriftDiagnosticV1) -> Result<Vec<u8>, String> {
    if value.diagnostic_digest != prevalence_digest(value) {
        return Err("qualified-six prevalence diagnostic rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumPrevalenceDriftDiagnosticV1")
        .string("partition", value.partition.as_str())
        .string("grain", format!("{:?}", value.grain))
        .unsigned("scorable_count", as_u64(value.scorable_count)?)
        .unsigned("positive_count", as_u64(value.positive_count)?)
        .unsigned("negative_count", as_u64(value.negative_count)?)
        .unsigned("group_count", as_u64(value.group_count)?)
        .unsigneds(
            "value_bits",
            &f64_bits(&[
                value.partition_positive_prevalence,
                value.minimum_group_prevalence,
                value.maximum_group_prevalence,
                value.minimum_deviation,
                value.maximum_deviation,
            ]),
        )
        .string(
            "prevalence_trajectory_digest",
            &value.prevalence_trajectory_digest,
        )
        .string("diagnostic_digest", &value.diagnostic_digest)
        .encode()
}

fn decode_prevalence(bytes: &[u8]) -> Result<MomentumPrevalenceDriftDiagnosticV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumPrevalenceDriftDiagnosticV1")?;
    let values = decode_f64_bits(fields.unsigneds("value_bits")?);
    if values.len() != 5 {
        return Err("qualified-six prevalence float cardinality rejected".to_string());
    }
    let value = MomentumPrevalenceDriftDiagnosticV1 {
        partition: parse_partition(&fields.string("partition")?)?,
        grain: parse_grain(&fields.string("grain")?)?,
        scorable_count: as_usize(fields.unsigned("scorable_count")?)?,
        positive_count: as_usize(fields.unsigned("positive_count")?)?,
        negative_count: as_usize(fields.unsigned("negative_count")?)?,
        partition_positive_prevalence: values[0],
        group_count: as_usize(fields.unsigned("group_count")?)?,
        minimum_group_prevalence: values[1],
        maximum_group_prevalence: values[2],
        minimum_deviation: values[3],
        maximum_deviation: values[4],
        prevalence_trajectory_digest: fields.string("prevalence_trajectory_digest")?,
        diagnostic_digest: fields.string("diagnostic_digest")?,
    };
    fields.finish()?;
    if value.diagnostic_digest != prevalence_digest(&value) {
        return Err("qualified-six prevalence diagnostic digest rejected".to_string());
    }
    Ok(value)
}

fn encode_calibration_bin(value: &MomentumCalibrationBinDiagnosticV1) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumCalibrationBinDiagnosticV1")
        .unsigned("lower_bound_bits", value.lower_bound.to_bits())
        .unsigned("upper_bound_bits", value.upper_bound.to_bits())
        .boolean("upper_inclusive", value.upper_inclusive)
        .unsigned("support", as_u64(value.support)?)
        .optional_string(
            "mean_predicted_probability_bits",
            &value
                .mean_predicted_probability
                .map(|item| item.to_bits().to_string()),
        )
        .optional_string(
            "observed_positive_frequency_bits",
            &value
                .observed_positive_frequency
                .map(|item| item.to_bits().to_string()),
        )
        .optional_string(
            "absolute_calibration_gap_bits",
            &value
                .absolute_calibration_gap
                .map(|item| item.to_bits().to_string()),
        )
        .encode()
}

fn decode_calibration_bin(bytes: &[u8]) -> Result<MomentumCalibrationBinDiagnosticV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumCalibrationBinDiagnosticV1")?;
    let value = MomentumCalibrationBinDiagnosticV1 {
        lower_bound: f64::from_bits(fields.unsigned("lower_bound_bits")?),
        upper_bound: f64::from_bits(fields.unsigned("upper_bound_bits")?),
        upper_inclusive: fields.boolean("upper_inclusive")?,
        support: as_usize(fields.unsigned("support")?)?,
        mean_predicted_probability: optional_f64(&mut fields, "mean_predicted_probability_bits")?,
        observed_positive_frequency: optional_f64(&mut fields, "observed_positive_frequency_bits")?,
        absolute_calibration_gap: optional_f64(&mut fields, "absolute_calibration_gap_bits")?,
    };
    fields.finish()?;
    Ok(value)
}

fn encode_calibration(value: &MomentumCalibrationDiagnosticV1) -> Result<Vec<u8>, String> {
    if value.diagnostic_digest != calibration_digest(value) {
        return Err("qualified-six calibration diagnostic rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumCalibrationDiagnosticV1")
        .string("participant_id", &value.participant_id)
        .string("partition", value.partition.as_str())
        .messages(
            "bins",
            value
                .bins
                .iter()
                .map(encode_calibration_bin)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .unsigned(
            "weighted_aggregate_calibration_gap_bits",
            value.weighted_aggregate_calibration_gap.to_bits(),
        )
        .unsigned("empty_bin_count", as_u64(value.empty_bin_count)?)
        .boolean("finite_value_proof", value.finite_value_proof)
        .string("diagnostic_digest", &value.diagnostic_digest)
        .encode()
}

fn decode_calibration(bytes: &[u8]) -> Result<MomentumCalibrationDiagnosticV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumCalibrationDiagnosticV1")?;
    let value = MomentumCalibrationDiagnosticV1 {
        participant_id: fields.string("participant_id")?,
        partition: parse_partition(&fields.string("partition")?)?,
        bins: fields
            .messages("bins")?
            .iter()
            .map(|bytes| decode_calibration_bin(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        weighted_aggregate_calibration_gap: f64::from_bits(
            fields.unsigned("weighted_aggregate_calibration_gap_bits")?,
        ),
        empty_bin_count: as_usize(fields.unsigned("empty_bin_count")?)?,
        finite_value_proof: fields.boolean("finite_value_proof")?,
        diagnostic_digest: fields.string("diagnostic_digest")?,
    };
    fields.finish()?;
    if value.bins.len() != 10 || value.diagnostic_digest != calibration_digest(&value) {
        return Err("qualified-six calibration diagnostic digest rejected".to_string());
    }
    Ok(value)
}

fn encode_probability(
    value: &MomentumProbabilityDistributionDiagnosticV1,
) -> Result<Vec<u8>, String> {
    if value.diagnostic_digest != probability_digest(value) {
        return Err("qualified-six probability diagnostic rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumProbabilityDistributionDiagnosticV1")
        .string("participant_id", &value.participant_id)
        .string("partition", value.partition.as_str())
        .unsigneds(
            "summary_bits",
            &f64_bits(&[
                value.minimum,
                value.percentile_01,
                value.percentile_05,
                value.percentile_25,
                value.median,
                value.percentile_75,
                value.percentile_95,
                value.percentile_99,
                value.maximum,
                value.mean,
                value.standard_deviation,
            ]),
        )
        .unsigned(
            "exact_constant_value_count",
            as_u64(value.exact_constant_value_count)?,
        )
        .unsigned("near_half_count", as_u64(value.near_half_count)?)
        .unsigned("extreme_low_count", as_u64(value.extreme_low_count)?)
        .unsigned("extreme_high_count", as_u64(value.extreme_high_count)?)
        .unsigned("nonfinite_count", as_u64(value.nonfinite_count)?)
        .string(
            "saturation_status",
            format!("{:?}", value.saturation_status),
        )
        .string("collapse_status", format!("{:?}", value.collapse_status))
        .string("diagnostic_digest", &value.diagnostic_digest)
        .encode()
}

fn decode_probability(bytes: &[u8]) -> Result<MomentumProbabilityDistributionDiagnosticV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumProbabilityDistributionDiagnosticV1")?;
    let values = decode_f64_bits(fields.unsigneds("summary_bits")?);
    if values.len() != 11 {
        return Err("qualified-six probability float cardinality rejected".to_string());
    }
    let value = MomentumProbabilityDistributionDiagnosticV1 {
        participant_id: fields.string("participant_id")?,
        partition: parse_partition(&fields.string("partition")?)?,
        minimum: values[0],
        percentile_01: values[1],
        percentile_05: values[2],
        percentile_25: values[3],
        median: values[4],
        percentile_75: values[5],
        percentile_95: values[6],
        percentile_99: values[7],
        maximum: values[8],
        mean: values[9],
        standard_deviation: values[10],
        exact_constant_value_count: as_usize(fields.unsigned("exact_constant_value_count")?)?,
        near_half_count: as_usize(fields.unsigned("near_half_count")?)?,
        extreme_low_count: as_usize(fields.unsigned("extreme_low_count")?)?,
        extreme_high_count: as_usize(fields.unsigned("extreme_high_count")?)?,
        nonfinite_count: as_usize(fields.unsigned("nonfinite_count")?)?,
        saturation_status: parse_saturation(&fields.string("saturation_status")?)?,
        collapse_status: parse_collapse(&fields.string("collapse_status")?)?,
        diagnostic_digest: fields.string("diagnostic_digest")?,
    };
    fields.finish()?;
    if value.diagnostic_digest != probability_digest(&value) {
        return Err("qualified-six probability diagnostic digest rejected".to_string());
    }
    Ok(value)
}

fn encode_regime(value: &MomentumRegimeMetricDiagnosticV1) -> Result<Vec<u8>, String> {
    if value.diagnostic_digest != regime_digest(value) {
        return Err("qualified-six regime diagnostic rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumRegimeMetricDiagnosticV1")
        .string("participant_id", &value.participant_id)
        .string("partition", value.partition.as_str())
        .string("regime_dimension", &value.regime_dimension)
        .optional_string(
            "volatility_regime",
            &value.volatility_regime.map(|item| format!("{item:?}")),
        )
        .optional_string(
            "daily_trend_regime",
            &value.daily_trend_regime.map(|item| format!("{item:?}")),
        )
        .unsigned("event_count", as_u64(value.event_count)?)
        .unsigned("scorable_count", as_u64(value.scorable_count)?)
        .unsigned("neutral_count", as_u64(value.neutral_count)?)
        .optional_string(
            "mean_brier_bits",
            &value.mean_brier.map(|item| item.to_bits().to_string()),
        )
        .optional_string(
            "correctness_bits",
            &value.correctness.map(|item| item.to_bits().to_string()),
        )
        .optional_string(
            "paired_delta_bits",
            &value
                .paired_brier_delta_versus_q0
                .map(|item| item.to_bits().to_string()),
        )
        .string("relation", format!("{:?}", value.relation))
        .boolean("finite_value_proof", value.finite_value_proof)
        .string("diagnostic_digest", &value.diagnostic_digest)
        .encode()
}

fn decode_regime(bytes: &[u8]) -> Result<MomentumRegimeMetricDiagnosticV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumRegimeMetricDiagnosticV1")?;
    let value = MomentumRegimeMetricDiagnosticV1 {
        participant_id: fields.string("participant_id")?,
        partition: parse_partition(&fields.string("partition")?)?,
        regime_dimension: fields.string("regime_dimension")?,
        volatility_regime: fields
            .optional_string("volatility_regime")?
            .map(|value| parse_volatility(&value))
            .transpose()?,
        daily_trend_regime: fields
            .optional_string("daily_trend_regime")?
            .map(|value| parse_trend(&value))
            .transpose()?,
        event_count: as_usize(fields.unsigned("event_count")?)?,
        scorable_count: as_usize(fields.unsigned("scorable_count")?)?,
        neutral_count: as_usize(fields.unsigned("neutral_count")?)?,
        mean_brier: optional_f64(&mut fields, "mean_brier_bits")?,
        correctness: optional_f64(&mut fields, "correctness_bits")?,
        paired_brier_delta_versus_q0: optional_f64(&mut fields, "paired_delta_bits")?,
        relation: parse_relation(&fields.string("relation")?)?,
        finite_value_proof: fields.boolean("finite_value_proof")?,
        diagnostic_digest: fields.string("diagnostic_digest")?,
    };
    fields.finish()?;
    if value.diagnostic_digest != regime_digest(&value) {
        return Err("qualified-six regime diagnostic digest rejected".to_string());
    }
    Ok(value)
}

fn encode_model_drift(value: &MomentumModelDriftDiagnosticV1) -> Result<Vec<u8>, String> {
    if value.diagnostic_digest != model_drift_digest(value) {
        return Err("qualified-six model drift diagnostic rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumModelDriftDiagnosticV1")
        .string("participant_id", &value.participant_id)
        .string("partition", value.partition.as_str())
        .unsigned("refit_count", as_u64(value.refit_count)?)
        .string(
            "parameter_digest_trajectory",
            &value.parameter_digest_trajectory,
        )
        .string(
            "normalizer_digest_trajectory",
            &value.normalizer_digest_trajectory,
        )
        .boolean("parameter_finite", value.parameter_finite)
        .boolean("normalizer_finite", value.normalizer_finite)
        .boolean("training_loss_finite", value.training_loss_finite)
        .unsigned(
            "minimum_training_example_count",
            as_u64(value.minimum_training_example_count)?,
        )
        .unsigned(
            "maximum_training_example_count",
            as_u64(value.maximum_training_example_count)?,
        )
        .unsigneds(
            "summary_bits",
            &f64_bits(&[
                value.minimum_training_prevalence,
                value.maximum_training_prevalence,
                value.median_parameter_norm_change,
                value.maximum_parameter_norm_change,
                value.median_normalizer_shift,
                value.maximum_normalizer_shift,
                value.median_daily_prediction_dispersion,
            ]),
        )
        .boolean("partition_boundary_shift", value.partition_boundary_shift)
        .string("status", format!("{:?}", value.status))
        .string("diagnostic_digest", &value.diagnostic_digest)
        .encode()
}

fn decode_model_drift(bytes: &[u8]) -> Result<MomentumModelDriftDiagnosticV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumModelDriftDiagnosticV1")?;
    let values = decode_f64_bits(fields.unsigneds("summary_bits")?);
    if values.len() != 7 {
        return Err("qualified-six model drift float cardinality rejected".to_string());
    }
    let value = MomentumModelDriftDiagnosticV1 {
        participant_id: fields.string("participant_id")?,
        partition: parse_partition(&fields.string("partition")?)?,
        refit_count: as_usize(fields.unsigned("refit_count")?)?,
        parameter_digest_trajectory: fields.string("parameter_digest_trajectory")?,
        normalizer_digest_trajectory: fields.string("normalizer_digest_trajectory")?,
        parameter_finite: fields.boolean("parameter_finite")?,
        normalizer_finite: fields.boolean("normalizer_finite")?,
        training_loss_finite: fields.boolean("training_loss_finite")?,
        minimum_training_example_count: as_usize(
            fields.unsigned("minimum_training_example_count")?,
        )?,
        maximum_training_example_count: as_usize(
            fields.unsigned("maximum_training_example_count")?,
        )?,
        minimum_training_prevalence: values[0],
        maximum_training_prevalence: values[1],
        median_parameter_norm_change: values[2],
        maximum_parameter_norm_change: values[3],
        median_normalizer_shift: values[4],
        maximum_normalizer_shift: values[5],
        median_daily_prediction_dispersion: values[6],
        partition_boundary_shift: fields.boolean("partition_boundary_shift")?,
        status: parse_model_drift(&fields.string("status")?)?,
        diagnostic_digest: fields.string("diagnostic_digest")?,
    };
    fields.finish()?;
    if value.diagnostic_digest != model_drift_digest(&value) {
        return Err("qualified-six model drift diagnostic digest rejected".to_string());
    }
    Ok(value)
}

fn encode_partition_stability(
    value: &MomentumPartitionStabilityReceiptV1,
) -> Result<Vec<u8>, String> {
    if value.receipt_digest != partition_stability_digest(value) {
        return Err("qualified-six partition stability receipt rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumPartitionStabilityReceiptV1")
        .string("participant_id", &value.participant_id)
        .string("classification", format!("{:?}", value.classification))
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_partition_stability(bytes: &[u8]) -> Result<MomentumPartitionStabilityReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumPartitionStabilityReceiptV1")?;
    let value = MomentumPartitionStabilityReceiptV1 {
        participant_id: fields.string("participant_id")?,
        classification: parse_partition_stability(&fields.string("classification")?)?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    if value.receipt_digest != partition_stability_digest(&value) {
        return Err("qualified-six partition stability digest rejected".to_string());
    }
    Ok(value)
}

fn encode_eligibility(value: &MomentumHoldoutEligibilityReceiptV1) -> Result<Vec<u8>, String> {
    if value.receipt_digest != eligibility_digest(value) {
        return Err("qualified-six eligibility receipt rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumHoldoutEligibilityReceiptV1")
        .string("participant_id", &value.participant_id)
        .boolean("lower_brier_development", value.lower_brier_development)
        .boolean("lower_brier_validation", value.lower_brier_validation)
        .boolean("sufficient_paired_support", value.sufficient_paired_support)
        .boolean(
            "finite_predictions_and_metrics",
            value.finite_predictions_and_metrics,
        )
        .boolean(
            "probability_collapse_absent",
            value.probability_collapse_absent,
        )
        .boolean(
            "chronology_and_leakage_passed",
            value.chronology_and_leakage_passed,
        )
        .boolean("integrity_passed", value.integrity_passed)
        .boolean("source_replay_unmutated", value.source_replay_unmutated)
        .string("eligibility", format!("{:?}", value.eligibility))
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_eligibility(bytes: &[u8]) -> Result<MomentumHoldoutEligibilityReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumHoldoutEligibilityReceiptV1")?;
    let value = MomentumHoldoutEligibilityReceiptV1 {
        participant_id: fields.string("participant_id")?,
        lower_brier_development: fields.boolean("lower_brier_development")?,
        lower_brier_validation: fields.boolean("lower_brier_validation")?,
        sufficient_paired_support: fields.boolean("sufficient_paired_support")?,
        finite_predictions_and_metrics: fields.boolean("finite_predictions_and_metrics")?,
        probability_collapse_absent: fields.boolean("probability_collapse_absent")?,
        chronology_and_leakage_passed: fields.boolean("chronology_and_leakage_passed")?,
        integrity_passed: fields.boolean("integrity_passed")?,
        source_replay_unmutated: fields.boolean("source_replay_unmutated")?,
        eligibility: parse_eligibility(&fields.string("eligibility")?)?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    if value.receipt_digest != eligibility_digest(&value) {
        return Err("qualified-six eligibility receipt digest rejected".to_string());
    }
    Ok(value)
}

fn encode_requirements(
    value: &MomentumQualifiedChallengerRequirementsV1,
) -> Result<Vec<u8>, String> {
    validate_requirements(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedChallengerRequirementsV1")
        .string("requirements_version", &value.requirements_version)
        .string(
            "source_diagnostic_registration_digest",
            &value.source_diagnostic_registration_digest,
        )
        .string(
            "source_diagnostic_report_digest",
            &value.source_diagnostic_report_digest,
        )
        .boolean(
            "constant_benchmark_mandatory",
            value.constant_benchmark_mandatory,
        )
        .boolean(
            "full_eight_claim_forbidden",
            value.full_eight_claim_forbidden,
        )
        .boolean("month_year_use_forbidden", value.month_year_use_forbidden)
        .boolean(
            "complexity_escalation_allowed",
            value.complexity_escalation_allowed,
        )
        .boolean(
            "interaction_expansion_allowed",
            value.interaction_expansion_allowed,
        )
        .boolean("sequence_model_allowed", value.sequence_model_allowed)
        .string(
            "micro_block_research_priority",
            format!("{:?}", value.micro_block_research_priority),
        )
        .string(
            "qualified_macro_addition_priority",
            format!("{:?}", value.qualified_macro_addition_priority),
        )
        .boolean("label_forensics_required", value.label_forensics_required)
        .boolean(
            "calibration_repair_required",
            value.calibration_repair_required,
        )
        .boolean("regime_stability_required", value.regime_stability_required)
        .boolean(
            "two_partition_improvement_required",
            value.two_partition_improvement_required,
        )
        .boolean(
            "new_model_execution_authorized",
            value.new_model_execution_authorized,
        )
        .boolean(
            "holdout_execution_authorized",
            value.holdout_execution_authorized,
        )
        .string("requirements_digest", &value.requirements_digest)
        .encode()
}

fn decode_requirements(bytes: &[u8]) -> Result<MomentumQualifiedChallengerRequirementsV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedChallengerRequirementsV1")?;
    let value = MomentumQualifiedChallengerRequirementsV1 {
        requirements_version: fields.string("requirements_version")?,
        source_diagnostic_registration_digest: fields
            .string("source_diagnostic_registration_digest")?,
        source_diagnostic_report_digest: fields.string("source_diagnostic_report_digest")?,
        constant_benchmark_mandatory: fields.boolean("constant_benchmark_mandatory")?,
        full_eight_claim_forbidden: fields.boolean("full_eight_claim_forbidden")?,
        month_year_use_forbidden: fields.boolean("month_year_use_forbidden")?,
        complexity_escalation_allowed: fields.boolean("complexity_escalation_allowed")?,
        interaction_expansion_allowed: fields.boolean("interaction_expansion_allowed")?,
        sequence_model_allowed: fields.boolean("sequence_model_allowed")?,
        micro_block_research_priority: parse_priority(
            &fields.string("micro_block_research_priority")?,
        )?,
        qualified_macro_addition_priority: parse_priority(
            &fields.string("qualified_macro_addition_priority")?,
        )?,
        label_forensics_required: fields.boolean("label_forensics_required")?,
        calibration_repair_required: fields.boolean("calibration_repair_required")?,
        regime_stability_required: fields.boolean("regime_stability_required")?,
        two_partition_improvement_required: fields.boolean("two_partition_improvement_required")?,
        new_model_execution_authorized: fields.boolean("new_model_execution_authorized")?,
        holdout_execution_authorized: fields.boolean("holdout_execution_authorized")?,
        requirements_digest: fields.string("requirements_digest")?,
    };
    fields.finish()?;
    validate_requirements(&value)?;
    Ok(value)
}

fn validate_journal(value: &MomentumQualifiedDiagnosticJournalV1) -> Result<(), String> {
    if value.journal_version != JOURNAL_VERSION
        || [
            &value.registration_digest,
            &value.source_replay_journal_digest,
            &value.regime_threshold_digest,
            &value.paired_suite_digest,
            &value.calendar_suite_digest,
            &value.rolling_suite_digest,
            &value.calibration_suite_digest,
            &value.probability_suite_digest,
            &value.prevalence_suite_digest,
            &value.regime_suite_digest,
            &value.model_drift_suite_digest,
            &value.partition_stability_suite_digest,
            &value.holdout_eligibility_suite_digest,
            &value.challenger_requirements_digest,
        ]
        .iter()
        .any(|digest| digest.is_empty())
        || value.holdout_label_reads != 0
        || value.holdout_prediction_reads != 0
        || value.holdout_metric_reads != 0
        || value.live_outcome_requests != 0
        || value.live_outcome_openings != 0
        || !value.deterministic
        || value.journal_digest != journal_digest(value)
    {
        return Err("qualified-six diagnostic journal rejected".to_string());
    }
    Ok(())
}

fn encode_journal(value: &MomentumQualifiedDiagnosticJournalV1) -> Result<Vec<u8>, String> {
    validate_journal(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedDiagnosticJournalV1")
        .string("journal_version", &value.journal_version)
        .string("registration_digest", &value.registration_digest)
        .string(
            "source_replay_journal_digest",
            &value.source_replay_journal_digest,
        )
        .string("regime_threshold_digest", &value.regime_threshold_digest)
        .string("paired_suite_digest", &value.paired_suite_digest)
        .string("calendar_suite_digest", &value.calendar_suite_digest)
        .string("rolling_suite_digest", &value.rolling_suite_digest)
        .string("calibration_suite_digest", &value.calibration_suite_digest)
        .string("probability_suite_digest", &value.probability_suite_digest)
        .string("prevalence_suite_digest", &value.prevalence_suite_digest)
        .string("regime_suite_digest", &value.regime_suite_digest)
        .string("model_drift_suite_digest", &value.model_drift_suite_digest)
        .string(
            "partition_stability_suite_digest",
            &value.partition_stability_suite_digest,
        )
        .string(
            "holdout_eligibility_suite_digest",
            &value.holdout_eligibility_suite_digest,
        )
        .string(
            "challenger_requirements_digest",
            &value.challenger_requirements_digest,
        )
        .unsigned("holdout_label_reads", as_u64(value.holdout_label_reads)?)
        .unsigned(
            "holdout_prediction_reads",
            as_u64(value.holdout_prediction_reads)?,
        )
        .unsigned("holdout_metric_reads", as_u64(value.holdout_metric_reads)?)
        .unsigned(
            "live_outcome_requests",
            as_u64(value.live_outcome_requests)?,
        )
        .unsigned(
            "live_outcome_openings",
            as_u64(value.live_outcome_openings)?,
        )
        .boolean("deterministic", value.deterministic)
        .string("journal_digest", &value.journal_digest)
        .encode()
}

fn decode_journal(bytes: &[u8]) -> Result<MomentumQualifiedDiagnosticJournalV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedDiagnosticJournalV1")?;
    let value = MomentumQualifiedDiagnosticJournalV1 {
        journal_version: fields.string("journal_version")?,
        registration_digest: fields.string("registration_digest")?,
        source_replay_journal_digest: fields.string("source_replay_journal_digest")?,
        regime_threshold_digest: fields.string("regime_threshold_digest")?,
        paired_suite_digest: fields.string("paired_suite_digest")?,
        calendar_suite_digest: fields.string("calendar_suite_digest")?,
        rolling_suite_digest: fields.string("rolling_suite_digest")?,
        calibration_suite_digest: fields.string("calibration_suite_digest")?,
        probability_suite_digest: fields.string("probability_suite_digest")?,
        prevalence_suite_digest: fields.string("prevalence_suite_digest")?,
        regime_suite_digest: fields.string("regime_suite_digest")?,
        model_drift_suite_digest: fields.string("model_drift_suite_digest")?,
        partition_stability_suite_digest: fields.string("partition_stability_suite_digest")?,
        holdout_eligibility_suite_digest: fields.string("holdout_eligibility_suite_digest")?,
        challenger_requirements_digest: fields.string("challenger_requirements_digest")?,
        holdout_label_reads: as_usize(fields.unsigned("holdout_label_reads")?)?,
        holdout_prediction_reads: as_usize(fields.unsigned("holdout_prediction_reads")?)?,
        holdout_metric_reads: as_usize(fields.unsigned("holdout_metric_reads")?)?,
        live_outcome_requests: as_usize(fields.unsigned("live_outcome_requests")?)?,
        live_outcome_openings: as_usize(fields.unsigned("live_outcome_openings")?)?,
        deterministic: fields.boolean("deterministic")?,
        journal_digest: fields.string("journal_digest")?,
    };
    fields.finish()?;
    validate_journal(&value)?;
    Ok(value)
}

fn parse_report_status(value: &str) -> Result<MomentumQualifiedDiagnosticStatusV1, String> {
    match value {
        "Unregistered" => Ok(MomentumQualifiedDiagnosticStatusV1::Unregistered),
        "Registered" => Ok(MomentumQualifiedDiagnosticStatusV1::Registered),
        "Complete" => Ok(MomentumQualifiedDiagnosticStatusV1::Complete),
        "IntegrityFailure" => Ok(MomentumQualifiedDiagnosticStatusV1::IntegrityFailure),
        _ => Err("qualified-six diagnostic report status rejected".to_string()),
    }
}

fn validate_report(value: &MomentumQualifiedSixDiagnosticReportV1) -> Result<(), String> {
    let zero_counters = [
        value.holdout_label_reads,
        value.holdout_prediction_reads,
        value.holdout_metric_reads,
        value.holdout_execution_modes,
        value.live_outcome_requests,
        value.live_outcome_openings,
        value.live_participant_changes,
        value.winner_selections,
        value.ranking_creations,
        value.reward_applications,
        value.penalty_applications,
        value.chair_decisions,
        value.trading_actions,
        value.network_request_attempts,
        value.month_view_load_count,
        value.year_view_load_count,
        value.model_refit_count,
        value.prediction_computation_count,
        value.evaluation_computation_count,
    ]
    .into_iter()
    .all(|count| count == 0);
    let registered = value.status != MomentumQualifiedDiagnosticStatusV1::Unregistered;
    let complete = value.status == MomentumQualifiedDiagnosticStatusV1::Complete;
    if value.report_version != REPORT_VERSION
        || value.run_mode.is_empty()
        || value.evidence_class
            != MomentumQualifiedDiagnosticEvidenceClassV1::PostResultDiagnosticOnly
        || !value.post_result
        || value.confirmatory_claim_allowed
        || value.holdout_authority
        || value.live_authority
        || value.trading_authority
        || !zero_counters
        || !value.full_eight_a3_blocked
        || !value.protected_artifacts_unchanged
        || !value.active_roster_unchanged
        || value.labels != PUBLIC_LABELS
        || registered
            != (value.source_replay_registration_digest.is_some()
                && value.source_replay_journal_digest.is_some()
                && value.source_public_report_digest.is_some()
                && value.diagnostic_registration_digest.is_some())
        || (registered
            && (value.participant_ids != participant_ids()
                || value.included_partitions != included_partitions()))
        || (complete
            && (value.paired_brier_diagnostics.len() != 8
                || value.calendar_stability_diagnostics.len() != 24
                || value.rolling_stability_diagnostics.len() != 32
                || value.calibration_diagnostics.len() != 10
                || value.probability_distribution_diagnostics.len() != 10
                || value.prevalence_drift_diagnostics.len() != 6
                || value.regime_diagnostics.len() != 150
                || value.model_drift_diagnostics.len() != 8
                || value.partition_stability_receipts.len() != 4
                || value.holdout_eligibility_receipts.len() != 4
                || value.volatility_threshold_low_upper.is_none()
                || value.volatility_threshold_medium_upper.is_none()
                || value.challenger_requirements.is_none()
                || value
                    .diagnostic_journal_digest
                    .as_deref()
                    .is_none_or(str::is_empty)))
        || (!complete
            && (!value.paired_brier_diagnostics.is_empty()
                || !value.calendar_stability_diagnostics.is_empty()
                || !value.rolling_stability_diagnostics.is_empty()
                || !value.calibration_diagnostics.is_empty()
                || !value.probability_distribution_diagnostics.is_empty()
                || !value.prevalence_drift_diagnostics.is_empty()
                || !value.regime_diagnostics.is_empty()
                || !value.model_drift_diagnostics.is_empty()
                || !value.partition_stability_receipts.is_empty()
                || !value.holdout_eligibility_receipts.is_empty()
                || value.challenger_requirements.is_some()
                || value.diagnostic_journal_digest.is_some()))
        || value
            .challenger_requirements
            .as_ref()
            .is_some_and(|requirements| {
                validate_requirements(requirements).is_err()
                    || requirements.source_diagnostic_report_digest != value.report_digest
            })
        || value.report_digest != report_digest(value)
    {
        return Err("qualified-six diagnostic public report rejected".to_string());
    }
    Ok(())
}

fn encode_report(value: &MomentumQualifiedSixDiagnosticReportV1) -> Result<Vec<u8>, String> {
    validate_report(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedSixDiagnosticReportV1")
        .string("report_version", &value.report_version)
        .string("run_mode", &value.run_mode)
        .string("status", format!("{:?}", value.status))
        .string("evidence_class", "PostResultDiagnosticOnly")
        .boolean("post_result", value.post_result)
        .boolean(
            "confirmatory_claim_allowed",
            value.confirmatory_claim_allowed,
        )
        .boolean("holdout_authority", value.holdout_authority)
        .boolean("live_authority", value.live_authority)
        .boolean("trading_authority", value.trading_authority)
        .optional_string(
            "source_replay_registration_digest",
            &value.source_replay_registration_digest,
        )
        .optional_string(
            "source_replay_journal_digest",
            &value.source_replay_journal_digest,
        )
        .optional_string(
            "source_public_report_digest",
            &value.source_public_report_digest,
        )
        .optional_string(
            "diagnostic_registration_digest",
            &value.diagnostic_registration_digest,
        )
        .strings("participant_ids", &value.participant_ids)
        .strings(
            "included_partitions",
            &value
                .included_partitions
                .iter()
                .map(|partition| partition.as_str().to_string())
                .collect::<Vec<_>>(),
        )
        .messages(
            "paired_brier_diagnostics",
            value
                .paired_brier_diagnostics
                .iter()
                .map(encode_paired)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "calendar_stability_diagnostics",
            value
                .calendar_stability_diagnostics
                .iter()
                .map(encode_calendar)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "rolling_stability_diagnostics",
            value
                .rolling_stability_diagnostics
                .iter()
                .map(encode_rolling)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "calibration_diagnostics",
            value
                .calibration_diagnostics
                .iter()
                .map(encode_calibration)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "probability_distribution_diagnostics",
            value
                .probability_distribution_diagnostics
                .iter()
                .map(encode_probability)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "prevalence_drift_diagnostics",
            value
                .prevalence_drift_diagnostics
                .iter()
                .map(encode_prevalence)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .optional_string(
            "volatility_threshold_low_upper_bits",
            &value
                .volatility_threshold_low_upper
                .map(|item| item.to_bits().to_string()),
        )
        .optional_string(
            "volatility_threshold_medium_upper_bits",
            &value
                .volatility_threshold_medium_upper
                .map(|item| item.to_bits().to_string()),
        )
        .messages(
            "regime_diagnostics",
            value
                .regime_diagnostics
                .iter()
                .map(encode_regime)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "model_drift_diagnostics",
            value
                .model_drift_diagnostics
                .iter()
                .map(encode_model_drift)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "partition_stability_receipts",
            value
                .partition_stability_receipts
                .iter()
                .map(encode_partition_stability)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "holdout_eligibility_receipts",
            value
                .holdout_eligibility_receipts
                .iter()
                .map(encode_eligibility)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "challenger_requirements",
            value
                .challenger_requirements
                .iter()
                .map(encode_requirements)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .unsigneds(
            "counters",
            &[
                as_u64(value.holdout_label_reads)?,
                as_u64(value.holdout_prediction_reads)?,
                as_u64(value.holdout_metric_reads)?,
                as_u64(value.holdout_execution_modes)?,
                as_u64(value.live_outcome_requests)?,
                as_u64(value.live_outcome_openings)?,
                as_u64(value.live_participant_changes)?,
                as_u64(value.winner_selections)?,
                as_u64(value.ranking_creations)?,
                as_u64(value.reward_applications)?,
                as_u64(value.penalty_applications)?,
                as_u64(value.chair_decisions)?,
                as_u64(value.trading_actions)?,
                as_u64(value.network_request_attempts)?,
                as_u64(value.month_view_load_count)?,
                as_u64(value.year_view_load_count)?,
                as_u64(value.artifacts_written)?,
                as_u64(value.duplicate_artifact_count)?,
                as_u64(value.model_refit_count)?,
                as_u64(value.prediction_computation_count)?,
                as_u64(value.evaluation_computation_count)?,
                as_u64(value.diagnostic_computation_count)?,
                value.runtime_duration_ms,
            ],
        )
        .boolean("full_eight_a3_blocked", value.full_eight_a3_blocked)
        .boolean(
            "protected_artifacts_unchanged",
            value.protected_artifacts_unchanged,
        )
        .boolean("active_roster_unchanged", value.active_roster_unchanged)
        .strings("labels", &value.labels)
        .optional_string(
            "diagnostic_journal_digest",
            &value.diagnostic_journal_digest,
        )
        .string("report_digest", &value.report_digest)
        .encode()
}

fn decode_report(bytes: &[u8]) -> Result<MomentumQualifiedSixDiagnosticReportV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedSixDiagnosticReportV1")?;
    let report_version = fields.string("report_version")?;
    let run_mode = fields.string("run_mode")?;
    let status = parse_report_status(&fields.string("status")?)?;
    if fields.string("evidence_class")? != "PostResultDiagnosticOnly" {
        return Err("qualified-six diagnostic report evidence class rejected".to_string());
    }
    let post_result = fields.boolean("post_result")?;
    let confirmatory_claim_allowed = fields.boolean("confirmatory_claim_allowed")?;
    let holdout_authority = fields.boolean("holdout_authority")?;
    let live_authority = fields.boolean("live_authority")?;
    let trading_authority = fields.boolean("trading_authority")?;
    let source_replay_registration_digest =
        fields.optional_string("source_replay_registration_digest")?;
    let source_replay_journal_digest = fields.optional_string("source_replay_journal_digest")?;
    let source_public_report_digest = fields.optional_string("source_public_report_digest")?;
    let diagnostic_registration_digest =
        fields.optional_string("diagnostic_registration_digest")?;
    let participant_ids = fields.strings("participant_ids")?;
    let included_partitions = fields
        .strings("included_partitions")?
        .iter()
        .map(|value| parse_partition(value))
        .collect::<Result<Vec<_>, _>>()?;
    let paired_brier_diagnostics = fields
        .messages("paired_brier_diagnostics")?
        .iter()
        .map(|bytes| decode_paired(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let calendar_stability_diagnostics = fields
        .messages("calendar_stability_diagnostics")?
        .iter()
        .map(|bytes| decode_calendar(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let rolling_stability_diagnostics = fields
        .messages("rolling_stability_diagnostics")?
        .iter()
        .map(|bytes| decode_rolling(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let calibration_diagnostics = fields
        .messages("calibration_diagnostics")?
        .iter()
        .map(|bytes| decode_calibration(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let probability_distribution_diagnostics = fields
        .messages("probability_distribution_diagnostics")?
        .iter()
        .map(|bytes| decode_probability(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let prevalence_drift_diagnostics = fields
        .messages("prevalence_drift_diagnostics")?
        .iter()
        .map(|bytes| decode_prevalence(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let volatility_threshold_low_upper =
        optional_f64(&mut fields, "volatility_threshold_low_upper_bits")?;
    let volatility_threshold_medium_upper =
        optional_f64(&mut fields, "volatility_threshold_medium_upper_bits")?;
    let regime_diagnostics = fields
        .messages("regime_diagnostics")?
        .iter()
        .map(|bytes| decode_regime(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let model_drift_diagnostics = fields
        .messages("model_drift_diagnostics")?
        .iter()
        .map(|bytes| decode_model_drift(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let partition_stability_receipts = fields
        .messages("partition_stability_receipts")?
        .iter()
        .map(|bytes| decode_partition_stability(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let holdout_eligibility_receipts = fields
        .messages("holdout_eligibility_receipts")?
        .iter()
        .map(|bytes| decode_eligibility(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let requirements = fields.messages("challenger_requirements")?;
    if requirements.len() > 1 {
        return Err("qualified-six challenger requirements cardinality rejected".to_string());
    }
    let challenger_requirements = requirements
        .first()
        .map(|bytes| decode_requirements(bytes))
        .transpose()?;
    let counters = fields.unsigneds("counters")?;
    if counters.len() != 23 {
        return Err("qualified-six diagnostic counter cardinality rejected".to_string());
    }
    let value = MomentumQualifiedSixDiagnosticReportV1 {
        report_version,
        run_mode,
        status,
        evidence_class: MomentumQualifiedDiagnosticEvidenceClassV1::PostResultDiagnosticOnly,
        post_result,
        confirmatory_claim_allowed,
        holdout_authority,
        live_authority,
        trading_authority,
        source_replay_registration_digest,
        source_replay_journal_digest,
        source_public_report_digest,
        diagnostic_registration_digest,
        participant_ids,
        included_partitions,
        paired_brier_diagnostics,
        calendar_stability_diagnostics,
        rolling_stability_diagnostics,
        calibration_diagnostics,
        probability_distribution_diagnostics,
        prevalence_drift_diagnostics,
        volatility_threshold_low_upper,
        volatility_threshold_medium_upper,
        regime_diagnostics,
        model_drift_diagnostics,
        partition_stability_receipts,
        holdout_eligibility_receipts,
        challenger_requirements,
        holdout_label_reads: as_usize(counters[0])?,
        holdout_prediction_reads: as_usize(counters[1])?,
        holdout_metric_reads: as_usize(counters[2])?,
        holdout_execution_modes: as_usize(counters[3])?,
        live_outcome_requests: as_usize(counters[4])?,
        live_outcome_openings: as_usize(counters[5])?,
        live_participant_changes: as_usize(counters[6])?,
        winner_selections: as_usize(counters[7])?,
        ranking_creations: as_usize(counters[8])?,
        reward_applications: as_usize(counters[9])?,
        penalty_applications: as_usize(counters[10])?,
        chair_decisions: as_usize(counters[11])?,
        trading_actions: as_usize(counters[12])?,
        network_request_attempts: as_usize(counters[13])?,
        month_view_load_count: as_usize(counters[14])?,
        year_view_load_count: as_usize(counters[15])?,
        full_eight_a3_blocked: fields.boolean("full_eight_a3_blocked")?,
        protected_artifacts_unchanged: fields.boolean("protected_artifacts_unchanged")?,
        active_roster_unchanged: fields.boolean("active_roster_unchanged")?,
        labels: fields.strings("labels")?,
        diagnostic_journal_digest: fields.optional_string("diagnostic_journal_digest")?,
        artifacts_written: as_usize(counters[16])?,
        duplicate_artifact_count: as_usize(counters[17])?,
        model_refit_count: as_usize(counters[18])?,
        prediction_computation_count: as_usize(counters[19])?,
        evaluation_computation_count: as_usize(counters[20])?,
        diagnostic_computation_count: as_usize(counters[21])?,
        runtime_duration_ms: counters[22],
        report_digest: fields.string("report_digest")?,
    };
    fields.finish()?;
    validate_report(&value)?;
    Ok(value)
}

fn persist_records<T>(
    category: &str,
    values: &[T],
    digest: impl Fn(&T) -> &str,
    encode: impl Fn(&T) -> Result<Vec<u8>, String>,
    decode_digest: impl Fn(&[u8]) -> Result<String, String> + Copy,
) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    for value in values {
        let digest = digest(value);
        add_counts(
            &mut counts,
            persist_one(category, digest, &encode(value)?, decode_digest)?,
        );
    }
    Ok(counts)
}

fn persist_suite(suite: &MomentumDiagnosticSuiteReceiptV1) -> Result<(usize, usize), String> {
    persist_one(
        &format!("suites/{}", suite.suite_name),
        &suite.suite_digest,
        &encode_suite(suite)?,
        |bytes| Ok(decode_suite(bytes)?.suite_digest),
    )
}

fn persist_static(
    policies: &[MomentumQualifiedDiagnosticPolicyV1],
    registration: &MomentumQualifiedSixDiagnosticRegistrationV1,
) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    for policy in policies {
        add_counts(
            &mut counts,
            persist_one(
                &format!("policies/{}", policy.policy_name),
                &policy.policy_digest,
                &encode_policy(policy)?,
                |bytes| Ok(decode_policy(bytes)?.policy_digest),
            )?,
        );
    }
    add_counts(
        &mut counts,
        persist_one(
            "registrations",
            &registration.registration_digest,
            &encode_registration(registration)?,
            |bytes| Ok(decode_registration(bytes)?.registration_digest),
        )?,
    );
    Ok(counts)
}

fn require_persisted_registration(
    policies: &[MomentumQualifiedDiagnosticPolicyV1],
    registration: &MomentumQualifiedSixDiagnosticRegistrationV1,
) -> Result<(), String> {
    let persisted = read_single("registrations", decode_registration)?
        .ok_or_else(|| "qualified-six diagnostic registration required".to_string())?;
    if &persisted != registration {
        return Err("qualified-six diagnostic registration identity mismatch".to_string());
    }
    for policy in policies {
        let persisted = read_single(&format!("policies/{}", policy.policy_name), decode_policy)?
            .ok_or_else(|| "qualified-six diagnostic policy required".to_string())?;
        if &persisted != policy {
            return Err("qualified-six diagnostic policy identity mismatch".to_string());
        }
    }
    Ok(())
}

fn persist_diagnostic_suites(
    paired: &[MomentumPairedBrierDiagnosticV1],
    calendar: &[MomentumCalendarStabilityDiagnosticV1],
    rolling: &[MomentumRollingStabilityDiagnosticV1],
    calibration: &[MomentumCalibrationDiagnosticV1],
    probability: &[MomentumProbabilityDistributionDiagnosticV1],
    prevalence: &[MomentumPrevalenceDriftDiagnosticV1],
    regimes: &[MomentumRegimeMetricDiagnosticV1],
    model_drift: &[MomentumModelDriftDiagnosticV1],
    partition_stability: &[MomentumPartitionStabilityReceiptV1],
    eligibility: &[MomentumHoldoutEligibilityReceiptV1],
) -> Result<(Vec<MomentumDiagnosticSuiteReceiptV1>, (usize, usize)), String> {
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_records(
            "paired_diagnostics",
            paired,
            |value| value.diagnostic_digest.as_str(),
            encode_paired,
            |bytes| Ok(decode_paired(bytes)?.diagnostic_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_records(
            "calendar_diagnostics",
            calendar,
            |value| value.diagnostic_digest.as_str(),
            encode_calendar,
            |bytes| Ok(decode_calendar(bytes)?.diagnostic_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_records(
            "rolling_diagnostics",
            rolling,
            |value| value.diagnostic_digest.as_str(),
            encode_rolling,
            |bytes| Ok(decode_rolling(bytes)?.diagnostic_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_records(
            "calibration_diagnostics",
            calibration,
            |value| value.diagnostic_digest.as_str(),
            encode_calibration,
            |bytes| Ok(decode_calibration(bytes)?.diagnostic_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_records(
            "probability_diagnostics",
            probability,
            |value| value.diagnostic_digest.as_str(),
            encode_probability,
            |bytes| Ok(decode_probability(bytes)?.diagnostic_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_records(
            "prevalence_diagnostics",
            prevalence,
            |value| value.diagnostic_digest.as_str(),
            encode_prevalence,
            |bytes| Ok(decode_prevalence(bytes)?.diagnostic_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_records(
            "regime_diagnostics",
            regimes,
            |value| value.diagnostic_digest.as_str(),
            encode_regime,
            |bytes| Ok(decode_regime(bytes)?.diagnostic_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_records(
            "model_drift_diagnostics",
            model_drift,
            |value| value.diagnostic_digest.as_str(),
            encode_model_drift,
            |bytes| Ok(decode_model_drift(bytes)?.diagnostic_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_records(
            "partition_stability_receipts",
            partition_stability,
            |value| value.receipt_digest.as_str(),
            encode_partition_stability,
            |bytes| Ok(decode_partition_stability(bytes)?.receipt_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_records(
            "holdout_eligibility_receipts",
            eligibility,
            |value| value.receipt_digest.as_str(),
            encode_eligibility,
            |bytes| Ok(decode_eligibility(bytes)?.receipt_digest),
        )?,
    );
    let suites = vec![
        build_suite(
            "paired",
            paired
                .iter()
                .map(|value| value.diagnostic_digest.clone())
                .collect(),
        ),
        build_suite(
            "calendar",
            calendar
                .iter()
                .map(|value| value.diagnostic_digest.clone())
                .collect(),
        ),
        build_suite(
            "rolling",
            rolling
                .iter()
                .map(|value| value.diagnostic_digest.clone())
                .collect(),
        ),
        build_suite(
            "calibration",
            calibration
                .iter()
                .map(|value| value.diagnostic_digest.clone())
                .collect(),
        ),
        build_suite(
            "probability",
            probability
                .iter()
                .map(|value| value.diagnostic_digest.clone())
                .collect(),
        ),
        build_suite(
            "prevalence",
            prevalence
                .iter()
                .map(|value| value.diagnostic_digest.clone())
                .collect(),
        ),
        build_suite(
            "regime",
            regimes
                .iter()
                .map(|value| value.diagnostic_digest.clone())
                .collect(),
        ),
        build_suite(
            "model-drift",
            model_drift
                .iter()
                .map(|value| value.diagnostic_digest.clone())
                .collect(),
        ),
        build_suite(
            "partition-stability",
            partition_stability
                .iter()
                .map(|value| value.receipt_digest.clone())
                .collect(),
        ),
        build_suite(
            "holdout-eligibility",
            eligibility
                .iter()
                .map(|value| value.receipt_digest.clone())
                .collect(),
        ),
    ];
    for suite in &suites {
        add_counts(&mut counts, persist_suite(suite)?);
    }
    Ok((suites, counts))
}

fn suite_by_name<'a>(
    suites: &'a [MomentumDiagnosticSuiteReceiptV1],
    name: &str,
) -> Result<&'a MomentumDiagnosticSuiteReceiptV1, String> {
    suites
        .iter()
        .find(|suite| suite.suite_name == name)
        .ok_or_else(|| "qualified-six diagnostic suite unavailable".to_string())
}

fn validate_source_aggregates(source: &MomentumQualifiedDiagnosticSourceV1) -> Result<(), String> {
    for partition in included_partitions() {
        let events = events_for_partition(source, partition);
        for participant_index in 0..5 {
            let participant_id = &participant_ids()[participant_index];
            let metrics = metric_for(&source.header, partition, participant_id)?;
            let scorable = events
                .iter()
                .filter(|event| event.label.is_some())
                .copied()
                .collect::<Vec<_>>();
            let neutral = events.len() - scorable.len();
            let mean_brier = scorable
                .iter()
                .map(|event| event.brier_values[participant_index])
                .sum::<f64>()
                / scorable.len() as f64;
            let correctness = scorable
                .iter()
                .filter(|event| event.correctness[participant_index])
                .count() as f64
                / scorable.len() as f64;
            if events.len() != metrics.total_prediction_events
                || scorable.len() != metrics.scorable_events
                || neutral != metrics.neutral_events
                || metrics.invalid_events != 0
                || metrics
                    .mean_brier_score
                    .is_none_or(|value| (value - mean_brier).abs() > COMPARISON_EPSILON)
                || metrics
                    .binary_correctness
                    .is_none_or(|value| (value - correctness).abs() > COMPARISON_EPSILON)
            {
                return Err("qualified-six diagnostic aggregate replay mismatch".to_string());
            }
        }
    }
    Ok(())
}

fn empty_report(run_mode: &str) -> MomentumQualifiedSixDiagnosticReportV1 {
    let mut value = MomentumQualifiedSixDiagnosticReportV1 {
        report_version: REPORT_VERSION.to_string(),
        run_mode: run_mode.to_string(),
        status: MomentumQualifiedDiagnosticStatusV1::Unregistered,
        evidence_class: MomentumQualifiedDiagnosticEvidenceClassV1::PostResultDiagnosticOnly,
        post_result: true,
        confirmatory_claim_allowed: false,
        holdout_authority: false,
        live_authority: false,
        trading_authority: false,
        source_replay_registration_digest: None,
        source_replay_journal_digest: None,
        source_public_report_digest: None,
        diagnostic_registration_digest: None,
        participant_ids: Vec::new(),
        included_partitions: Vec::new(),
        paired_brier_diagnostics: Vec::new(),
        calendar_stability_diagnostics: Vec::new(),
        rolling_stability_diagnostics: Vec::new(),
        calibration_diagnostics: Vec::new(),
        probability_distribution_diagnostics: Vec::new(),
        prevalence_drift_diagnostics: Vec::new(),
        volatility_threshold_low_upper: None,
        volatility_threshold_medium_upper: None,
        regime_diagnostics: Vec::new(),
        model_drift_diagnostics: Vec::new(),
        partition_stability_receipts: Vec::new(),
        holdout_eligibility_receipts: Vec::new(),
        challenger_requirements: None,
        holdout_label_reads: 0,
        holdout_prediction_reads: 0,
        holdout_metric_reads: 0,
        holdout_execution_modes: 0,
        live_outcome_requests: 0,
        live_outcome_openings: 0,
        live_participant_changes: 0,
        winner_selections: 0,
        ranking_creations: 0,
        reward_applications: 0,
        penalty_applications: 0,
        chair_decisions: 0,
        trading_actions: 0,
        network_request_attempts: 0,
        month_view_load_count: 0,
        year_view_load_count: 0,
        full_eight_a3_blocked: true,
        protected_artifacts_unchanged: true,
        active_roster_unchanged: true,
        labels: PUBLIC_LABELS
            .iter()
            .map(|label| (*label).to_string())
            .collect(),
        diagnostic_journal_digest: None,
        artifacts_written: 0,
        duplicate_artifact_count: 0,
        model_refit_count: 0,
        prediction_computation_count: 0,
        evaluation_computation_count: 0,
        diagnostic_computation_count: 0,
        runtime_duration_ms: 0,
        report_digest: String::new(),
    };
    value.report_digest = report_digest(&value);
    value
}

fn apply_registration_to_report(
    report: &mut MomentumQualifiedSixDiagnosticReportV1,
    registration: &MomentumQualifiedSixDiagnosticRegistrationV1,
) {
    report.status = MomentumQualifiedDiagnosticStatusV1::Registered;
    report.source_replay_registration_digest =
        Some(registration.source_replay_registration_digest.clone());
    report.source_replay_journal_digest = Some(registration.source_replay_journal_digest.clone());
    report.source_public_report_digest = Some(registration.source_public_report_digest.clone());
    report.diagnostic_registration_digest = Some(registration.registration_digest.clone());
    report.participant_ids = registration.included_participant_ids.clone();
    report.included_partitions = registration.included_partitions.clone();
}

fn duplicate_complete_report(
    mut report: MomentumQualifiedSixDiagnosticReportV1,
    mode: MomentumQualifiedDiagnosticRunModeV1,
    started: Instant,
) -> Result<MomentumQualifiedSixDiagnosticReportV1, String> {
    let header = load_momentum_qualified_diagnostic_source_header_v1()?;
    let protected = momentum_qualified_replay_protected_state_v1()?;
    if report.source_replay_registration_digest.as_deref()
        != Some(header.registration_digest.as_str())
        || report.source_replay_journal_digest.as_deref()
            != Some(header.replay_journal_digest.as_str())
        || report.source_public_report_digest.as_deref()
            != Some(header.public_report_digest.as_str())
        || protected.live_tree_digest != header.protected_live_tree_digest
        || protected.active_roster_digest != header.protected_active_roster_digest
        || protected.live_outcome_requests != 0
        || protected.live_outcome_openings != 0
        || protected.epoch_three_registered
    {
        return Err("qualified-six diagnostic protected source changed".to_string());
    }
    report.run_mode = mode.as_str().to_string();
    report.artifacts_written = 0;
    report.duplicate_artifact_count = count_artifacts()?;
    report.model_refit_count = 0;
    report.prediction_computation_count = 0;
    report.evaluation_computation_count = 0;
    report.diagnostic_computation_count = 0;
    report.runtime_duration_ms = started.elapsed().as_millis() as u64;
    report.report_digest = report_digest(&report);
    validate_report(&report)?;
    Ok(report)
}

fn count_artifacts() -> Result<usize, String> {
    fn count(root: &Path) -> Result<usize, String> {
        if !root.exists() {
            return Ok(0);
        }
        let mut total = 0usize;
        for entry in fs::read_dir(root)
            .map_err(|_| "qualified-six diagnostic artifact count failed".to_string())?
        {
            let path = entry
                .map_err(|_| "qualified-six diagnostic artifact count failed".to_string())?
                .path();
            if path.is_dir() {
                total += count(&path)?;
            } else if path.extension().is_some_and(|extension| extension == "pb") {
                total += 1;
            }
        }
        Ok(total)
    }
    count(Path::new(ROOT))
}

fn run_diagnostics_inner(
    mode: MomentumQualifiedDiagnosticRunModeV1,
) -> Result<MomentumQualifiedSixDiagnosticReportV1, String> {
    let started = Instant::now();
    if let Some(report) = read_single("final_reports", decode_report)? {
        return duplicate_complete_report(report, mode, started);
    }
    let protected_before = momentum_qualified_replay_protected_state_v1()?;
    let header = load_momentum_qualified_diagnostic_source_header_v1()?;
    let policies = build_policies()?;
    let registration = build_registration(&header, &policies)?;
    validate_header_registration(&header, &registration)?;
    let persisted_registration = read_single("registrations", decode_registration)?;
    if mode == MomentumQualifiedDiagnosticRunModeV1::Status && persisted_registration.is_none() {
        let protected_after = momentum_qualified_replay_protected_state_v1()?;
        let mut report = empty_report(mode.as_str());
        report.protected_artifacts_unchanged = protected_before == protected_after;
        report.active_roster_unchanged =
            protected_before.active_roster_digest == protected_after.active_roster_digest;
        report.runtime_duration_ms = started.elapsed().as_millis() as u64;
        report.report_digest = report_digest(&report);
        validate_report(&report)?;
        return Ok(report);
    }
    if let Some(persisted) = persisted_registration {
        if persisted != registration {
            return Err("qualified-six diagnostic registration mismatch".to_string());
        }
    }
    if mode != MomentumQualifiedDiagnosticRunModeV1::RegisterAndExecuteLocal {
        let protected_after = momentum_qualified_replay_protected_state_v1()?;
        let mut report = empty_report(mode.as_str());
        apply_registration_to_report(&mut report, &registration);
        report.protected_artifacts_unchanged = protected_before == protected_after;
        report.active_roster_unchanged =
            protected_before.active_roster_digest == protected_after.active_roster_digest;
        report.runtime_duration_ms = started.elapsed().as_millis() as u64;
        report.report_digest = report_digest(&report);
        validate_report(&report)?;
        return Ok(report);
    }

    let mut counts = persist_static(&policies, &registration)?;
    require_persisted_registration(&policies, &registration)?;

    // Private development/validation evidence is first opened only after the
    // post-result registration and all fixed policies have been persisted and reopened.
    let source = load_momentum_qualified_diagnostic_source_v1(&header)?;
    validate_source_aggregates(&source)?;

    // Development-only thresholds are persisted and reopened before any validation
    // regime assignment is performed.
    let thresholds = derive_regime_thresholds(&source, &registration)?;
    add_counts(
        &mut counts,
        persist_one(
            "regime_thresholds",
            &thresholds.threshold_digest,
            &encode_threshold(&thresholds)?,
            |bytes| Ok(decode_threshold(bytes)?.threshold_digest),
        )?,
    );
    let reopened_thresholds = read_single("regime_thresholds", decode_threshold)?
        .ok_or_else(|| "qualified-six diagnostic regime threshold reopen failed".to_string())?;
    if reopened_thresholds != thresholds {
        return Err("qualified-six diagnostic regime threshold mismatch".to_string());
    }

    let paired = compute_paired(&source)?;
    let calendar = compute_calendar(&source)?;
    let rolling = compute_rolling(&source)?;
    let calibration = compute_calibration(&source)?;
    let probability = compute_probability_distributions(&source)?;
    let prevalence = compute_prevalence(&source)?;
    let regimes = compute_regimes(&source, &reopened_thresholds)?;
    let model_drift = compute_model_drift(&source)?;
    let partition_stability = compute_partition_stability(&header)?;
    let eligibility = compute_holdout_eligibility(&header)?;
    let requirements = build_challenger_requirements(&registration, &header, &partition_stability)?;
    let (suites, suite_counts) = persist_diagnostic_suites(
        &paired,
        &calendar,
        &rolling,
        &calibration,
        &probability,
        &prevalence,
        &regimes,
        &model_drift,
        &partition_stability,
        &eligibility,
    )?;
    add_counts(&mut counts, suite_counts);

    let mut journal = MomentumQualifiedDiagnosticJournalV1 {
        journal_version: JOURNAL_VERSION.to_string(),
        registration_digest: registration.registration_digest.clone(),
        source_replay_journal_digest: header.replay_journal_digest.clone(),
        regime_threshold_digest: reopened_thresholds.threshold_digest.clone(),
        paired_suite_digest: suite_by_name(&suites, "paired")?.suite_digest.clone(),
        calendar_suite_digest: suite_by_name(&suites, "calendar")?.suite_digest.clone(),
        rolling_suite_digest: suite_by_name(&suites, "rolling")?.suite_digest.clone(),
        calibration_suite_digest: suite_by_name(&suites, "calibration")?.suite_digest.clone(),
        probability_suite_digest: suite_by_name(&suites, "probability")?.suite_digest.clone(),
        prevalence_suite_digest: suite_by_name(&suites, "prevalence")?.suite_digest.clone(),
        regime_suite_digest: suite_by_name(&suites, "regime")?.suite_digest.clone(),
        model_drift_suite_digest: suite_by_name(&suites, "model-drift")?.suite_digest.clone(),
        partition_stability_suite_digest: suite_by_name(&suites, "partition-stability")?
            .suite_digest
            .clone(),
        holdout_eligibility_suite_digest: suite_by_name(&suites, "holdout-eligibility")?
            .suite_digest
            .clone(),
        challenger_requirements_digest: requirements.requirements_digest.clone(),
        holdout_label_reads: 0,
        holdout_prediction_reads: 0,
        holdout_metric_reads: 0,
        live_outcome_requests: 0,
        live_outcome_openings: 0,
        deterministic: true,
        journal_digest: String::new(),
    };
    journal.journal_digest = journal_digest(&journal);
    validate_journal(&journal)?;
    add_counts(
        &mut counts,
        persist_one(
            "diagnostic_journals",
            &journal.journal_digest,
            &encode_journal(&journal)?,
            |bytes| Ok(decode_journal(bytes)?.journal_digest),
        )?,
    );

    let protected_after = momentum_qualified_replay_protected_state_v1()?;
    let current_header = load_momentum_qualified_diagnostic_source_header_v1()?;
    if current_header != header {
        return Err("qualified-six diagnostic source changed during execution".to_string());
    }
    let mut report = empty_report(mode.as_str());
    apply_registration_to_report(&mut report, &registration);
    report.status = MomentumQualifiedDiagnosticStatusV1::Complete;
    report.paired_brier_diagnostics = paired;
    report.calendar_stability_diagnostics = calendar;
    report.rolling_stability_diagnostics = rolling;
    report.calibration_diagnostics = calibration;
    report.probability_distribution_diagnostics = probability;
    report.prevalence_drift_diagnostics = prevalence;
    report.volatility_threshold_low_upper = Some(reopened_thresholds.low_volatility_upper);
    report.volatility_threshold_medium_upper = Some(reopened_thresholds.medium_volatility_upper);
    report.regime_diagnostics = regimes;
    report.model_drift_diagnostics = model_drift;
    report.partition_stability_receipts = partition_stability;
    report.holdout_eligibility_receipts = eligibility;
    report.challenger_requirements = Some(requirements);
    report.diagnostic_journal_digest = Some(journal.journal_digest);
    report.protected_artifacts_unchanged = protected_before == protected_after;
    report.active_roster_unchanged =
        protected_before.active_roster_digest == protected_after.active_roster_digest;
    report.artifacts_written = counts.0 + 2;
    report.duplicate_artifact_count = counts.1;
    report.diagnostic_computation_count = source.events.len()
        + source.refits.len()
        + report.paired_brier_diagnostics.len()
        + report.calendar_stability_diagnostics.len()
        + report.rolling_stability_diagnostics.len()
        + report.calibration_diagnostics.len()
        + report.probability_distribution_diagnostics.len()
        + report.prevalence_drift_diagnostics.len()
        + report.regime_diagnostics.len()
        + report.model_drift_diagnostics.len()
        + report.partition_stability_receipts.len()
        + report.holdout_eligibility_receipts.len();
    report.runtime_duration_ms = started.elapsed().as_millis() as u64;
    report.report_digest = report_digest(&report);
    if let Some(requirements) = &mut report.challenger_requirements {
        requirements.source_diagnostic_report_digest = report.report_digest.clone();
        requirements.requirements_digest = requirements_digest(requirements);
    }
    report.report_digest = report_digest(&report);
    validate_report(&report)?;
    let requirements = report
        .challenger_requirements
        .as_ref()
        .ok_or_else(|| "qualified-six challenger requirements unavailable".to_string())?;
    add_counts(
        &mut counts,
        persist_one(
            "challenger_requirements",
            &requirements.requirements_digest,
            &encode_requirements(requirements)?,
            |bytes| Ok(decode_requirements(bytes)?.requirements_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "final_reports",
            &report.report_digest,
            &encode_report(&report)?,
            |bytes| Ok(decode_report(bytes)?.report_digest),
        )?,
    );
    if counts.0 != report.artifacts_written {
        return Err("qualified-six diagnostic write accounting mismatch".to_string());
    }
    let reopened = read_single("final_reports", decode_report)?
        .ok_or_else(|| "qualified-six diagnostic final report reopen failed".to_string())?;
    if reopened != report {
        return Err("qualified-six diagnostic final report mismatch".to_string());
    }
    Ok(report)
}

pub fn run_momentum_qualified_six_diagnostics_v1(
    mode: MomentumQualifiedDiagnosticRunModeV1,
) -> Result<MomentumQualifiedSixDiagnosticReportV1, String> {
    match run_diagnostics_inner(mode) {
        Ok(report) => Ok(report),
        Err(error)
            if error.contains("artifact")
                || error.contains("conflict")
                || error.contains("mismatch")
                || error.contains("source changed") =>
        {
            let mut report = empty_report(mode.as_str());
            report.status = MomentumQualifiedDiagnosticStatusV1::IntegrityFailure;
            report.report_digest = report_digest(&report);
            validate_report(&report)?;
            Ok(report)
        }
        Err(error) => Err(error),
    }
}

pub(super) fn read_momentum_qualified_six_diagnostic_report_snapshot_v1()
-> Result<Option<MomentumQualifiedSixDiagnosticReportV1>, String> {
    read_single("final_reports", decode_report)
}

pub fn read_momentum_qualified_six_challenger_requirements_v1()
-> Result<Option<MomentumQualifiedChallengerRequirementsV1>, String> {
    Ok(read_single("final_reports", decode_report)?
        .and_then(|report| report.challenger_requirements))
}

pub fn format_momentum_qualified_six_diagnostics_text_v1(
    report: &MomentumQualifiedSixDiagnosticReportV1,
) -> String {
    use std::fmt::Write as _;
    let mut output = String::new();
    let _ = writeln!(output, "status={:?}", report.status);
    let _ = writeln!(output, "evidence_class={:?}", report.evidence_class);
    let _ = writeln!(output, "post_result={}", report.post_result);
    let _ = writeln!(
        output,
        "confirmatory_claim_allowed={}",
        report.confirmatory_claim_allowed
    );
    let _ = writeln!(
        output,
        "diagnostic_registration_digest={:?}",
        report.diagnostic_registration_digest
    );
    let _ = writeln!(
        output,
        "source_replay_registration_digest={:?}",
        report.source_replay_registration_digest
    );
    let _ = writeln!(
        output,
        "source_replay_journal_digest={:?}",
        report.source_replay_journal_digest
    );
    let _ = writeln!(
        output,
        "source_public_report_digest={:?}",
        report.source_public_report_digest
    );
    for paired in &report.paired_brier_diagnostics {
        let _ = writeln!(
            output,
            "paired={};partition={};count={};mean={:.12};median={:.12};negative={};positive={};equivalent={}",
            paired.participant_id,
            paired.partition.as_str(),
            paired.paired_event_count,
            paired.mean_delta,
            paired.median_delta,
            paired.negative_delta_count,
            paired.positive_delta_count,
            paired.equivalent_delta_count,
        );
    }
    for stability in &report.partition_stability_receipts {
        let _ = writeln!(
            output,
            "partition_stability={};classification={:?}",
            stability.participant_id, stability.classification
        );
    }
    for eligibility in &report.holdout_eligibility_receipts {
        let _ = writeln!(
            output,
            "holdout_eligibility={};classification={:?}",
            eligibility.participant_id, eligibility.eligibility
        );
    }
    let _ = writeln!(
        output,
        "diagnostic_counts=calendar:{};rolling:{};calibration:{};probability:{};prevalence:{};regime:{};model_drift:{}",
        report.calendar_stability_diagnostics.len(),
        report.rolling_stability_diagnostics.len(),
        report.calibration_diagnostics.len(),
        report.probability_distribution_diagnostics.len(),
        report.prevalence_drift_diagnostics.len(),
        report.regime_diagnostics.len(),
        report.model_drift_diagnostics.len(),
    );
    let _ = writeln!(
        output,
        "volatility_thresholds=low_upper:{:?};medium_upper:{:?}",
        report.volatility_threshold_low_upper, report.volatility_threshold_medium_upper
    );
    let _ = writeln!(
        output,
        "holdout=labels:{};predictions:{};metrics:{};execution_modes:{}",
        report.holdout_label_reads,
        report.holdout_prediction_reads,
        report.holdout_metric_reads,
        report.holdout_execution_modes,
    );
    let _ = writeln!(
        output,
        "authority=winner:{};ranking:{};reward:{};penalty:{};chair:{};trading:{};network:{}",
        report.winner_selections,
        report.ranking_creations,
        report.reward_applications,
        report.penalty_applications,
        report.chair_decisions,
        report.trading_actions,
        report.network_request_attempts,
    );
    let _ = writeln!(output, "labels={}", report.labels.join(","));
    let _ = writeln!(
        output,
        "diagnostic_journal_digest={:?}",
        report.diagnostic_journal_digest
    );
    let _ = writeln!(output, "report_digest={}", report.report_digest);
    let _ = writeln!(output, "artifacts_written={}", report.artifacts_written);
    let _ = writeln!(
        output,
        "duplicate_artifact_count={}",
        report.duplicate_artifact_count
    );
    let _ = writeln!(output, "model_refit_count={}", report.model_refit_count);
    let _ = writeln!(
        output,
        "prediction_computation_count={}",
        report.prediction_computation_count
    );
    let _ = writeln!(
        output,
        "evaluation_computation_count={}",
        report.evaluation_computation_count
    );
    let _ = writeln!(
        output,
        "diagnostic_computation_count={}",
        report.diagnostic_computation_count
    );
    let _ = writeln!(output, "runtime_duration_ms={}", report.runtime_duration_ms);
    output
}

pub fn format_momentum_qualified_six_challenger_requirements_text_v1(
    requirements: &MomentumQualifiedChallengerRequirementsV1,
) -> String {
    use std::fmt::Write as _;
    let mut output = String::new();
    let _ = writeln!(
        output,
        "requirements_version={}",
        requirements.requirements_version
    );
    let _ = writeln!(
        output,
        "micro_block_research_priority={:?}",
        requirements.micro_block_research_priority
    );
    let _ = writeln!(
        output,
        "qualified_macro_addition_priority={:?}",
        requirements.qualified_macro_addition_priority
    );
    let _ = writeln!(
        output,
        "new_model_execution_authorized={}",
        requirements.new_model_execution_authorized
    );
    let _ = writeln!(
        output,
        "holdout_execution_authorized={}",
        requirements.holdout_execution_authorized
    );
    let _ = writeln!(
        output,
        "requirements_digest={}",
        requirements.requirements_digest
    );
    output
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    static TEMP_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

    struct TestArtifact(PathBuf);

    impl TestArtifact {
        fn new(name: &str) -> Self {
            Self(std::env::temp_dir().join(format!(
                "soma-qualified-diagnostic-{name}-{}-{}.pb",
                std::process::id(),
                TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            )))
        }
    }

    impl Drop for TestArtifact {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    fn synthetic_metrics(
        participant_index: usize,
        partition: MomentumReplayPartitionV1,
    ) -> MomentumQualifiedParticipantMetricsV1 {
        let scores = match partition {
            MomentumReplayPartitionV1::Development => [0.25, 0.24, 0.26, 0.27, 0.28],
            MomentumReplayPartitionV1::Validation => [0.25, 0.26, 0.24, 0.27, 0.28],
            MomentumReplayPartitionV1::SealedHoldout => unreachable!(),
        };
        MomentumQualifiedParticipantMetricsV1 {
            participant_id: participant_ids()[participant_index].clone(),
            partition,
            total_prediction_events: 1_100,
            scorable_events: 1_078,
            neutral_events: 22,
            invalid_events: 0,
            finite_prediction_count: 1_100,
            probability_collapsed: false,
            mean_brier_score: Some(scores[participant_index]),
            binary_correctness: Some(1.0),
            delta_versus_constant: Some(scores[participant_index] - scores[0]),
            paired_scorable_count: 1_078,
            chronology_audit_passed: true,
            leakage_audit_passed: true,
            metrics_digest: format!("metrics-{participant_index}-{partition:?}"),
        }
    }

    fn synthetic_header() -> MomentumQualifiedDiagnosticSourceHeaderV1 {
        MomentumQualifiedDiagnosticSourceHeaderV1 {
            registration_digest: "source-registration".to_string(),
            replay_journal_digest: "source-journal".to_string(),
            public_report_digest: "source-report".to_string(),
            participant_ids: participant_ids(),
            development_boundary_digest: "development-boundary".to_string(),
            validation_boundary_digest: "validation-boundary".to_string(),
            holdout_boundary_digest: "holdout-boundary".to_string(),
            development_aggregate_digest: "development-aggregate".to_string(),
            validation_aggregate_digest: "validation-aggregate".to_string(),
            development_metrics: (0..5)
                .map(|index| synthetic_metrics(index, MomentumReplayPartitionV1::Development))
                .collect(),
            validation_metrics: (0..5)
                .map(|index| synthetic_metrics(index, MomentumReplayPartitionV1::Validation))
                .collect(),
            benchmark_comparisons: Vec::new(),
            contribution_comparisons: vec![MomentumQualifiedContributionReceiptV1 {
                comparison_version: "test".to_string(),
                added_participant_id: MomentumQualifiedParticipantV1::Q4QualifiedSixFusionLogistic
                    .id()
                    .to_string(),
                baseline_participant_id: MomentumQualifiedParticipantV1::Q2MicroBlockLogistic
                    .id()
                    .to_string(),
                development_delta_bits: Some(0.02_f64.to_bits()),
                validation_delta_bits: Some(0.04_f64.to_bits()),
                paired_development_count: 1_078,
                paired_validation_count: 1_078,
                status: MomentumQualifiedContributionStatusV1::HigherBrierWithAddedBlock,
                comparison_digest: "contribution".to_string(),
            }],
            holdout_label_reads: 0,
            holdout_metric_computations: 0,
            holdout_participant_predictions: 0,
            month_view_load_count: 0,
            year_view_load_count: 0,
            full_eight_a3_blocked: true,
            chronology_audit_passed: true,
            leakage_audit_passed: true,
            protected_live_tree_digest: "live-tree".to_string(),
            protected_active_roster_digest: "roster".to_string(),
        }
    }

    fn synthetic_source() -> MomentumQualifiedDiagnosticSourceV1 {
        let header = synthetic_header();
        let mut events = Vec::new();
        let mut refits = Vec::new();
        for (partition_index, partition) in included_partitions().into_iter().enumerate() {
            let base = 1_600_000_000_000_u64 + partition_index as u64 * 2_000 * 600_000;
            let scores = match partition {
                MomentumReplayPartitionV1::Development => [0.25, 0.24, 0.26, 0.27, 0.28],
                MomentumReplayPartitionV1::Validation => [0.25, 0.26, 0.24, 0.27, 0.28],
                MomentumReplayPartitionV1::SealedHoldout => unreachable!(),
            };
            for index in 0..1_100 {
                let neutral = index % 50 == 0;
                events.push(MomentumQualifiedDiagnosticEventEvidenceV1 {
                    partition,
                    prediction_timestamp_ms: base + index as u64 * 600_000,
                    target_timestamp_ms: base + (index as u64 + 1) * 600_000,
                    event_plan_digest: format!("event-{partition_index}-{index}"),
                    daily_refit_receipt_digest: format!(
                        "refit-{partition_index}-{}",
                        index / 100
                    ),
                    probabilities: (0..5)
                        .map(|participant| {
                            0.44 + participant as f64 * 0.01 + (index % 11) as f64 * 0.001
                        })
                        .collect(),
                    label_status: if neutral {
                        super::super::momentum_qualified_six_replay_v1::MomentumQualifiedLabelStatusV1::Neutral
                    } else if index % 2 == 0 {
                        super::super::momentum_qualified_six_replay_v1::MomentumQualifiedLabelStatusV1::Up
                    } else {
                        super::super::momentum_qualified_six_replay_v1::MomentumQualifiedLabelStatusV1::Down
                    },
                    label: (!neutral).then_some(if index % 2 == 0 { 1.0 } else { 0.0 }),
                    brier_values: (!neutral).then(|| scores.to_vec()).unwrap_or_default(),
                    correctness: (!neutral).then(|| vec![true; 5]).unwrap_or_default(),
                    micro_volatility: Some(0.01 + (index % 30) as f64 * 0.001),
                    daily_trend_return: Some(match index % 3 {
                        0 => -0.01,
                        1 => 0.0,
                        _ => 0.01,
                    }),
                });
            }
            for refit_index in 0..11 {
                refits.push(MomentumQualifiedDiagnosticRefitEvidenceV1 {
                    partition,
                    utc_day_boundary_ms: base / DAY_MS * DAY_MS + refit_index as u64 * DAY_MS,
                    refit_digest: format!("refit-{partition_index}-{refit_index}"),
                    parameter_digests: (0..5)
                        .map(|participant| {
                            format!("parameter-{partition_index}-{refit_index}-{participant}")
                        })
                        .collect(),
                    normalizer_digests: (0..6)
                        .map(|view| format!("normalizer-{partition_index}-{refit_index}-{view}"))
                        .collect(),
                    parameter_norms: (0..5)
                        .map(|participant| participant as f64 + 1.0 + refit_index as f64 * 0.01)
                        .collect(),
                    normalizer_profiles: (0..6)
                        .map(|view| {
                            vec![
                                view as f64,
                                refit_index as f64 * 0.01,
                                1.0,
                                1.0 + refit_index as f64 * 0.001,
                            ]
                        })
                        .collect(),
                    parameter_finite: true,
                    normalizer_finite: true,
                    training_example_count: 512 + refit_index,
                    training_class_prevalence: 0.49 + refit_index as f64 * 0.001,
                    training_loss_finite: true,
                });
            }
        }
        MomentumQualifiedDiagnosticSourceV1 {
            header,
            events,
            refits,
        }
    }

    fn synthetic_registration() -> MomentumQualifiedSixDiagnosticRegistrationV1 {
        let policies = build_policies().expect("policies");
        build_registration(&synthetic_header(), &policies).expect("registration")
    }

    #[test]
    fn sprint99_01_qualified_six_replay_invariants_reopen() {
        let header = load_momentum_qualified_diagnostic_source_header_v1().expect("header");
        assert_eq!(header.participant_ids, participant_ids());
        assert!(header.chronology_audit_passed && header.leakage_audit_passed);
    }

    #[test]
    fn sprint99_02_merged_replay_identities_are_complete() {
        let header = load_momentum_qualified_diagnostic_source_header_v1().expect("header");
        assert!(!header.registration_digest.is_empty());
        assert!(!header.replay_journal_digest.is_empty());
        assert!(!header.public_report_digest.is_empty());
    }

    #[test]
    fn sprint99_03_registration_binds_completed_replay() {
        let header = synthetic_header();
        let registration = synthetic_registration();
        assert!(validate_header_registration(&header, &registration).is_ok());
    }

    #[test]
    fn sprint99_04_evidence_class_is_post_result_only() {
        let registration = synthetic_registration();
        assert!(registration.post_result);
        assert!(!registration.confirmatory_claim_allowed);
    }

    #[test]
    fn sprint99_05_development_and_validation_are_readable() {
        let source = synthetic_source();
        assert_eq!(
            events_for_partition(&source, MomentumReplayPartitionV1::Development).len(),
            1_100
        );
        assert_eq!(
            events_for_partition(&source, MomentumReplayPartitionV1::Validation).len(),
            1_100
        );
    }

    #[test]
    fn sprint99_06_holdout_remains_unreadable() {
        let source = synthetic_source();
        assert!(
            source
                .events
                .iter()
                .all(|event| event.partition != MomentumReplayPartitionV1::SealedHoldout)
        );
        assert!(synthetic_registration().holdout_access_forbidden);
    }

    #[test]
    fn sprint99_07_live_outcome_remains_unreadable() {
        let header = load_momentum_qualified_diagnostic_source_header_v1().expect("header");
        assert_eq!(header.holdout_label_reads, 0);
        let protected = momentum_qualified_replay_protected_state_v1().expect("protected");
        assert_eq!(protected.live_outcome_requests, 0);
        assert_eq!(protected.live_outcome_openings, 0);
    }

    #[test]
    fn sprint99_08_paired_brier_event_sets_are_exact() {
        let paired = compute_paired(&synthetic_source()).expect("paired");
        assert!(paired.iter().all(|value| value.paired_event_count == 1_078));
    }

    #[test]
    fn sprint99_09_neutral_events_are_excluded() {
        let paired = compute_paired(&synthetic_source()).expect("paired");
        assert!(paired.iter().all(|value| value.paired_event_count < 1_100));
    }

    #[test]
    fn sprint99_10_partitions_remain_separate() {
        let paired = compute_paired(&synthetic_source()).expect("paired");
        assert_eq!(
            paired
                .iter()
                .filter(|value| value.partition == MomentumReplayPartitionV1::Development)
                .count(),
            4
        );
        assert_eq!(
            paired
                .iter()
                .filter(|value| value.partition == MomentumReplayPartitionV1::Validation)
                .count(),
            4
        );
    }

    #[test]
    fn sprint99_11_calendar_day_grouping_is_deterministic() {
        let timestamp = 1_600_000_000_000;
        assert_eq!(
            calendar_key(timestamp, MomentumDiagnosticCalendarGrainV1::UtcDay),
            calendar_key(timestamp, MomentumDiagnosticCalendarGrainV1::UtcDay)
        );
    }

    #[test]
    fn sprint99_12_calendar_week_grouping_is_deterministic() {
        let timestamp = 1_600_000_000_000;
        assert_eq!(
            calendar_key(timestamp, MomentumDiagnosticCalendarGrainV1::UtcWeek),
            calendar_key(timestamp, MomentumDiagnosticCalendarGrainV1::UtcWeek)
        );
    }

    #[test]
    fn sprint99_13_calendar_month_grouping_is_deterministic() {
        let timestamp = 1_600_000_000_000;
        assert_eq!(
            calendar_key(timestamp, MomentumDiagnosticCalendarGrainV1::UtcMonth),
            calendar_key(timestamp, MomentumDiagnosticCalendarGrainV1::UtcMonth)
        );
    }

    #[test]
    fn sprint99_14_day_scale_rolling_windows_derive() {
        let paired = paired_values(
            &synthetic_source(),
            MomentumReplayPartitionV1::Development,
            1,
        )
        .expect("paired");
        assert!(!rolling_windows(&paired, ROLLING_DAY_EVENTS, 1).is_empty());
    }

    #[test]
    fn sprint99_15_week_scale_rolling_windows_derive() {
        let paired = paired_values(
            &synthetic_source(),
            MomentumReplayPartitionV1::Development,
            1,
        )
        .expect("paired");
        assert!(!rolling_windows(&paired, ROLLING_WEEK_EVENTS, 1).is_empty());
    }

    #[test]
    fn sprint99_16_calibration_bins_are_fixed() {
        assert_eq!(CALIBRATION_BOUNDARIES.len(), 11);
        assert_eq!(calibration_bin_index(0.0).expect("zero"), 0);
        assert_eq!(calibration_bin_index(1.0).expect("one"), 9);
    }

    #[test]
    fn sprint99_17_probability_quantiles_are_deterministic() {
        let sorted = sorted_finite(&[3.0, 1.0, 2.0]).expect("sorted");
        assert_eq!(quantile_sorted(&sorted, 0.5).expect("median"), 2.0);
    }

    #[test]
    fn sprint99_18_collapse_policy_is_reused_unchanged() {
        let policy = policy_by_name(
            &build_policies().expect("policies"),
            "probability-distribution",
        )
        .expect("policy")
        .clone();
        assert!(
            policy.frozen_values.iter().any(|value| {
                value.contains(&COLLAPSE_VARIANCE_THRESHOLD.to_bits().to_string())
            })
        );
    }

    #[test]
    fn sprint99_19_q0_benchmark_collapse_exemption_is_explicit() {
        let diagnostics =
            compute_probability_distributions(&synthetic_source()).expect("probability");
        assert!(
            diagnostics
                .iter()
                .filter(|value| {
                    value.participant_id
                        == MomentumQualifiedParticipantV1::Q0TrainingPrevalenceConstant.id()
                })
                .all(|value| value.collapse_status
                    == MomentumQualifiedProbabilityCollapseStatusV1::BenchmarkExempt)
        );
    }

    #[test]
    fn sprint99_20_prevalence_uses_scorable_labels_only() {
        let values = compute_prevalence(&synthetic_source()).expect("prevalence");
        assert!(values.iter().all(|value| value.scorable_count == 1_078));
    }

    #[test]
    fn sprint99_21_volatility_thresholds_use_development_only() {
        let source = synthetic_source();
        let thresholds =
            derive_regime_thresholds(&source, &synthetic_registration()).expect("thresholds");
        assert_eq!(thresholds.validation_value_access_count, 0);
    }

    #[test]
    fn sprint99_22_validation_cannot_change_volatility_thresholds() {
        let source = synthetic_source();
        let first = derive_regime_thresholds(&source, &synthetic_registration()).expect("first");
        let mut changed = source.clone();
        for event in changed
            .events
            .iter_mut()
            .filter(|event| event.partition == MomentumReplayPartitionV1::Validation)
        {
            event.micro_volatility = Some(999.0);
        }
        let second = derive_regime_thresholds(&changed, &synthetic_registration()).expect("second");
        assert_eq!(first, second);
    }

    #[test]
    fn sprint99_23_daily_trend_uses_exact_numeric_policy() {
        assert_eq!(trend_regime(0.0), MomentumQualifiedDailyTrendRegimeV1::Flat);
        assert_eq!(
            trend_regime(-0.01),
            MomentumQualifiedDailyTrendRegimeV1::DownTrend
        );
    }

    #[test]
    fn sprint99_24_combined_regime_assignment_does_not_use_targets() {
        let source = synthetic_source();
        let thresholds =
            derive_regime_thresholds(&source, &synthetic_registration()).expect("thresholds");
        let first = compute_regimes(&source, &thresholds).expect("first");
        let mut changed = source.clone();
        for event in &mut changed.events {
            event.target_timestamp_ms += 1;
        }
        let second = compute_regimes(&changed, &thresholds).expect("second");
        assert_eq!(first, second);
    }

    #[test]
    fn sprint99_25_low_support_regime_is_explicit() {
        let source = synthetic_source();
        let thresholds =
            derive_regime_thresholds(&source, &synthetic_registration()).expect("thresholds");
        assert!(
            compute_regimes(&source, &thresholds)
                .expect("regimes")
                .iter()
                .any(|value| value.relation
                    == MomentumDiagnosticRelationV1::InsufficientDiagnosticSupport)
        );
    }

    #[test]
    fn sprint99_26_daily_refit_receipts_reopen_exactly() {
        let source = synthetic_source();
        assert!(source.refits.iter().all(|refit| {
            !refit.refit_digest.is_empty()
                && refit.parameter_digests.len() == 5
                && refit.normalizer_digests.len() == 6
        }));
    }

    #[test]
    fn sprint99_27_model_coefficients_remain_private() {
        let report = empty_report("test");
        assert!(
            !serde_json::to_string(&report)
                .expect("json")
                .contains("coefficient")
        );
    }

    #[test]
    fn sprint99_28_normalizer_values_remain_private() {
        let report = empty_report("test");
        assert!(
            !serde_json::to_string(&report)
                .expect("json")
                .contains("normalizer_profiles")
        );
    }

    #[test]
    fn sprint99_29_parameter_drift_summaries_are_finite() {
        assert!(
            compute_model_drift(&synthetic_source())
                .expect("drift")
                .iter()
                .all(|value| value.maximum_parameter_norm_change.is_finite())
        );
    }

    #[test]
    fn sprint99_30_partition_stability_derives_from_metrics() {
        let values = compute_partition_stability(&synthetic_header()).expect("stability");
        assert_eq!(
            values[0].classification,
            MomentumQualifiedPartitionStabilityV1::DevelopmentOnlyLowerBrier
        );
        assert_eq!(
            values[1].classification,
            MomentumQualifiedPartitionStabilityV1::ValidationOnlyLowerBrier
        );
    }

    #[test]
    fn sprint99_31_validation_only_improvement_is_not_eligible() {
        let values = compute_holdout_eligibility(&synthetic_header()).expect("eligibility");
        assert_eq!(
            values[1].eligibility,
            MomentumQualifiedHoldoutEligibilityV1::NotEligibleForSealedHoldout
        );
    }

    #[test]
    fn sprint99_32_development_only_improvement_is_not_eligible() {
        let values = compute_holdout_eligibility(&synthetic_header()).expect("eligibility");
        assert_eq!(
            values[0].eligibility,
            MomentumQualifiedHoldoutEligibilityV1::NotEligibleForSealedHoldout
        );
    }

    #[test]
    fn sprint99_33_consistently_worse_is_not_eligible() {
        let values = compute_holdout_eligibility(&synthetic_header()).expect("eligibility");
        assert_eq!(
            values[2].eligibility,
            MomentumQualifiedHoldoutEligibilityV1::NotEligibleForSealedHoldout
        );
    }

    #[test]
    fn sprint99_34_only_two_partition_lower_brier_passes_first_gate() {
        let mut header = synthetic_header();
        header.development_metrics[1].delta_versus_constant = Some(-0.01);
        header.validation_metrics[1].delta_versus_constant = Some(-0.01);
        let values = compute_holdout_eligibility(&header).expect("eligibility");
        assert_eq!(
            values[0].eligibility,
            MomentumQualifiedHoldoutEligibilityV1::EligibleForFutureSealedHoldoutEvaluation
        );
    }

    #[test]
    fn sprint99_35_probability_collapse_blocks_candidate() {
        let mut header = synthetic_header();
        header.development_metrics[1].delta_versus_constant = Some(-0.01);
        header.validation_metrics[1].delta_versus_constant = Some(-0.01);
        header.validation_metrics[1].probability_collapsed = true;
        assert_eq!(
            compute_holdout_eligibility(&header).expect("eligibility")[0].eligibility,
            MomentumQualifiedHoldoutEligibilityV1::NotEligibleForSealedHoldout
        );
    }

    #[test]
    fn sprint99_36_chronology_failure_blocks_candidate() {
        let mut header = synthetic_header();
        header.development_metrics[1].delta_versus_constant = Some(-0.01);
        header.validation_metrics[1].delta_versus_constant = Some(-0.01);
        header.validation_metrics[1].chronology_audit_passed = false;
        assert_eq!(
            compute_holdout_eligibility(&header).expect("eligibility")[0].eligibility,
            MomentumQualifiedHoldoutEligibilityV1::NotEligibleForSealedHoldout
        );
    }

    #[test]
    fn sprint99_37_no_winner_is_selected() {
        assert_eq!(empty_report("test").winner_selections, 0);
    }

    #[test]
    fn sprint99_38_challenger_requirements_authorize_no_model() {
        let stability = compute_partition_stability(&synthetic_header()).expect("stability");
        let requirements = build_challenger_requirements(
            &synthetic_registration(),
            &synthetic_header(),
            &stability,
        )
        .expect("requirements");
        assert!(!requirements.new_model_execution_authorized);
    }

    #[test]
    fn sprint99_39_challenger_requirements_authorize_no_holdout() {
        let stability = compute_partition_stability(&synthetic_header()).expect("stability");
        let requirements = build_challenger_requirements(
            &synthetic_registration(),
            &synthetic_header(),
            &stability,
        )
        .expect("requirements");
        assert!(!requirements.holdout_execution_authorized);
    }

    #[test]
    fn sprint99_40_full_eight_remains_blocked() {
        assert!(synthetic_header().full_eight_a3_blocked);
    }

    #[test]
    fn sprint99_41_month_and_year_remain_inaccessible() {
        let header = synthetic_header();
        assert_eq!(header.month_view_load_count, 0);
        assert_eq!(header.year_view_load_count, 0);
    }

    #[test]
    fn sprint99_42_live_counts_and_roster_remain_unchanged() {
        let before = momentum_qualified_replay_protected_state_v1().expect("before");
        let after = momentum_qualified_replay_protected_state_v1().expect("after");
        assert_eq!(before, after);
    }

    #[test]
    fn sprint99_43_reward_and_chair_counters_are_zero() {
        let report = empty_report("test");
        assert_eq!(report.reward_applications, 0);
        assert_eq!(report.chair_decisions, 0);
    }

    #[test]
    fn sprint99_44_network_counters_are_zero() {
        assert_eq!(empty_report("test").network_request_attempts, 0);
    }

    #[test]
    fn sprint99_45_duplicate_diagnostics_write_zero() {
        let path = TestArtifact::new("duplicate");
        let policy = build_policies().expect("policies").remove(0);
        let bytes = encode_policy(&policy).expect("bytes");
        let first = persist_artifact(&path.0, &bytes, &policy.policy_digest, |bytes| {
            Ok(decode_policy(bytes)?.policy_digest)
        })
        .expect("first");
        let second = persist_artifact(&path.0, &bytes, &policy.policy_digest, |bytes| {
            Ok(decode_policy(bytes)?.policy_digest)
        })
        .expect("second");
        assert_eq!(first, (1, 0));
        assert_eq!(second, (0, 1));
    }

    #[test]
    fn sprint99_46_conflicting_diagnostics_reject() {
        let path = TestArtifact::new("conflict");
        let policy = build_policies().expect("policies").remove(0);
        let bytes = encode_policy(&policy).expect("bytes");
        persist_artifact(&path.0, &bytes, &policy.policy_digest, |bytes| {
            Ok(decode_policy(bytes)?.policy_digest)
        })
        .expect("first");
        assert!(
            persist_artifact(&path.0, &bytes, "different", |bytes| {
                Ok(decode_policy(bytes)?.policy_digest)
            })
            .is_err()
        );
    }

    #[test]
    fn sprint99_47_malformed_protobuf_rejects() {
        let policy = build_policies().expect("policies").remove(0);
        let mut bytes = encode_policy(&policy).expect("bytes");
        bytes.truncate(bytes.len() / 2);
        assert!(decode_policy(&bytes).is_err());
    }

    #[test]
    fn sprint99_48_text_and_json_agree() {
        let report = empty_report("test");
        let text = format_momentum_qualified_six_diagnostics_text_v1(&report);
        let json = serde_json::to_value(&report).expect("json");
        for field in [
            "artifacts_written",
            "duplicate_artifact_count",
            "model_refit_count",
            "prediction_computation_count",
            "evaluation_computation_count",
            "diagnostic_computation_count",
            "runtime_duration_ms",
        ] {
            assert!(text.contains(&format!("{field}={}", json[field])));
        }
        assert!(text.contains(&format!("report_digest={}", report.report_digest)));
    }
}
