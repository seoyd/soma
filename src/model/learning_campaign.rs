//! Offline, walk-forward learning orchestration for the shadow momentum model.
//!
//! The campaign owns evidence validation, chronological partitioning, and immutable
//! result records. It never joins the active committee or submits an order.

use std::collections::BTreeSet;

use crate::{
    core::stable_hash_string,
    data::{DataSnapshot, DatasetKind, SnapshotSourceType, historical_replay_dataset_digest_v0},
};

use super::{
    BackendFallbackPolicy, BackendPreference, BaselineComparisonV0, ConstantProbabilityBaselineV0,
    EncodedTrainingExampleV0, EvaluationMetricsV0, FeatureNormalizerV0, FrozenMamba3EncoderV0,
    HeadTrainingConfigV0, IndexRangeV0, LearningError, LinearMomentumBaselineV0,
    LogisticPredictionHeadV0, Mamba3BackendKind, MambaRepresentationValueStatusV0,
    ModelAgentDeploymentStatus, ModelMathematicalStatus, MomentumCandleV0, MomentumFeatureConfigV0,
    MomentumSequenceConfigV0, SandboxModelMetricsV0, SandboxModelVersionJournalV0,
    SandboxModelVersionV0, SequenceExampleV0, apply_sgd_v0, brier_loss_and_gradients_v0,
    build_momentum_features_v0, build_momentum_sequence_examples_v0, evaluate_head_v0,
    mamba_representation_value_status_v0, train_frozen_mamba_head_v0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignSplitPolicyV0 {
    ExpandingWindow,
    RollingWindow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadInitializationPolicyV0 {
    ColdStartEachWindow,
    WarmStartPreviousEligible,
    CompareColdAndWarm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumLearningPathV0 {
    Cold,
    Warm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignShadowSuggestedActionV0 {
    UpwardWatch,
    DownwardWatch,
    Abstain,
}

impl MomentumLearningPathV0 {
    fn label(self) -> &'static str {
        match self {
            Self::Cold => "cold",
            Self::Warm => "warm",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelDriftStatusV0 {
    Stable,
    CalibrationDrift,
    ProbabilityCollapse,
    OverconfidenceIncrease,
    PerformanceDegradation,
    ParameterInstability,
    Mixed,
    InsufficientEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumLearningCampaignStatusV0 {
    Completed,
    Mixed,
    FailedBaselines,
    DriftDetected,
    InsufficientEvidence,
    NoHistoricalLearningEvidence,
    RejectedForSafety,
    BackendUnavailable,
    LeakageInvariantFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignErrorV0 {
    InvalidConfig,
    UnsupportedSplitPolicy,
    InsufficientHistory,
    PurgeGapTooSmall,
    Overflow,
    UnsafeEvidence,
    MutableEvidence,
    CorruptEvidence,
    NonMonotonicEvidence,
    DuplicateTimestamp,
    IncompatibleEvidence,
    BackendUnavailable,
    InvalidWarmParent,
    VersionCycle,
    LeakageInvariantFailed,
    Learning,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProbabilityCollapseConfigV0 {
    pub minimum_probability_stddev: f32,
    pub minimum_prediction_entropy: f32,
    pub maximum_single_side_fraction: f32,
    pub low_saturation_threshold: f32,
    pub high_saturation_threshold: f32,
    pub maximum_saturation_fraction: f32,
    pub minimum_unique_probability_bins: usize,
    pub comparison_epsilon: f32,
    pub minimum_samples: usize,
}

impl Default for ProbabilityCollapseConfigV0 {
    fn default() -> Self {
        Self {
            minimum_probability_stddev: 0.01,
            minimum_prediction_entropy: 0.1,
            maximum_single_side_fraction: 0.98,
            low_saturation_threshold: 0.05,
            high_saturation_threshold: 0.95,
            maximum_saturation_fraction: 0.9,
            minimum_unique_probability_bins: 2,
            comparison_epsilon: 1e-4,
            minimum_samples: 4,
        }
    }
}

impl ProbabilityCollapseConfigV0 {
    pub fn validate(&self) -> Result<(), CampaignErrorV0> {
        if !self.minimum_probability_stddev.is_finite()
            || self.minimum_probability_stddev < 0.0
            || !self.minimum_prediction_entropy.is_finite()
            || self.minimum_prediction_entropy < 0.0
            || !self.maximum_single_side_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.maximum_single_side_fraction)
            || !self.low_saturation_threshold.is_finite()
            || !self.high_saturation_threshold.is_finite()
            || !(0.0..=1.0).contains(&self.low_saturation_threshold)
            || !(0.0..=1.0).contains(&self.high_saturation_threshold)
            || self.low_saturation_threshold >= self.high_saturation_threshold
            || !self.maximum_saturation_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.maximum_saturation_fraction)
            || self.minimum_unique_probability_bins == 0
            || self.minimum_unique_probability_bins > 10
            || !self.comparison_epsilon.is_finite()
            || self.comparison_epsilon < 0.0
            || self.minimum_samples == 0
        {
            Err(CampaignErrorV0::InvalidConfig)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbabilityCollapseSubtypeV0 {
    NearConstantProbability,
    NearZeroProbability,
    NearOneProbability,
    SingleSidePrediction,
    SaturatedProbability,
    LowEntropyPrediction,
    InsufficientUniquePredictions,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProbabilityCollapseMetricsV0 {
    pub sample_count: usize,
    pub mean_probability: f32,
    pub probability_stddev: f32,
    pub minimum_probability: f32,
    pub maximum_probability: f32,
    pub mean_entropy: f32,
    pub unique_probability_bins: usize,
    pub low_saturation_fraction: f32,
    pub high_saturation_fraction: f32,
    pub positive_prediction_fraction: f32,
    pub saturation_fraction: f32,
    pub high_confidence_error_count: usize,
    pub subtypes: Vec<ProbabilityCollapseSubtypeV0>,
}

/// Validation-only policy for deciding whether a checkpoint has enough measured
/// signal to earn a sealed-test evaluation.  It deliberately has no test fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationSignalGateConfigV0 {
    pub minimum_samples: usize,
    pub minimum_probability_stddev: f32,
    pub minimum_entropy: f32,
    pub minimum_brier_resolution: f32,
    pub maximum_saturation_fraction: f32,
    pub maximum_single_side_fraction: f32,
    pub minimum_unique_probability_bins: usize,
    pub maximum_brier_delta_vs_constant: f32,
    pub minimum_rank_auc_margin: Option<f32>,
    pub comparison_epsilon: f32,
    pub brier_bin_count: usize,
}

impl Default for ValidationSignalGateConfigV0 {
    fn default() -> Self {
        Self {
            minimum_samples: 4,
            minimum_probability_stddev: 0.01,
            minimum_entropy: 0.1,
            minimum_brier_resolution: 0.0001,
            maximum_saturation_fraction: 0.9,
            maximum_single_side_fraction: 0.98,
            minimum_unique_probability_bins: 2,
            maximum_brier_delta_vs_constant: 0.0,
            minimum_rank_auc_margin: None,
            comparison_epsilon: 1e-4,
            brier_bin_count: 10,
        }
    }
}

impl ValidationSignalGateConfigV0 {
    pub fn validate(&self) -> Result<(), CampaignErrorV0> {
        let fraction = |value: f32| value.is_finite() && (0.0..=1.0).contains(&value);
        if self.minimum_samples == 0
            || !self.minimum_probability_stddev.is_finite()
            || self.minimum_probability_stddev < 0.0
            || !self.minimum_entropy.is_finite()
            || self.minimum_entropy < 0.0
            || !self.minimum_brier_resolution.is_finite()
            || self.minimum_brier_resolution < 0.0
            || !fraction(self.maximum_saturation_fraction)
            || !fraction(self.maximum_single_side_fraction)
            || self.minimum_unique_probability_bins == 0
            || !self.maximum_brier_delta_vs_constant.is_finite()
            || self.maximum_brier_delta_vs_constant < 0.0
            || self
                .minimum_rank_auc_margin
                .is_some_and(|value| !fraction(value))
            || !self.comparison_epsilon.is_finite()
            || self.comparison_epsilon < 0.0
            || self.brier_bin_count < 2
        {
            Err(CampaignErrorV0::InvalidConfig)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrierDecompositionV0 {
    pub brier_score: f32,
    pub reliability: f32,
    pub resolution: f32,
    pub uncertainty: f32,
    pub bin_count: usize,
    pub occupied_bin_count: usize,
    pub sample_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryRankAucStatusV0 {
    Defined,
    UndefinedSingleClass,
    InsufficientClassCount,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryRankAucV0 {
    pub status: BinaryRankAucStatusV0,
    pub value: Option<f32>,
    pub positive_count: usize,
    pub negative_count: usize,
    pub tie_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForecastMetricBundleV0 {
    pub evaluation: EvaluationMetricsV0,
    pub collapse: ProbabilityCollapseMetricsV0,
    pub brier: BrierDecompositionV0,
    pub rank_auc: BinaryRankAucV0,
    pub finite: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationSignalStatusV0 {
    Usable,
    ConstantLike,
    NoResolution,
    NoDiscrimination,
    Collapsed,
    NumericallyInvalid,
    InsufficientSamples,
    SingleClassValidation,
    TemporalOutOfSupport,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckpointEligibilityV0 {
    Eligible,
    RejectedCollapse,
    RejectedNoResolution,
    RejectedNoDiscrimination,
    RejectedConstantLike,
    RejectedWorseThanConstant,
    RejectedInsufficientSamples,
    RejectedSingleClassValidation,
    RejectedNumericalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckpointTrajectoryStatusV0 {
    HealthyEligibleCheckpointFound,
    CollapsedCheckpointSelectedByOldPolicy,
    NoUsableValidationSignal,
    ValidationSignalTooWeak,
    ValidationSignalUnstable,
    WarmStartLockIn,
    CandidateNumericalFailure,
    InsufficientValidationSamples,
    NondeterministicTrajectory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckpointGeneralizationStatusV0 {
    TestNotEvaluatedNoEligibleCheckpoint,
    GeneralizedWithoutCollapse,
    TemporalGeneralizationCollapse,
    TestPerformanceDegradation,
    TestCalibrationFailure,
    TestDiscriminationFailure,
    Mixed,
    InsufficientEvidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckpointObservationV0 {
    pub epoch: usize,
    pub train: ForecastMetricBundleV0,
    pub validation: ForecastMetricBundleV0,
    pub head_digest: String,
    pub weight_norm: f32,
    pub bias: f32,
    pub gradient_norm: f32,
    pub update_norm: f32,
    pub finite: bool,
    pub signal_status: ValidationSignalStatusV0,
    pub eligibility: CheckpointEligibilityV0,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointRefV0 {
    pub candidate: MomentumForensicCandidateV0,
    pub epoch: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EligibleCheckpointV0 {
    pub reference: CheckpointRefV0,
    pub validation_brier: f32,
    pub reliability: f32,
    pub resolution: f32,
    pub entropy: f32,
    pub probability_stddev: f32,
    pub rank_auc: Option<f32>,
    pub head_digest: String,
    pub update_norm: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EligibleCheckpointFrontierV0 {
    pub checkpoints: Vec<EligibleCheckpointV0>,
    pub rejected_count_by_reason: Vec<(CheckpointEligibilityV0, usize)>,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckpointTrajectoryV0 {
    pub candidate: MomentumForensicCandidateV0,
    pub checkpoints: Vec<CheckpointObservationV0>,
    pub frontier: EligibleCheckpointFrontierV0,
    pub selected_checkpoint: Option<CheckpointRefV0>,
    pub status: CheckpointTrajectoryStatusV0,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShadowLearningAbstentionReasonV0 {
    NoUsableValidationSignal,
    AllCandidatesCollapsed,
    InsufficientValidationSamples,
    SingleClassValidation,
    NumericalFailure,
    TemporalOutOfSupport,
    TemporalSupportUnavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShadowLearningAbstentionV0 {
    pub agent_id: String,
    pub campaign_id: String,
    pub window_id: String,
    pub reason: ShadowLearningAbstentionReasonV0,
    pub eligible_to_vote: bool,
    pub eligible_to_execute: bool,
    pub eligible_for_promotion: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DistributionShiftMetricBundleV0 {
    pub sample_count_reference: usize,
    pub sample_count_target: usize,
    pub dimensions: usize,
    pub mean_absolute_standardized_mean_shift: f32,
    pub maximum_absolute_standardized_mean_shift: f32,
    pub mean_absolute_log_variance_ratio: f32,
    pub maximum_absolute_log_variance_ratio: f32,
    pub out_of_support_fraction: f32,
    pub dimensions_out_of_support: usize,
    pub finite: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DistributionSupportEnvelopeV0 {
    pub means: Vec<f32>,
    pub scales: Vec<f32>,
    pub lower_z_limit: f32,
    pub upper_z_limit: f32,
    pub maximum_out_of_support_fraction: f32,
    pub epsilon: f32,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShadowSupportGateConfigV0 {
    pub minimum_samples: usize,
    pub maximum_mean_standardized_shift: f32,
    pub maximum_dimension_standardized_shift: f32,
    pub maximum_mean_log_variance_ratio: f32,
    pub maximum_out_of_support_fraction: f32,
    pub minimum_validation_coverage: f32,
    pub comparison_epsilon: f32,
}

impl Default for ShadowSupportGateConfigV0 {
    fn default() -> Self {
        Self {
            minimum_samples: 4,
            maximum_mean_standardized_shift: 3.0,
            maximum_dimension_standardized_shift: 6.0,
            maximum_mean_log_variance_ratio: 2.0,
            maximum_out_of_support_fraction: 0.1,
            minimum_validation_coverage: 0.8,
            comparison_epsilon: 1e-6,
        }
    }
}
impl ShadowSupportGateConfigV0 {
    pub fn validate(&self) -> Result<(), CampaignErrorV0> {
        if self.minimum_samples == 0
            || [
                self.maximum_mean_standardized_shift,
                self.maximum_dimension_standardized_shift,
                self.maximum_mean_log_variance_ratio,
                self.comparison_epsilon,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
            || [
                self.maximum_out_of_support_fraction,
                self.minimum_validation_coverage,
            ]
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            Err(CampaignErrorV0::InvalidConfig)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowSupportDecisionV0 {
    InSupport,
    OutOfSupport,
    SupportGateUnavailable,
    InsufficientEvidence,
    NumericalFailure,
    NotEvaluated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportEnvelopeConstructionStatusV0 {
    Ready,
    InsufficientTrainingSamples,
    MissingRepresentationData,
    DimensionMismatch,
    ConstantDimensionPolicyFailure,
    NonFiniteInput,
    NonFiniteStatistics,
    DigestMismatch,
    ConstructionFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportGateApplicabilityStatusV0 {
    Applicable,
    ApplicableWithLimitedDiagnostics,
    InsufficientValidationSamples,
    ValidationAuditRejected,
    RequiredMetricUnavailable,
    UnsupportedRepresentationShape,
    NumericalFailure,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportMetricIdV0 {
    ValidationSampleCount,
    ValidationCoverage,
    MeanStandardizedShift,
    MaximumStandardizedShift,
    MeanLogVarianceRatio,
    OutOfSupportFraction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportMetricDecisionV0 {
    Passed,
    Breached,
    NotApplicable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SupportMetricEvaluationV0 {
    pub metric_id: SupportMetricIdV0,
    pub measured_value: Option<f32>,
    pub configured_threshold: Option<f32>,
    pub decision: SupportMetricDecisionV0,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportEnvelopeTraceV0 {
    pub window_id: String,
    pub candidate_id: String,
    pub checkpoint_epoch: usize,
    pub construction_status: SupportEnvelopeConstructionStatusV0,
    pub sample_count: usize,
    pub dimension_count: usize,
    pub means_finite: bool,
    pub scales_finite: bool,
    pub constant_dimension_count: usize,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidationSupportResultV0 {
    pub gate_applicability: SupportGateApplicabilityStatusV0,
    pub support_decision: ShadowSupportDecisionV0,
    pub first_breach_metric: Option<SupportMetricIdV0>,
    pub breached_metric_count: usize,
    pub missing_required_metric_count: usize,
    pub missing_optional_metric_count: usize,
    pub result_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrainHistorySupportAuditStatusV0 {
    SelfConsistent,
    ChronologicallyNonstationary,
    OverRejectingOnTrainingHistory,
    InsufficientAuditEvidence,
    NumericalFailure,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrainHistorySupportAuditV0 {
    pub fixed_chronological_fold_count: usize,
    pub in_support_fold_count: usize,
    pub out_of_support_fold_count: usize,
    pub insufficient_evidence_fold_count: usize,
    pub unavailable_fold_count: usize,
    pub first_breach_metric: Option<SupportMetricIdV0>,
    pub status: TrainHistorySupportAuditStatusV0,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumSupportTraceV0 {
    pub envelope: SupportEnvelopeTraceV0,
    pub metrics: Vec<SupportMetricEvaluationV0>,
    pub validation: ValidationSupportResultV0,
    pub train_history_audit: TrainHistorySupportAuditV0,
    pub test_support_decision: ShadowSupportDecisionV0,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EarliestTemporalShiftStageV0 {
    None,
    RawFeatures,
    NormalizedFeatures,
    Sequences,
    FrozenRepresentations,
    RepresentationScale,
    Logits,
    Probabilities,
    OutcomesOnly,
    MultipleStages,
    InsufficientEvidence,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalDistributionShiftStatusV0 {
    Stable,
    NormalizedFeatureShift,
    FrozenRepresentationShift,
    LogitDistributionShift,
    ProbabilityDistributionShift,
    MultiStageShift,
    InsufficientSamples,
    NumericalFailure,
    Unknown,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TemporalGeneralizationResultV0 {
    pub validation_support_decision: ShadowSupportDecisionV0,
    pub test_support_decision: ShadowSupportDecisionV0,
    pub validation_support_coverage: f32,
    pub support_envelope_digest: String,
    pub support_envelope_constant_dimension_count: usize,
    pub train_history_support_audit: TrainHistorySupportAuditV0,
    pub raw_feature_shift: Option<DistributionShiftMetricBundleV0>,
    pub normalized_feature_shift: Option<DistributionShiftMetricBundleV0>,
    pub sequence_shift: DistributionShiftMetricBundleV0,
    pub frozen_representation_shift: DistributionShiftMetricBundleV0,
    pub representation_shift: DistributionShiftMetricBundleV0,
    pub validation_representation_shift: DistributionShiftMetricBundleV0,
    pub logit_shift: DistributionShiftMetricBundleV0,
    pub probability_shift: DistributionShiftMetricBundleV0,
    pub outcome_shift: DistributionShiftMetricBundleV0,
    pub earliest_shift_stage: EarliestTemporalShiftStageV0,
    pub shift_status: TemporalDistributionShiftStatusV0,
    pub root_cause: ProbabilityCollapseRootCauseV0,
    pub counterfactual_test_evaluated: bool,
    pub decision_digest: String,
}

impl ProbabilityCollapseMetricsV0 {
    pub fn is_collapsed(&self) -> bool {
        !self.subtypes.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbabilityCollapseDiagnosticStatusV0 {
    Reproduced,
    NotReproduced,
    RootCauseIdentified,
    MultipleContributingCauses,
    InsufficientDiagnosticEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbabilityCollapseRootCauseV0 {
    RawFeatureCollapse,
    NormalizedFeatureCollapse,
    SequenceDiversityCollapse,
    EncoderRepresentationCollapse,
    RepresentationScaleMismatch,
    FeatureScaleDrift,
    SequenceSupportBreach,
    FrozenRepresentationSupportBreach,
    RepresentationScaleDrift,
    HeadBiasSensitivity,
    LogitVarianceCollapse,
    OutcomePrevalenceShift,
    ImplementationBug,
    HeadInitializationCollapse,
    GradientVanishing,
    GradientExplosion,
    OptimizerInstability,
    BiasDominatedPrediction,
    ValidationCheckpointCollapse,
    WarmStartLockIn,
    ClassPrevalenceDominance,
    ProbabilitySaturation,
    CalibrationOnlyFailure,
    Mixed,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumForensicCandidateV0 {
    C0Reference,
    C1RepresentationNormalized,
    C2PrevalenceBias,
    C3Combined,
}

impl MomentumForensicCandidateV0 {
    fn label(self) -> &'static str {
        match self {
            Self::C0Reference => "c0_reference",
            Self::C1RepresentationNormalized => "c1_representation_normalized",
            Self::C2PrevalenceBias => "c2_prevalence_bias",
            Self::C3Combined => "c3_combined",
        }
    }

    fn uses_representation_normalization(self) -> bool {
        matches!(self, Self::C1RepresentationNormalized | Self::C3Combined)
    }

    fn uses_prevalence_bias(self) -> bool {
        matches!(self, Self::C2PrevalenceBias | Self::C3Combined)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RepresentationNormalizerV0 {
    pub means: Vec<f32>,
    pub scales: Vec<f32>,
    pub constant_dimension_indices: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumForensicCandidateResultV0 {
    pub candidate: MomentumForensicCandidateV0,
    pub frozen_head: LogisticPredictionHeadV0,
    pub validation: EvaluationMetricsV0,
    pub validation_collapse: ProbabilityCollapseMetricsV0,
    pub test: Option<EvaluationMetricsV0>,
    pub test_collapse: Option<ProbabilityCollapseMetricsV0>,
    pub eligible_for_selection: bool,
    pub trajectory: Option<CheckpointTrajectoryV0>,
    pub selected_checkpoint: Option<CheckpointRefV0>,
    pub generalization_status: CheckpointGeneralizationStatusV0,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumLearningDiagnosticTraceV0 {
    pub window_id: String,
    pub path: MomentumLearningPathV0,
    pub raw_feature_stddev: f32,
    pub normalized_feature_stddev: f32,
    pub sequence_count: usize,
    pub duplicate_sequence_count: usize,
    pub representation_stddev: f32,
    pub duplicate_representation_count: usize,
    pub initial_probability_stddev: f32,
    pub final_probability_stddev: f32,
    pub root_cause: ProbabilityCollapseRootCauseV0,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumProbabilityCollapseForensicsV0 {
    pub window_id: String,
    pub diagnostic_status: ProbabilityCollapseDiagnosticStatusV0,
    pub root_cause: ProbabilityCollapseRootCauseV0,
    pub candidates: Vec<MomentumForensicCandidateV0>,
    pub selected_candidate: Option<MomentumForensicCandidateV0>,
    pub candidate_results: Vec<MomentumForensicCandidateResultV0>,
    pub test_partition_opened_once: bool,
    pub selected_checkpoint: Option<CheckpointRefV0>,
    pub representation_normalizer_digest: String,
    pub abstention: Option<ShadowLearningAbstentionV0>,
    pub temporal_generalization: Option<TemporalGeneralizationResultV0>,
}

/// Deterministic, sanitized campaign gates. These gates describe the boundary
/// between offline assessment and authorities that this campaign never receives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignSafetyGateV0 {
    ImmutableSanitizedEvidence,
    RealHistoricalEvidence,
    CanonicalSemanticDigest,
    ChronologicalEvidence,
    FiniteOhlcvValues,
    MinimumHistory,
    PurgedChronologicalWindows,
    CpuFullInferenceReady,
    FrozenEncoderCaptured,
    OfflineShadowLearning,
    PromotionEligibility,
    VotingEligibility,
    ExecutionEligibility,
    FrozenEncoderUnchanged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignSafetyGateOutcomeV0 {
    Passed,
    Rejected,
    Blocked,
    NotEvaluatedAfterEarlierRejection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CampaignSafetyGateEvaluationV0 {
    pub gate: CampaignSafetyGateV0,
    pub outcome: CampaignSafetyGateOutcomeV0,
    pub reason_code: Option<String>,
    /// Deliberately count/boolean-only facts; no local paths, identifiers, or values.
    pub sanitized_facts: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CampaignLayeredEligibilityV0 {
    pub offline_shadow_learning: bool,
    pub promotion: bool,
    pub voting: bool,
    pub execution: bool,
}

impl Default for CampaignLayeredEligibilityV0 {
    fn default() -> Self {
        Self {
            offline_shadow_learning: false,
            promotion: false,
            voting: false,
            execution: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CampaignSafetyTraceV0 {
    pub gates: Vec<CampaignSafetyGateEvaluationV0>,
    pub first_rejecting_gate: Option<CampaignSafetyGateV0>,
    pub first_reason_code: Option<String>,
    pub eligibility: CampaignLayeredEligibilityV0,
}

impl Default for CampaignSafetyTraceV0 {
    fn default() -> Self {
        Self {
            gates: Vec::new(),
            first_rejecting_gate: None,
            first_reason_code: None,
            eligibility: CampaignLayeredEligibilityV0::default(),
        }
    }
}

impl From<LearningError> for CampaignErrorV0 {
    fn from(_: LearningError) -> Self {
        Self::Learning
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AggregateMambaGateConfigV0 {
    pub minimum_windows: usize,
    pub comparison_epsilon: f32,
    pub minimum_win_fraction: f32,
    pub maximum_mean_degradation: f32,
    pub minimum_test_samples: usize,
}

impl Default for AggregateMambaGateConfigV0 {
    fn default() -> Self {
        Self {
            minimum_windows: 3,
            comparison_epsilon: 1e-4,
            minimum_win_fraction: 0.6,
            maximum_mean_degradation: 0.01,
            minimum_test_samples: 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelDriftConfigV0 {
    pub probability_stddev_floor: f32,
    pub mean_probability_shift_limit: f32,
    pub high_confidence_error_increase: usize,
    pub brier_degradation_limit: f32,
    pub head_weight_norm_change_limit: f32,
}

impl Default for ModelDriftConfigV0 {
    fn default() -> Self {
        Self {
            probability_stddev_floor: 0.01,
            mean_probability_shift_limit: 0.25,
            high_confidence_error_increase: 3,
            brier_degradation_limit: 0.02,
            head_weight_norm_change_limit: 2.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumLearningCampaignConfigV0 {
    pub campaign_id: String,
    pub agent_id: String,
    pub campaign_seed: u64,
    pub minimum_history_rows: usize,
    pub train_rows: usize,
    pub validation_rows: usize,
    pub test_rows: usize,
    pub step_rows: usize,
    pub purge_gap_rows: usize,
    pub minimum_test_samples: usize,
    pub minimum_evaluated_windows: usize,
    pub split_policy: CampaignSplitPolicyV0,
    pub initialization_policy: HeadInitializationPolicyV0,
    pub feature_config: MomentumFeatureConfigV0,
    pub sequence_config: MomentumSequenceConfigV0,
    pub training_config: HeadTrainingConfigV0,
    pub backend_preference: BackendPreference,
    pub fallback_policy: BackendFallbackPolicy,
    pub aggregate_gate: AggregateMambaGateConfigV0,
    pub drift_config: ModelDriftConfigV0,
    pub collapse_config: ProbabilityCollapseConfigV0,
    pub validation_signal_gate: ValidationSignalGateConfigV0,
    pub support_gate: ShadowSupportGateConfigV0,
}

impl Default for MomentumLearningCampaignConfigV0 {
    fn default() -> Self {
        let feature_config = MomentumFeatureConfigV0::default();
        let sequence_config = MomentumSequenceConfigV0::default();
        let required_gap = sequence_config.sequence_length - 1 + sequence_config.prediction_horizon;
        Self {
            campaign_id: "momentum-shadow-campaign".to_string(),
            agent_id: "momentum_mamba_shadow".to_string(),
            campaign_seed: 29,
            minimum_history_rows: 128,
            train_rows: 64,
            validation_rows: 24,
            test_rows: 24,
            step_rows: 24,
            purge_gap_rows: required_gap,
            minimum_test_samples: 4,
            minimum_evaluated_windows: 2,
            split_policy: CampaignSplitPolicyV0::ExpandingWindow,
            initialization_policy: HeadInitializationPolicyV0::CompareColdAndWarm,
            feature_config,
            sequence_config,
            training_config: HeadTrainingConfigV0::default(),
            backend_preference: BackendPreference::Auto,
            fallback_policy: BackendFallbackPolicy::AllowCpuFallback,
            aggregate_gate: AggregateMambaGateConfigV0::default(),
            drift_config: ModelDriftConfigV0::default(),
            collapse_config: ProbabilityCollapseConfigV0::default(),
            validation_signal_gate: ValidationSignalGateConfigV0::default(),
            support_gate: ShadowSupportGateConfigV0::default(),
        }
    }
}

impl MomentumLearningCampaignConfigV0 {
    pub fn required_purge_gap(&self) -> Result<usize, CampaignErrorV0> {
        self.sequence_config
            .sequence_length
            .checked_sub(1)
            .and_then(|value| value.checked_add(self.sequence_config.prediction_horizon))
            .ok_or(CampaignErrorV0::Overflow)
    }

    pub fn validate(&self) -> Result<(), CampaignErrorV0> {
        if self.campaign_id.trim().is_empty()
            || self.agent_id.trim().is_empty()
            || self.minimum_history_rows == 0
            || self.train_rows == 0
            || self.validation_rows == 0
            || self.test_rows == 0
            || self.step_rows == 0
            || self.minimum_test_samples == 0
            || self.minimum_evaluated_windows == 0
            || self.aggregate_gate.minimum_windows == 0
            || self.aggregate_gate.minimum_test_samples == 0
            || !self.aggregate_gate.comparison_epsilon.is_finite()
            || self.aggregate_gate.comparison_epsilon < 0.0
            || !self.aggregate_gate.minimum_win_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.aggregate_gate.minimum_win_fraction)
            || !self.aggregate_gate.maximum_mean_degradation.is_finite()
            || self.aggregate_gate.maximum_mean_degradation < 0.0
            || !self.drift_config.probability_stddev_floor.is_finite()
            || self.drift_config.probability_stddev_floor < 0.0
            || !self.drift_config.mean_probability_shift_limit.is_finite()
            || self.drift_config.mean_probability_shift_limit < 0.0
            || !self.drift_config.brier_degradation_limit.is_finite()
            || self.drift_config.brier_degradation_limit < 0.0
            || !self.drift_config.head_weight_norm_change_limit.is_finite()
            || self.drift_config.head_weight_norm_change_limit < 0.0
        {
            return Err(CampaignErrorV0::InvalidConfig);
        }
        self.feature_config
            .validate()
            .map_err(|_| CampaignErrorV0::InvalidConfig)?;
        self.sequence_config
            .validate()
            .map_err(|_| CampaignErrorV0::InvalidConfig)?;
        self.training_config
            .validate()
            .map_err(|_| CampaignErrorV0::InvalidConfig)?;
        self.collapse_config.validate()?;
        self.validation_signal_gate.validate()?;
        self.support_gate.validate()?;
        if self.split_policy == CampaignSplitPolicyV0::RollingWindow {
            return Err(CampaignErrorV0::UnsupportedSplitPolicy);
        }
        if self.purge_gap_rows < self.required_purge_gap()? {
            return Err(CampaignErrorV0::PurgeGapTooSmall);
        }
        self.train_rows
            .checked_add(self.purge_gap_rows)
            .and_then(|value| value.checked_add(self.validation_rows))
            .and_then(|value| value.checked_add(self.purge_gap_rows))
            .and_then(|value| value.checked_add(self.test_rows))
            .ok_or(CampaignErrorV0::Overflow)?;
        Ok(())
    }

    pub fn digest(&self) -> String {
        stable_hash_string(&format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}:{:?}:{}:{}:{}:{:?}:{:?}:{:?}:{:?}:{:?}",
            self.campaign_id,
            self.campaign_seed,
            self.minimum_history_rows,
            self.train_rows,
            self.validation_rows,
            self.test_rows,
            self.step_rows,
            self.purge_gap_rows,
            self.minimum_test_samples,
            self.minimum_evaluated_windows,
            self.split_policy,
            self.initialization_policy,
            self.feature_config.digest(),
            self.sequence_config.sequence_length,
            self.training_config.digest(),
            self.backend_preference,
            self.fallback_policy,
            self.collapse_config,
            self.validation_signal_gate,
            self.support_gate,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumLearningWindowV0 {
    pub window_id: String,
    pub train_range: IndexRangeV0,
    pub validation_range: IndexRangeV0,
    pub test_range: IndexRangeV0,
    pub train_sequence_range: IndexRangeV0,
    pub validation_sequence_range: IndexRangeV0,
    pub test_sequence_range: IndexRangeV0,
    pub snapshot_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PredictionDiagnosticsV0 {
    pub metrics: EvaluationMetricsV0,
    pub probability_stddev: f32,
    pub minimum_probability: f32,
    pub maximum_probability: f32,
    pub low_confidence_correct_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AggregateMambaValueEvidenceV0 {
    pub evaluated_windows: usize,
    pub sufficient_windows: usize,
    pub mamba_beats_constant_count: usize,
    pub mamba_beats_linear_count: usize,
    pub linear_beats_mamba_count: usize,
    pub mamba_ties_linear_count: usize,
    pub mean_brier_delta_vs_linear: f32,
    pub median_brier_delta_vs_linear: f32,
    pub high_confidence_error_delta_vs_linear: i64,
    pub status: MambaRepresentationValueStatusV0,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WarmStartValueStatusV0 {
    Helped,
    Failed,
    Mixed,
    InsufficientEvidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WarmStartValueEvidenceV0 {
    pub compared_windows: usize,
    pub warm_beats_cold_count: usize,
    pub cold_beats_warm_count: usize,
    pub tie_count: usize,
    pub mean_test_brier_delta: f32,
    pub mean_convergence_epoch_delta: f32,
    pub mean_parameter_drift_delta: f32,
    pub status: WarmStartValueStatusV0,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CampaignShadowAssessmentV0 {
    pub probability_up: f32,
    pub confidence: f32,
    pub suggested_action: CampaignShadowSuggestedActionV0,
    pub model_version_id: String,
    pub evidence_snapshot_ids: Vec<String>,
    pub backend: Mamba3BackendKind,
    pub mathematical_status: ModelMathematicalStatus,
    pub deployment_status: ModelAgentDeploymentStatus,
    pub eligible_to_vote: bool,
    pub eligible_to_execute: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RejectedLearningWindowV0 {
    pub window_id: String,
    pub path: Option<MomentumLearningPathV0>,
    pub reason: CampaignErrorV0,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumLearningPathResultV0 {
    pub path: MomentumLearningPathV0,
    pub parent_version_id: Option<String>,
    pub initial_head_digest: String,
    pub initial_head: LogisticPredictionHeadV0,
    pub initial_weight_norm: f32,
    pub initial_bias: f32,
    pub initial_probability_mean: f32,
    pub initial_probability_stddev: f32,
    pub training_prevalence: f32,
    pub final_head_digest: String,
    pub final_head: LogisticPredictionHeadV0,
    pub stopped_epoch: usize,
    pub train: PredictionDiagnosticsV0,
    pub validation: PredictionDiagnosticsV0,
    pub test: PredictionDiagnosticsV0,
    pub baselines: BaselineComparisonV0,
    pub frozen_linear_comparator: LinearMomentumBaselineV0,
    pub frozen_constant_comparator: ConstantProbabilityBaselineV0,
    pub version: SandboxModelVersionV0,
    pub shadow_assessment: CampaignShadowAssessmentV0,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumLearningWindowResultV0 {
    pub window: MomentumLearningWindowV0,
    pub normalizer_digest: String,
    pub feature_config_digest: String,
    pub feature_order: Vec<String>,
    pub paths: Vec<MomentumLearningPathResultV0>,
    pub drift_status: ModelDriftStatusV0,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumLearningCampaignResultV0 {
    pub campaign_id: String,
    pub status: MomentumLearningCampaignStatusV0,
    pub windows: Vec<MomentumLearningWindowResultV0>,
    pub aggregate_mamba_evidence: AggregateMambaValueEvidenceV0,
    pub warm_start_evidence: Option<WarmStartValueEvidenceV0>,
    pub aggregate_drift: ModelDriftStatusV0,
    pub generated_versions: Vec<SandboxModelVersionV0>,
    pub shadow_assessments: Vec<CampaignShadowAssessmentV0>,
    pub rejected_windows: Vec<RejectedLearningWindowV0>,
    pub reason_codes: Vec<String>,
    pub safety_trace: CampaignSafetyTraceV0,
    pub collapse_forensics: Vec<MomentumProbabilityCollapseForensicsV0>,
    pub validation_signal_gate: ValidationSignalGateConfigV0,
    pub support_gate: ShadowSupportGateConfigV0,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarmStartLockInStatusV0 {
    NoLockInEvidence,
    LockInSuspected,
    LockInConfirmed,
    WarmAndColdBothNoSignal,
    WarmBetter,
    ColdBetter,
    Mixed,
    InsufficientEvidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WarmColdTrajectoryComparisonV0 {
    pub window_id: String,
    pub cold_initial_head_digest: String,
    pub warm_initial_head_digest: String,
    pub warm_parent_version_id: Option<String>,
    pub initial_parameter_distance: f32,
    pub initial_bias_difference: f32,
    pub cold_initial_probability_mean: f32,
    pub warm_initial_probability_mean: f32,
    pub cold_initial_probability_stddev: f32,
    pub warm_initial_probability_stddev: f32,
    pub training_prevalence: f32,
    pub cold_stopped_epoch: usize,
    pub warm_stopped_epoch: usize,
    pub cold_validation_brier: f32,
    pub warm_validation_brier: f32,
    pub cold_validation_probability_stddev: f32,
    pub warm_validation_probability_stddev: f32,
    pub cold_won_validation: bool,
    pub warm_won_validation: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AggregateTemporalGeneralizationEvidenceV0 {
    pub total_windows: usize,
    pub no_signal_windows: usize,
    pub selected_checkpoint_windows: usize,
    pub support_gate_usable_windows: usize,
    pub validation_in_support_windows: usize,
    pub validation_out_of_support_windows: usize,
    pub validation_insufficient_windows: usize,
    pub validation_gate_unavailable_windows: usize,
    pub in_support_windows: usize,
    pub out_of_support_windows: usize,
    pub support_gate_unavailable_windows: usize,
    pub temporal_collapse_windows: usize,
    pub raw_feature_shift_windows: usize,
    pub normalized_feature_shift_windows: usize,
    pub sequence_shift_windows: usize,
    pub representation_shift_windows: usize,
    pub logit_shift_windows: usize,
    pub probability_shift_windows: usize,
    pub outcomes_only_windows: usize,
    pub warm_lock_in_windows: usize,
    pub operational_abstentions: usize,
    pub counterfactual_evaluations: usize,
    pub accepted_predictive_versions: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportGatedMomentumSeriesVerdictV0 {
    InSupportUsableSignalAndMambaHelpedOnThisSeries,
    InSupportUsableSignalButLinearStrongerOnThisSeries,
    InSupportMixedEvidence,
    TemporalOutOfSupportAbstention,
    FrozenRepresentationShiftRisk,
    WarmStartLockInRisk,
    NoUsableValidationSignal,
    InsufficientEvidence,
    CampaignFailed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumTemporalDiagnosticReportV0 {
    pub report_version: String,
    pub campaign_digest: String,
    pub evidence_row_count: usize,
    pub evidence_pack_digest_prefix: String,
    pub selected_window_id: Option<String>,
    pub selected_candidate: Option<MomentumForensicCandidateV0>,
    pub selected_checkpoint: Option<CheckpointRefV0>,
    pub raw_feature_shift: Option<DistributionShiftMetricBundleV0>,
    pub normalized_feature_shift: Option<DistributionShiftMetricBundleV0>,
    pub sequence_shift: Option<DistributionShiftMetricBundleV0>,
    pub frozen_representation_shift: Option<DistributionShiftMetricBundleV0>,
    pub representation_scale_shift: Option<DistributionShiftMetricBundleV0>,
    pub logit_shift: Option<DistributionShiftMetricBundleV0>,
    pub probability_shift: Option<DistributionShiftMetricBundleV0>,
    pub outcome_shift: Option<DistributionShiftMetricBundleV0>,
    pub validation_support_decision: ShadowSupportDecisionV0,
    pub test_support_decision: ShadowSupportDecisionV0,
    pub validation_support_coverage: Option<f32>,
    pub support_decision_digest: Option<String>,
    pub earliest_shift_stage: EarliestTemporalShiftStageV0,
    pub temporal_root_cause: ProbabilityCollapseRootCauseV0,
    pub warm_start_status: WarmStartLockInStatusV0,
    pub warm_cold_comparisons: Vec<WarmColdTrajectoryComparisonV0>,
    pub aggregate: AggregateTemporalGeneralizationEvidenceV0,
    pub final_verdict: SupportGatedMomentumSeriesVerdictV0,
    pub operational_result: String,
    pub counterfactual_result: String,
    pub layered_eligibility: CampaignLayeredEligibilityV0,
    pub reason_codes: Vec<String>,
    pub report_digest: String,
}

#[derive(Clone)]
struct ValidatedEvidence {
    candles: Vec<MomentumCandleV0>,
    snapshot_ids: Vec<String>,
}

#[derive(Clone)]
struct WarmParent {
    window_id: String,
    version: SandboxModelVersionV0,
    head: LogisticPredictionHeadV0,
}

struct TemporalFeaturePartitionsV0 {
    raw_train: Vec<Vec<f32>>,
    raw_test: Vec<Vec<f32>>,
    normalized_train: Vec<Vec<f32>>,
    normalized_test: Vec<Vec<f32>>,
}

pub fn build_momentum_learning_windows_v0(
    config: &MomentumLearningCampaignConfigV0,
    row_count: usize,
    snapshot_ids: &[String],
) -> Result<Vec<MomentumLearningWindowV0>, CampaignErrorV0> {
    config.validate()?;
    if row_count < config.minimum_history_rows {
        return Err(CampaignErrorV0::InsufficientHistory);
    }
    let mut snapshot_ids = snapshot_ids.to_vec();
    snapshot_ids.sort();
    snapshot_ids.dedup();
    if snapshot_ids.is_empty() {
        return Err(CampaignErrorV0::UnsafeEvidence);
    }
    let mut windows = Vec::new();
    let mut train_end = config.train_rows;
    loop {
        let validation_start = train_end
            .checked_add(config.purge_gap_rows)
            .ok_or(CampaignErrorV0::Overflow)?;
        let validation_end = validation_start
            .checked_add(config.validation_rows)
            .ok_or(CampaignErrorV0::Overflow)?;
        let test_start = validation_end
            .checked_add(config.purge_gap_rows)
            .ok_or(CampaignErrorV0::Overflow)?;
        let test_end = test_start
            .checked_add(config.test_rows)
            .ok_or(CampaignErrorV0::Overflow)?;
        if test_end > row_count {
            break;
        }
        let window_id = format!(
            "window-{}",
            stable_hash_string(&format!(
                "{}:{}:{}:{}:{}",
                config.campaign_id, train_end, validation_start, validation_end, test_end
            ))
        );
        windows.push(MomentumLearningWindowV0 {
            window_id,
            train_range: IndexRangeV0 {
                start: 0,
                end: train_end,
            },
            validation_range: IndexRangeV0 {
                start: validation_start,
                end: validation_end,
            },
            test_range: IndexRangeV0 {
                start: test_start,
                end: test_end,
            },
            train_sequence_range: IndexRangeV0 {
                start: 0,
                end: train_end,
            },
            validation_sequence_range: IndexRangeV0 {
                start: validation_start,
                end: validation_end,
            },
            test_sequence_range: IndexRangeV0 {
                start: test_start,
                end: test_end,
            },
            snapshot_ids: snapshot_ids.clone(),
        });
        train_end = train_end
            .checked_add(config.step_rows)
            .ok_or(CampaignErrorV0::Overflow)?;
    }
    if windows.is_empty() {
        return Err(CampaignErrorV0::InsufficientHistory);
    }
    Ok(windows)
}

impl RepresentationNormalizerV0 {
    pub fn fit(examples: &[EncodedTrainingExampleV0]) -> Result<Self, CampaignErrorV0> {
        let dimension = examples
            .first()
            .map(|example| example.representation.len())
            .filter(|dimension| *dimension > 0)
            .ok_or(CampaignErrorV0::InsufficientHistory)?;
        if examples.iter().any(|example| {
            example.representation.len() != dimension
                || example
                    .representation
                    .iter()
                    .any(|value| !value.is_finite())
        }) {
            return Err(CampaignErrorV0::Learning);
        }
        let mut means = Vec::with_capacity(dimension);
        let mut scales = Vec::with_capacity(dimension);
        let mut constant_dimension_indices = Vec::new();
        for index in 0..dimension {
            let values = examples
                .iter()
                .map(|example| example.representation[index])
                .collect::<Vec<_>>();
            let mean = mean_f32(&values)?;
            let scale = stddev_f32(&values, mean)?;
            means.push(mean);
            if scale <= 1e-6 {
                scales.push(1.0);
                constant_dimension_indices.push(index);
            } else {
                scales.push(scale);
            }
        }
        Ok(Self {
            means,
            scales,
            constant_dimension_indices,
        })
    }

    pub fn transform(
        &self,
        examples: &[EncodedTrainingExampleV0],
    ) -> Result<Vec<EncodedTrainingExampleV0>, CampaignErrorV0> {
        if self.means.is_empty()
            || self.means.len() != self.scales.len()
            || self
                .means
                .iter()
                .chain(&self.scales)
                .any(|value| !value.is_finite())
            || self.scales.iter().any(|value| *value <= 0.0)
        {
            return Err(CampaignErrorV0::Learning);
        }
        examples
            .iter()
            .map(|example| {
                if example.representation.len() != self.means.len() {
                    return Err(CampaignErrorV0::Learning);
                }
                let representation = example
                    .representation
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (value - self.means[index]) / self.scales[index])
                    .collect::<Vec<_>>();
                if representation.iter().any(|value| !value.is_finite()) {
                    return Err(CampaignErrorV0::Learning);
                }
                Ok(EncodedTrainingExampleV0 {
                    representation,
                    label: example.label,
                    snapshot_ids: example.snapshot_ids.clone(),
                })
            })
            .collect()
    }

    pub fn transform_representation(
        &self,
        representation: &[f32],
    ) -> Result<Vec<f32>, CampaignErrorV0> {
        if self.means.is_empty()
            || self.means.len() != self.scales.len()
            || representation.len() != self.means.len()
            || self
                .means
                .iter()
                .chain(&self.scales)
                .chain(representation)
                .any(|value| !value.is_finite())
            || self.scales.iter().any(|value| *value <= 0.0)
        {
            return Err(CampaignErrorV0::Learning);
        }
        let transformed = representation
            .iter()
            .enumerate()
            .map(|(index, value)| (value - self.means[index]) / self.scales[index])
            .collect::<Vec<_>>();
        if transformed.iter().any(|value| !value.is_finite()) {
            return Err(CampaignErrorV0::Learning);
        }
        Ok(transformed)
    }

    pub fn digest(&self) -> String {
        stable_hash_string(&format!(
            "{}:{}:{}",
            self.means
                .iter()
                .map(|value| value.to_bits().to_string())
                .collect::<Vec<_>>()
                .join(","),
            self.scales
                .iter()
                .map(|value| value.to_bits().to_string())
                .collect::<Vec<_>>()
                .join(","),
            self.constant_dimension_indices
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(","),
        ))
    }
}

pub fn probability_collapse_metrics_v0(
    probabilities: &[f32],
    labels: &[f32],
    config: &ProbabilityCollapseConfigV0,
) -> Result<ProbabilityCollapseMetricsV0, CampaignErrorV0> {
    config.validate()?;
    if probabilities.len() != labels.len() || probabilities.len() < config.minimum_samples {
        return Err(CampaignErrorV0::InsufficientHistory);
    }
    if probabilities
        .iter()
        .chain(labels)
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(CampaignErrorV0::Learning);
    }
    let mean_probability = mean_f32(probabilities)?;
    let probability_stddev = stddev_f32(probabilities, mean_probability)?;
    let minimum_probability = probabilities.iter().copied().fold(f32::INFINITY, f32::min);
    let maximum_probability = probabilities
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let mean_entropy = probabilities
        .iter()
        .map(|value| {
            let value = value.clamp(1e-6, 1.0 - 1e-6);
            -value * value.ln() - (1.0 - value) * (1.0 - value).ln()
        })
        .sum::<f32>()
        / probabilities.len() as f32;
    let mut occupied_bins = [false; 10];
    for probability in probabilities {
        occupied_bins[((*probability * 10.0).floor() as usize).min(9)] = true;
    }
    let unique_probability_bins = occupied_bins.into_iter().filter(|value| *value).count();
    let low_count = probabilities
        .iter()
        .filter(|value| **value <= config.low_saturation_threshold)
        .count();
    let high_count = probabilities
        .iter()
        .filter(|value| **value >= config.high_saturation_threshold)
        .count();
    let positive_count = probabilities.iter().filter(|value| **value >= 0.5).count();
    let count = probabilities.len() as f32;
    let low_saturation_fraction = low_count as f32 / count;
    let high_saturation_fraction = high_count as f32 / count;
    let positive_prediction_fraction = positive_count as f32 / count;
    let saturation_fraction = low_saturation_fraction + high_saturation_fraction;
    let high_confidence_error_count = probabilities
        .iter()
        .zip(labels)
        .filter(|(probability, label)| {
            (**probability >= 0.8 && **label < 0.5) || (**probability <= 0.2 && **label >= 0.5)
        })
        .count();
    let mut subtypes = Vec::new();
    if probability_stddev < config.minimum_probability_stddev {
        subtypes.push(ProbabilityCollapseSubtypeV0::NearConstantProbability);
    }
    if low_saturation_fraction >= config.maximum_saturation_fraction {
        subtypes.push(ProbabilityCollapseSubtypeV0::NearZeroProbability);
    }
    if high_saturation_fraction >= config.maximum_saturation_fraction {
        subtypes.push(ProbabilityCollapseSubtypeV0::NearOneProbability);
    }
    if positive_prediction_fraction >= config.maximum_single_side_fraction
        || positive_prediction_fraction <= 1.0 - config.maximum_single_side_fraction
    {
        subtypes.push(ProbabilityCollapseSubtypeV0::SingleSidePrediction);
    }
    if saturation_fraction >= config.maximum_saturation_fraction {
        subtypes.push(ProbabilityCollapseSubtypeV0::SaturatedProbability);
    }
    if mean_entropy < config.minimum_prediction_entropy {
        subtypes.push(ProbabilityCollapseSubtypeV0::LowEntropyPrediction);
    }
    if unique_probability_bins < config.minimum_unique_probability_bins {
        subtypes.push(ProbabilityCollapseSubtypeV0::InsufficientUniquePredictions);
    }
    Ok(ProbabilityCollapseMetricsV0 {
        sample_count: probabilities.len(),
        mean_probability,
        probability_stddev,
        minimum_probability,
        maximum_probability,
        mean_entropy,
        unique_probability_bins,
        low_saturation_fraction,
        high_saturation_fraction,
        positive_prediction_fraction,
        saturation_fraction,
        high_confidence_error_count,
        subtypes,
    })
}

pub fn brier_decomposition_v0(
    probabilities: &[f32],
    labels: &[f32],
    bin_count: usize,
) -> Result<BrierDecompositionV0, CampaignErrorV0> {
    if probabilities.is_empty() || probabilities.len() != labels.len() || bin_count < 2 {
        return Err(CampaignErrorV0::InvalidConfig);
    }
    if probabilities
        .iter()
        .chain(labels)
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(CampaignErrorV0::Learning);
    }
    let prevalence = labels.iter().sum::<f32>() / labels.len() as f32;
    let mut reliability = 0.0;
    let mut resolution = 0.0;
    let mut occupied = 0;
    for bin in 0..bin_count {
        let rows = probabilities
            .iter()
            .zip(labels)
            .filter(|(p, _)| {
                let index = ((**p * bin_count as f32).floor() as usize).min(bin_count - 1);
                index == bin
            })
            .collect::<Vec<_>>();
        if rows.is_empty() {
            continue;
        }
        occupied += 1;
        let count = rows.len() as f32;
        let mean_p = rows.iter().map(|(p, _)| **p).sum::<f32>() / count;
        let mean_y = rows.iter().map(|(_, y)| **y).sum::<f32>() / count;
        reliability += count / labels.len() as f32 * (mean_p - mean_y).powi(2);
        resolution += count / labels.len() as f32 * (mean_y - prevalence).powi(2);
    }
    let uncertainty = prevalence * (1.0 - prevalence);
    let brier_score = probabilities
        .iter()
        .zip(labels)
        .map(|(p, y)| (*p - *y).powi(2))
        .sum::<f32>()
        / labels.len() as f32;
    Ok(BrierDecompositionV0 {
        brier_score,
        reliability,
        resolution,
        uncertainty,
        bin_count,
        occupied_bin_count: occupied,
        sample_count: labels.len(),
    })
}

pub fn binary_rank_auc_v0(
    probabilities: &[f32],
    labels: &[f32],
) -> Result<BinaryRankAucV0, CampaignErrorV0> {
    if probabilities.len() != labels.len() || probabilities.is_empty() {
        return Err(CampaignErrorV0::InsufficientHistory);
    }
    if probabilities
        .iter()
        .chain(labels)
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(CampaignErrorV0::Learning);
    }
    let positive_count = labels.iter().filter(|value| **value >= 0.5).count();
    let negative_count = labels.len() - positive_count;
    if positive_count == 0 || negative_count == 0 {
        return Ok(BinaryRankAucV0 {
            status: BinaryRankAucStatusV0::UndefinedSingleClass,
            value: None,
            positive_count,
            negative_count,
            tie_count: 0,
        });
    }
    let mut values = probabilities
        .iter()
        .copied()
        .zip(labels.iter().copied())
        .collect::<Vec<_>>();
    values.sort_by(|left, right| left.0.total_cmp(&right.0));
    let mut positive_rank_sum = 0.0;
    let mut ties = 0;
    let mut start = 0;
    while start < values.len() {
        let mut end = start + 1;
        while end < values.len() && values[end].0 == values[start].0 {
            end += 1;
        }
        let rank = (start as f32 + 1.0 + end as f32) / 2.0;
        positive_rank_sum +=
            values[start..end].iter().filter(|(_, y)| *y >= 0.5).count() as f32 * rank;
        ties += (end - start).saturating_sub(1);
        start = end;
    }
    let auc = (positive_rank_sum - positive_count as f32 * (positive_count as f32 + 1.0) / 2.0)
        / (positive_count * negative_count) as f32;
    Ok(BinaryRankAucV0 {
        status: BinaryRankAucStatusV0::Defined,
        value: Some(auc),
        positive_count,
        negative_count,
        tie_count: ties,
    })
}

fn forecast_metric_bundle(
    head: &LogisticPredictionHeadV0,
    examples: &[EncodedTrainingExampleV0],
    collapse: &ProbabilityCollapseConfigV0,
    gate: &ValidationSignalGateConfigV0,
) -> Result<ForecastMetricBundleV0, CampaignErrorV0> {
    let probabilities = probabilities_for_head(head, examples)?;
    let labels = examples
        .iter()
        .map(|example| example.label)
        .collect::<Vec<_>>();
    let evaluation = evaluate_head_v0(head, examples)?;
    let collapse = probability_collapse_metrics_v0(&probabilities, &labels, collapse)?;
    let brier = brier_decomposition_v0(&probabilities, &labels, gate.brier_bin_count)?;
    let rank_auc = binary_rank_auc_v0(&probabilities, &labels)?;
    let finite = [
        evaluation.brier_score,
        brier.reliability,
        brier.resolution,
        brier.uncertainty,
    ]
    .iter()
    .all(|value| value.is_finite());
    Ok(ForecastMetricBundleV0 {
        evaluation,
        collapse,
        brier,
        rank_auc,
        finite,
    })
}

impl DistributionSupportEnvelopeV0 {
    pub fn fit(
        rows: &[Vec<f32>],
        gate: &ShadowSupportGateConfigV0,
    ) -> Result<Self, CampaignErrorV0> {
        gate.validate()?;
        let width = rows
            .first()
            .map(Vec::len)
            .filter(|width| *width > 0)
            .ok_or(CampaignErrorV0::InsufficientHistory)?;
        if rows.len() < gate.minimum_samples
            || rows
                .iter()
                .any(|row| row.len() != width || row.iter().any(|value| !value.is_finite()))
        {
            return Err(CampaignErrorV0::Learning);
        }
        let means = (0..width)
            .map(|index| rows.iter().map(|row| row[index]).sum::<f32>() / rows.len() as f32)
            .collect::<Vec<_>>();
        let scales = (0..width)
            .map(|index| {
                (rows
                    .iter()
                    .map(|row| (row[index] - means[index]).powi(2))
                    .sum::<f32>()
                    / rows.len() as f32)
                    .sqrt()
                    .max(gate.comparison_epsilon)
            })
            .collect::<Vec<_>>();
        let digest = stable_hash_string(&format!(
            "{:?}:{:?}:{:.6}",
            means, scales, gate.maximum_dimension_standardized_shift
        ));
        Ok(Self {
            means,
            scales,
            lower_z_limit: -gate.maximum_dimension_standardized_shift,
            upper_z_limit: gate.maximum_dimension_standardized_shift,
            maximum_out_of_support_fraction: gate.maximum_out_of_support_fraction,
            epsilon: gate.comparison_epsilon,
            digest,
        })
    }
}

pub fn distribution_shift_metrics_v0(
    reference: &[Vec<f32>],
    target: &[Vec<f32>],
    envelope: &DistributionSupportEnvelopeV0,
) -> Result<DistributionShiftMetricBundleV0, CampaignErrorV0> {
    let width = envelope.means.len();
    if width == 0
        || envelope.scales.len() != width
        || reference.len() < 1
        || target.len() < 1
        || reference
            .iter()
            .chain(target)
            .any(|row| row.len() != width || row.iter().any(|value| !value.is_finite()))
    {
        return Err(CampaignErrorV0::Learning);
    }
    let mut mean_shift = Vec::with_capacity(width);
    let mut variance_shift = Vec::with_capacity(width);
    let mut breaches = 0usize;
    let mut out = 0usize;
    for index in 0..width {
        let reference_mean =
            reference.iter().map(|row| row[index]).sum::<f32>() / reference.len() as f32;
        let target_mean = target.iter().map(|row| row[index]).sum::<f32>() / target.len() as f32;
        let reference_variance = reference
            .iter()
            .map(|row| (row[index] - reference_mean).powi(2))
            .sum::<f32>()
            / reference.len() as f32;
        let target_variance = target
            .iter()
            .map(|row| (row[index] - target_mean).powi(2))
            .sum::<f32>()
            / target.len() as f32;
        mean_shift.push(
            ((target_mean - reference_mean) / reference_variance.sqrt().max(envelope.epsilon))
                .abs(),
        );
        variance_shift.push(
            (target_variance.max(envelope.epsilon) / reference_variance.max(envelope.epsilon))
                .ln()
                .abs(),
        );
        if target.iter().any(|row| {
            let z =
                (row[index] - envelope.means[index]) / envelope.scales[index].max(envelope.epsilon);
            z < envelope.lower_z_limit || z > envelope.upper_z_limit
        }) {
            breaches += 1;
        }
    }
    for row in target {
        for index in 0..width {
            let z =
                (row[index] - envelope.means[index]) / envelope.scales[index].max(envelope.epsilon);
            out += usize::from(z < envelope.lower_z_limit || z > envelope.upper_z_limit);
        }
    }
    Ok(DistributionShiftMetricBundleV0 {
        sample_count_reference: reference.len(),
        sample_count_target: target.len(),
        dimensions: width,
        mean_absolute_standardized_mean_shift: mean_shift.iter().sum::<f32>() / width as f32,
        maximum_absolute_standardized_mean_shift: mean_shift.iter().copied().fold(0.0, f32::max),
        mean_absolute_log_variance_ratio: variance_shift.iter().sum::<f32>() / width as f32,
        maximum_absolute_log_variance_ratio: variance_shift.iter().copied().fold(0.0, f32::max),
        out_of_support_fraction: out as f32 / (target.len() * width) as f32,
        dimensions_out_of_support: breaches,
        finite: true,
    })
}

fn support_decision(
    metrics: &DistributionShiftMetricBundleV0,
    gate: &ShadowSupportGateConfigV0,
    validation: bool,
) -> ShadowSupportDecisionV0 {
    if !metrics.finite {
        ShadowSupportDecisionV0::NumericalFailure
    } else if metrics.sample_count_reference < gate.minimum_samples
        || metrics.sample_count_target < gate.minimum_samples
    {
        ShadowSupportDecisionV0::InsufficientEvidence
    } else if validation && 1.0 - metrics.out_of_support_fraction < gate.minimum_validation_coverage
    {
        ShadowSupportDecisionV0::OutOfSupport
    } else if metrics.mean_absolute_standardized_mean_shift > gate.maximum_mean_standardized_shift
        || metrics.maximum_absolute_standardized_mean_shift
            > gate.maximum_dimension_standardized_shift
        || metrics.mean_absolute_log_variance_ratio > gate.maximum_mean_log_variance_ratio
        || metrics.out_of_support_fraction > gate.maximum_out_of_support_fraction
    {
        ShadowSupportDecisionV0::OutOfSupport
    } else {
        ShadowSupportDecisionV0::InSupport
    }
}

fn shadow_support_decision_label_v0(decision: ShadowSupportDecisionV0) -> &'static str {
    match decision {
        ShadowSupportDecisionV0::InSupport => "in_support",
        ShadowSupportDecisionV0::OutOfSupport => "out_of_support",
        ShadowSupportDecisionV0::SupportGateUnavailable => "support_gate_unavailable",
        ShadowSupportDecisionV0::InsufficientEvidence => "insufficient_evidence",
        ShadowSupportDecisionV0::NumericalFailure => "numerical_failure",
        ShadowSupportDecisionV0::NotEvaluated => "not_evaluated",
    }
}

fn support_gate_applicability_v0(
    metrics: &DistributionShiftMetricBundleV0,
    gate: &ShadowSupportGateConfigV0,
) -> SupportGateApplicabilityStatusV0 {
    if !metrics.finite {
        SupportGateApplicabilityStatusV0::NumericalFailure
    } else if metrics.dimensions == 0
        || metrics.sample_count_reference == 0
        || metrics.sample_count_target == 0
    {
        SupportGateApplicabilityStatusV0::UnsupportedRepresentationShape
    } else if metrics.sample_count_reference < gate.minimum_samples
        || metrics.sample_count_target < gate.minimum_samples
    {
        SupportGateApplicabilityStatusV0::InsufficientValidationSamples
    } else {
        SupportGateApplicabilityStatusV0::Applicable
    }
}

fn support_metric_evaluations_v0(
    metrics: &DistributionShiftMetricBundleV0,
    gate: &ShadowSupportGateConfigV0,
) -> Vec<SupportMetricEvaluationV0> {
    let lower = |metric_id, measured_value, threshold| SupportMetricEvaluationV0 {
        metric_id,
        measured_value: Some(measured_value),
        configured_threshold: Some(threshold),
        decision: if measured_value >= threshold {
            SupportMetricDecisionV0::Passed
        } else {
            SupportMetricDecisionV0::Breached
        },
        required: true,
    };
    let upper = |metric_id, measured_value, threshold| SupportMetricEvaluationV0 {
        metric_id,
        measured_value: Some(measured_value),
        configured_threshold: Some(threshold),
        decision: if measured_value <= threshold {
            SupportMetricDecisionV0::Passed
        } else {
            SupportMetricDecisionV0::Breached
        },
        required: true,
    };
    vec![
        lower(
            SupportMetricIdV0::ValidationSampleCount,
            metrics.sample_count_target as f32,
            gate.minimum_samples as f32,
        ),
        lower(
            SupportMetricIdV0::ValidationCoverage,
            1.0 - metrics.out_of_support_fraction,
            gate.minimum_validation_coverage,
        ),
        upper(
            SupportMetricIdV0::MeanStandardizedShift,
            metrics.mean_absolute_standardized_mean_shift,
            gate.maximum_mean_standardized_shift,
        ),
        upper(
            SupportMetricIdV0::MaximumStandardizedShift,
            metrics.maximum_absolute_standardized_mean_shift,
            gate.maximum_dimension_standardized_shift,
        ),
        upper(
            SupportMetricIdV0::MeanLogVarianceRatio,
            metrics.mean_absolute_log_variance_ratio,
            gate.maximum_mean_log_variance_ratio,
        ),
        upper(
            SupportMetricIdV0::OutOfSupportFraction,
            metrics.out_of_support_fraction,
            gate.maximum_out_of_support_fraction,
        ),
    ]
}

fn train_history_support_audit_v0(
    rows: &[Vec<f32>],
    gate: &ShadowSupportGateConfigV0,
) -> TrainHistorySupportAuditV0 {
    let mut split_points = [
        rows.len() / 2,
        rows.len().saturating_sub(gate.minimum_samples),
    ]
    .into_iter()
    .filter(|split| *split >= gate.minimum_samples && rows.len() - *split >= gate.minimum_samples)
    .collect::<Vec<_>>();
    split_points.sort_unstable();
    split_points.dedup();

    let mut in_support_fold_count = 0usize;
    let mut out_of_support_fold_count = 0usize;
    let mut insufficient_evidence_fold_count = 0usize;
    let mut unavailable_fold_count = 0usize;
    let mut first_breach_metric = None;
    for split in &split_points {
        let reference = &rows[..*split];
        let target = &rows[*split..];
        let decision = DistributionSupportEnvelopeV0::fit(reference, gate)
            .and_then(|envelope| distribution_shift_metrics_v0(reference, target, &envelope))
            .map(|metrics| {
                if first_breach_metric.is_none() {
                    first_breach_metric = support_metric_evaluations_v0(&metrics, gate)
                        .into_iter()
                        .find(|metric| metric.decision == SupportMetricDecisionV0::Breached)
                        .map(|metric| metric.metric_id);
                }
                support_decision(&metrics, gate, false)
            });
        match decision {
            Ok(ShadowSupportDecisionV0::InSupport) => in_support_fold_count += 1,
            Ok(ShadowSupportDecisionV0::OutOfSupport) => out_of_support_fold_count += 1,
            Ok(ShadowSupportDecisionV0::InsufficientEvidence) => {
                insufficient_evidence_fold_count += 1
            }
            Ok(_) | Err(_) => unavailable_fold_count += 1,
        }
    }
    let status = if split_points.is_empty() {
        TrainHistorySupportAuditStatusV0::InsufficientAuditEvidence
    } else if unavailable_fold_count > 0 {
        TrainHistorySupportAuditStatusV0::NumericalFailure
    } else if out_of_support_fold_count == 0 {
        TrainHistorySupportAuditStatusV0::SelfConsistent
    } else if out_of_support_fold_count * 2 >= split_points.len() {
        TrainHistorySupportAuditStatusV0::OverRejectingOnTrainingHistory
    } else {
        TrainHistorySupportAuditStatusV0::ChronologicallyNonstationary
    };
    let digest = stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{:?}:{:?}",
        split_points.len(),
        in_support_fold_count,
        out_of_support_fold_count,
        insufficient_evidence_fold_count,
        unavailable_fold_count,
        first_breach_metric,
        status,
    ));
    TrainHistorySupportAuditV0 {
        fixed_chronological_fold_count: split_points.len(),
        in_support_fold_count,
        out_of_support_fold_count,
        insufficient_evidence_fold_count,
        unavailable_fold_count,
        first_breach_metric,
        status,
        digest,
    }
}

pub fn momentum_support_traces_v0(
    campaign: &MomentumLearningCampaignResultV0,
) -> Vec<MomentumSupportTraceV0> {
    let mut traces = campaign
        .collapse_forensics
        .iter()
        .filter_map(|forensics| {
            let temporal = forensics.temporal_generalization.as_ref()?;
            let checkpoint = forensics.selected_checkpoint.as_ref()?;
            let metrics = support_metric_evaluations_v0(
                &temporal.validation_representation_shift,
                &campaign.support_gate,
            );
            let first_breach_metric = metrics
                .iter()
                .find(|metric| metric.decision == SupportMetricDecisionV0::Breached)
                .map(|metric| metric.metric_id);
            let breached_metric_count = metrics
                .iter()
                .filter(|metric| metric.decision == SupportMetricDecisionV0::Breached)
                .count();
            let applicability = support_gate_applicability_v0(
                &temporal.validation_representation_shift,
                &campaign.support_gate,
            );
            let result_digest = stable_hash_string(&format!(
                "{:?}:{:?}:{:?}:{}:{}",
                applicability,
                temporal.validation_support_decision,
                first_breach_metric,
                breached_metric_count,
                temporal.support_envelope_digest,
            ));
            Some(MomentumSupportTraceV0 {
                envelope: SupportEnvelopeTraceV0 {
                    window_id: forensics.window_id.clone(),
                    candidate_id: format!("{:?}", checkpoint.candidate),
                    checkpoint_epoch: checkpoint.epoch,
                    construction_status: SupportEnvelopeConstructionStatusV0::Ready,
                    sample_count: temporal
                        .validation_representation_shift
                        .sample_count_reference,
                    dimension_count: temporal.validation_representation_shift.dimensions,
                    means_finite: temporal.validation_representation_shift.finite,
                    scales_finite: temporal.validation_representation_shift.finite,
                    constant_dimension_count: temporal.support_envelope_constant_dimension_count,
                    digest: temporal.support_envelope_digest.clone(),
                },
                metrics,
                validation: ValidationSupportResultV0 {
                    gate_applicability: applicability,
                    support_decision: temporal.validation_support_decision,
                    first_breach_metric,
                    breached_metric_count,
                    missing_required_metric_count: 0,
                    missing_optional_metric_count: 0,
                    result_digest,
                },
                train_history_audit: temporal.train_history_support_audit.clone(),
                test_support_decision: temporal.test_support_decision,
            })
        })
        .collect::<Vec<_>>();
    traces.sort_by(|left, right| left.envelope.window_id.cmp(&right.envelope.window_id));
    traces
}

fn flatten_sequence_inputs_v0(examples: &[SequenceExampleV0]) -> Vec<Vec<f32>> {
    examples
        .iter()
        .map(|example| example.input.iter().flatten().copied().collect())
        .collect()
}

fn distribution_shift_for_rows_v0(
    reference: &[Vec<f32>],
    target: &[Vec<f32>],
    gate: &ShadowSupportGateConfigV0,
) -> Result<DistributionShiftMetricBundleV0, CampaignErrorV0> {
    let envelope = DistributionSupportEnvelopeV0::fit(reference, gate)?;
    distribution_shift_metrics_v0(reference, target, &envelope)
}

fn distribution_shift_for_scalars_v0(
    reference: &[f32],
    target: &[f32],
    gate: &ShadowSupportGateConfigV0,
) -> Result<DistributionShiftMetricBundleV0, CampaignErrorV0> {
    let reference = reference
        .iter()
        .map(|value| vec![*value])
        .collect::<Vec<_>>();
    let target = target.iter().map(|value| vec![*value]).collect::<Vec<_>>();
    distribution_shift_for_rows_v0(&reference, &target, gate)
}

fn head_logits_v0(
    head: &LogisticPredictionHeadV0,
    examples: &[EncodedTrainingExampleV0],
) -> Result<Vec<f32>, CampaignErrorV0> {
    head.validate()?;
    examples
        .iter()
        .map(|example| {
            if example.representation.len() != head.weights.len()
                || example
                    .representation
                    .iter()
                    .any(|value| !value.is_finite())
            {
                return Err(CampaignErrorV0::Learning);
            }
            let logit = head.bias
                + head
                    .weights
                    .iter()
                    .zip(&example.representation)
                    .map(|(weight, value)| weight * value)
                    .sum::<f32>();
            if logit.is_finite() {
                Ok(logit)
            } else {
                Err(CampaignErrorV0::Learning)
            }
        })
        .collect()
}

fn has_material_distribution_shift_v0(
    metrics: &DistributionShiftMetricBundleV0,
    gate: &ShadowSupportGateConfigV0,
) -> bool {
    !metrics.finite
        || metrics.mean_absolute_standardized_mean_shift > gate.maximum_mean_standardized_shift
        || metrics.maximum_absolute_standardized_mean_shift
            > gate.maximum_dimension_standardized_shift
        || metrics.mean_absolute_log_variance_ratio > gate.maximum_mean_log_variance_ratio
        || metrics.out_of_support_fraction > gate.maximum_out_of_support_fraction
}

fn earliest_temporal_shift_stage_v0(
    raw: Option<&DistributionShiftMetricBundleV0>,
    normalized: Option<&DistributionShiftMetricBundleV0>,
    sequence: &DistributionShiftMetricBundleV0,
    frozen_representation: &DistributionShiftMetricBundleV0,
    representation_scale: &DistributionShiftMetricBundleV0,
    logits: &DistributionShiftMetricBundleV0,
    probabilities: &DistributionShiftMetricBundleV0,
    outcomes: &DistributionShiftMetricBundleV0,
    gate: &ShadowSupportGateConfigV0,
) -> EarliestTemporalShiftStageV0 {
    let stages = [
        (raw, EarliestTemporalShiftStageV0::RawFeatures),
        (normalized, EarliestTemporalShiftStageV0::NormalizedFeatures),
    ];
    if let Some((_, stage)) = stages.into_iter().find(|(metrics, _)| {
        metrics.is_some_and(|metrics| has_material_distribution_shift_v0(metrics, gate))
    }) {
        return stage;
    }
    [
        (sequence, EarliestTemporalShiftStageV0::Sequences),
        (
            frozen_representation,
            EarliestTemporalShiftStageV0::FrozenRepresentations,
        ),
        (
            representation_scale,
            EarliestTemporalShiftStageV0::RepresentationScale,
        ),
        (logits, EarliestTemporalShiftStageV0::Logits),
        (probabilities, EarliestTemporalShiftStageV0::Probabilities),
        (outcomes, EarliestTemporalShiftStageV0::OutcomesOnly),
    ]
    .into_iter()
    .find(|(metrics, _)| has_material_distribution_shift_v0(metrics, gate))
    .map(|(_, stage)| stage)
    .unwrap_or(EarliestTemporalShiftStageV0::None)
}

fn temporal_shift_status_v0(
    stage: EarliestTemporalShiftStageV0,
) -> TemporalDistributionShiftStatusV0 {
    match stage {
        EarliestTemporalShiftStageV0::None => TemporalDistributionShiftStatusV0::Stable,
        EarliestTemporalShiftStageV0::RawFeatures
        | EarliestTemporalShiftStageV0::NormalizedFeatures => {
            TemporalDistributionShiftStatusV0::NormalizedFeatureShift
        }
        EarliestTemporalShiftStageV0::Sequences
        | EarliestTemporalShiftStageV0::FrozenRepresentations
        | EarliestTemporalShiftStageV0::RepresentationScale => {
            TemporalDistributionShiftStatusV0::FrozenRepresentationShift
        }
        EarliestTemporalShiftStageV0::Logits => {
            TemporalDistributionShiftStatusV0::LogitDistributionShift
        }
        EarliestTemporalShiftStageV0::Probabilities => {
            TemporalDistributionShiftStatusV0::ProbabilityDistributionShift
        }
        EarliestTemporalShiftStageV0::OutcomesOnly => TemporalDistributionShiftStatusV0::Stable,
        EarliestTemporalShiftStageV0::MultipleStages => {
            TemporalDistributionShiftStatusV0::MultiStageShift
        }
        EarliestTemporalShiftStageV0::InsufficientEvidence => {
            TemporalDistributionShiftStatusV0::InsufficientSamples
        }
    }
}

fn temporal_root_cause_v0(
    stage: EarliestTemporalShiftStageV0,
    probability_shift: &DistributionShiftMetricBundleV0,
) -> ProbabilityCollapseRootCauseV0 {
    match stage {
        EarliestTemporalShiftStageV0::RawFeatures => {
            ProbabilityCollapseRootCauseV0::RawFeatureCollapse
        }
        EarliestTemporalShiftStageV0::NormalizedFeatures => {
            ProbabilityCollapseRootCauseV0::FeatureScaleDrift
        }
        EarliestTemporalShiftStageV0::Sequences => {
            ProbabilityCollapseRootCauseV0::SequenceSupportBreach
        }
        EarliestTemporalShiftStageV0::FrozenRepresentations => {
            ProbabilityCollapseRootCauseV0::FrozenRepresentationSupportBreach
        }
        EarliestTemporalShiftStageV0::RepresentationScale => {
            ProbabilityCollapseRootCauseV0::RepresentationScaleDrift
        }
        EarliestTemporalShiftStageV0::Logits => {
            ProbabilityCollapseRootCauseV0::LogitVarianceCollapse
        }
        EarliestTemporalShiftStageV0::Probabilities
            if probability_shift.out_of_support_fraction > 0.0 =>
        {
            ProbabilityCollapseRootCauseV0::ProbabilitySaturation
        }
        EarliestTemporalShiftStageV0::Probabilities => {
            ProbabilityCollapseRootCauseV0::HeadBiasSensitivity
        }
        EarliestTemporalShiftStageV0::OutcomesOnly => {
            ProbabilityCollapseRootCauseV0::OutcomePrevalenceShift
        }
        _ => ProbabilityCollapseRootCauseV0::Unknown,
    }
}

fn validation_status(
    bundle: &ForecastMetricBundleV0,
    gate: &ValidationSignalGateConfigV0,
) -> ValidationSignalStatusV0 {
    if !bundle.finite {
        ValidationSignalStatusV0::NumericallyInvalid
    } else if bundle.evaluation.sample_count < gate.minimum_samples {
        ValidationSignalStatusV0::InsufficientSamples
    } else if bundle.collapse.is_collapsed() {
        ValidationSignalStatusV0::Collapsed
    } else if bundle.collapse.probability_stddev < gate.minimum_probability_stddev
        || bundle.collapse.mean_entropy < gate.minimum_entropy
    {
        ValidationSignalStatusV0::ConstantLike
    } else if bundle.brier.resolution < gate.minimum_brier_resolution {
        ValidationSignalStatusV0::NoResolution
    } else if bundle.rank_auc.status == BinaryRankAucStatusV0::UndefinedSingleClass {
        ValidationSignalStatusV0::SingleClassValidation
    } else if gate.minimum_rank_auc_margin.is_some_and(|margin| {
        bundle
            .rank_auc
            .value
            .is_some_and(|value| value < 0.5 + margin)
    }) {
        ValidationSignalStatusV0::NoDiscrimination
    } else {
        ValidationSignalStatusV0::Usable
    }
}

fn checkpoint_eligibility(
    bundle: &ForecastMetricBundleV0,
    constant_brier: f32,
    gate: &ValidationSignalGateConfigV0,
) -> CheckpointEligibilityV0 {
    match validation_status(bundle, gate) {
        ValidationSignalStatusV0::NumericallyInvalid => {
            CheckpointEligibilityV0::RejectedNumericalFailure
        }
        ValidationSignalStatusV0::InsufficientSamples => {
            CheckpointEligibilityV0::RejectedInsufficientSamples
        }
        ValidationSignalStatusV0::Collapsed => CheckpointEligibilityV0::RejectedCollapse,
        ValidationSignalStatusV0::ConstantLike => CheckpointEligibilityV0::RejectedConstantLike,
        ValidationSignalStatusV0::NoResolution => CheckpointEligibilityV0::RejectedNoResolution,
        ValidationSignalStatusV0::NoDiscrimination => {
            CheckpointEligibilityV0::RejectedNoDiscrimination
        }
        ValidationSignalStatusV0::SingleClassValidation => {
            CheckpointEligibilityV0::RejectedSingleClassValidation
        }
        _ if bundle.evaluation.brier_score
            > constant_brier + gate.maximum_brier_delta_vs_constant =>
        {
            CheckpointEligibilityV0::RejectedWorseThanConstant
        }
        _ => CheckpointEligibilityV0::Eligible,
    }
}

pub fn run_momentum_probability_collapse_forensics_v0(
    config: &MomentumLearningCampaignConfigV0,
    encoder: &FrozenMamba3EncoderV0,
    train: &[SequenceExampleV0],
    validation: &[SequenceExampleV0],
    test: &[SequenceExampleV0],
    collapse_config: &ProbabilityCollapseConfigV0,
) -> Result<MomentumProbabilityCollapseForensicsV0, CampaignErrorV0> {
    run_momentum_probability_collapse_forensics_with_temporal_inputs_v0(
        config,
        encoder,
        train,
        validation,
        test,
        collapse_config,
        None,
        "sealed-window",
    )
}

fn run_momentum_probability_collapse_forensics_with_temporal_inputs_v0(
    config: &MomentumLearningCampaignConfigV0,
    encoder: &FrozenMamba3EncoderV0,
    train: &[SequenceExampleV0],
    validation: &[SequenceExampleV0],
    test: &[SequenceExampleV0],
    collapse_config: &ProbabilityCollapseConfigV0,
    temporal_inputs: Option<&TemporalFeaturePartitionsV0>,
    window_id: &str,
) -> Result<MomentumProbabilityCollapseForensicsV0, CampaignErrorV0> {
    config.validate()?;
    collapse_config.validate()?;
    if train.is_empty() || validation.is_empty() || test.is_empty() {
        return Err(CampaignErrorV0::InsufficientHistory);
    }
    let train_encoded = encoder.encode_batch(train)?;
    let validation_encoded = encoder.encode_batch(validation)?;
    let normalizer = RepresentationNormalizerV0::fit(&train_encoded)?;
    let candidates = vec![
        MomentumForensicCandidateV0::C0Reference,
        MomentumForensicCandidateV0::C1RepresentationNormalized,
        MomentumForensicCandidateV0::C2PrevalenceBias,
        MomentumForensicCandidateV0::C3Combined,
    ];
    let mut candidate_results = Vec::with_capacity(candidates.len());
    let mut trained = Vec::with_capacity(candidates.len());
    for candidate in &candidates {
        let (candidate_train, candidate_validation) = candidate_encoded_partitions(
            *candidate,
            &normalizer,
            &train_encoded,
            &validation_encoded,
        )?;
        let mut head = LogisticPredictionHeadV0::seeded(
            candidate_train[0].representation.len(),
            deterministic_candidate_seed(config.campaign_seed, *candidate),
        )?;
        if candidate.uses_prevalence_bias() {
            head.bias = prevalence_logit(&candidate_train)?;
        }
        let (trajectory, heads) = trace_encoded_head(
            head,
            *candidate,
            &candidate_train,
            &candidate_validation,
            &config.training_config,
            collapse_config,
            &config.validation_signal_gate,
        )?;
        let head = trajectory
            .selected_checkpoint
            .as_ref()
            .and_then(|selected| heads.get(selected.epoch.saturating_sub(1)))
            .cloned()
            .or_else(|| heads.last().cloned())
            .ok_or(CampaignErrorV0::Learning)?;
        let validation_metrics = evaluate_head_v0(&head, &candidate_validation)?;
        let validation_probabilities = probabilities_for_head(&head, &candidate_validation)?;
        let validation_labels = candidate_validation
            .iter()
            .map(|example| example.label)
            .collect::<Vec<_>>();
        let validation_collapse = probability_collapse_metrics_v0(
            &validation_probabilities,
            &validation_labels,
            collapse_config,
        )?;
        let eligible_for_selection = trajectory.selected_checkpoint.is_some();
        candidate_results.push(MomentumForensicCandidateResultV0 {
            candidate: *candidate,
            frozen_head: head.clone(),
            validation: validation_metrics,
            validation_collapse,
            test: None,
            test_collapse: None,
            eligible_for_selection,
            selected_checkpoint: trajectory.selected_checkpoint.clone(),
            trajectory: Some(trajectory),
            generalization_status:
                CheckpointGeneralizationStatusV0::TestNotEvaluatedNoEligibleCheckpoint,
        });
        trained.push((
            head,
            candidate.uses_representation_normalization(),
            candidate_train,
            candidate_validation,
        ));
    }
    let selected_index =
        select_forensic_candidate(&candidate_results, collapse_config.comparison_epsilon);
    let mut test_partition_opened_once = false;
    let mut temporal_generalization = None;
    if let Some(index) = selected_index {
        let (head, uses_normalization, candidate_train, candidate_validation) = &trained[index];
        let test_encoded = encoder.encode_batch(test)?;
        let test_encoded = if *uses_normalization {
            normalizer.transform(&test_encoded)?
        } else {
            test_encoded
        };
        let train_rows = candidate_train
            .iter()
            .map(|row| row.representation.clone())
            .collect::<Vec<_>>();
        let validation_rows = candidate_validation
            .iter()
            .map(|row| row.representation.clone())
            .collect::<Vec<_>>();
        let test_rows = test_encoded
            .iter()
            .map(|row| row.representation.clone())
            .collect::<Vec<_>>();
        let raw_representation_train = train_encoded
            .iter()
            .map(|row| row.representation.clone())
            .collect::<Vec<_>>();
        let raw_representation_test = encoder
            .encode_batch(test)?
            .iter()
            .map(|row| row.representation.clone())
            .collect::<Vec<_>>();
        let envelope = DistributionSupportEnvelopeV0::fit(&train_rows, &config.support_gate)?;
        let train_history_support_audit =
            train_history_support_audit_v0(&train_rows, &config.support_gate);
        let support_envelope_constant_dimension_count = envelope
            .scales
            .iter()
            .filter(|scale| **scale <= config.support_gate.comparison_epsilon)
            .count();
        let validation_shift =
            distribution_shift_metrics_v0(&train_rows, &validation_rows, &envelope)?;
        let test_shift = distribution_shift_metrics_v0(&train_rows, &test_rows, &envelope)?;
        let frozen_representation_shift = distribution_shift_for_rows_v0(
            &raw_representation_train,
            &raw_representation_test,
            &config.support_gate,
        )?;
        let sequence_shift = distribution_shift_for_rows_v0(
            &flatten_sequence_inputs_v0(train),
            &flatten_sequence_inputs_v0(test),
            &config.support_gate,
        )?;
        let raw_feature_shift = temporal_inputs
            .map(|inputs| {
                distribution_shift_for_rows_v0(
                    &inputs.raw_train,
                    &inputs.raw_test,
                    &config.support_gate,
                )
            })
            .transpose()?;
        let normalized_feature_shift = temporal_inputs
            .map(|inputs| {
                distribution_shift_for_rows_v0(
                    &inputs.normalized_train,
                    &inputs.normalized_test,
                    &config.support_gate,
                )
            })
            .transpose()?;
        let logits_train = head_logits_v0(head, &candidate_train)?;
        let logits_test = head_logits_v0(head, &test_encoded)?;
        let logit_shift =
            distribution_shift_for_scalars_v0(&logits_train, &logits_test, &config.support_gate)?;
        let probabilities_train = probabilities_for_head(head, &candidate_train)?;
        let probabilities_test = probabilities_for_head(head, &test_encoded)?;
        let probability_shift = distribution_shift_for_scalars_v0(
            &probabilities_train,
            &probabilities_test,
            &config.support_gate,
        )?;
        let validation_support = support_decision(&validation_shift, &config.support_gate, true);
        let test_support = if validation_support == ShadowSupportDecisionV0::InSupport {
            support_decision(&test_shift, &config.support_gate, false)
        } else {
            ShadowSupportDecisionV0::NotEvaluated
        };
        let validation_support_coverage = 1.0 - validation_shift.out_of_support_fraction;
        let decision_digest = stable_hash_string(&format!(
            "{}:{}:{:.8}:{}",
            shadow_support_decision_label_v0(validation_support),
            shadow_support_decision_label_v0(test_support),
            validation_support_coverage,
            envelope.digest,
        ));
        let test_metrics = evaluate_head_v0(head, &test_encoded)?;
        let test_labels = test_encoded
            .iter()
            .map(|example| example.label)
            .collect::<Vec<_>>();
        let test_collapse =
            probability_collapse_metrics_v0(&probabilities_test, &test_labels, collapse_config)?;
        let train_labels = candidate_train
            .iter()
            .map(|example| example.label)
            .collect::<Vec<_>>();
        let outcome_shift =
            distribution_shift_for_scalars_v0(&train_labels, &test_labels, &config.support_gate)?;
        let earliest_shift_stage = earliest_temporal_shift_stage_v0(
            raw_feature_shift.as_ref(),
            normalized_feature_shift.as_ref(),
            &sequence_shift,
            &frozen_representation_shift,
            &test_shift,
            &logit_shift,
            &probability_shift,
            &outcome_shift,
            &config.support_gate,
        );
        let shift_status = temporal_shift_status_v0(earliest_shift_stage);
        let root_cause = temporal_root_cause_v0(earliest_shift_stage, &probability_shift);
        temporal_generalization = Some(TemporalGeneralizationResultV0 {
            validation_support_decision: validation_support,
            test_support_decision: test_support,
            validation_support_coverage,
            support_envelope_digest: envelope.digest.clone(),
            support_envelope_constant_dimension_count,
            train_history_support_audit,
            raw_feature_shift,
            normalized_feature_shift,
            sequence_shift,
            frozen_representation_shift,
            representation_shift: test_shift.clone(),
            validation_representation_shift: validation_shift,
            logit_shift,
            probability_shift,
            outcome_shift,
            earliest_shift_stage,
            shift_status,
            root_cause,
            counterfactual_test_evaluated: true,
            decision_digest,
        });
        candidate_results[index].test = Some(test_metrics);
        candidate_results[index].test_collapse = Some(test_collapse);
        candidate_results[index].generalization_status = if candidate_results[index]
            .test_collapse
            .as_ref()
            .is_some_and(ProbabilityCollapseMetricsV0::is_collapsed)
        {
            CheckpointGeneralizationStatusV0::TemporalGeneralizationCollapse
        } else {
            CheckpointGeneralizationStatusV0::GeneralizedWithoutCollapse
        };
        test_partition_opened_once = true;
    }
    let selected_candidate = selected_index.map(|index| candidates[index]);
    let selected_result = selected_index.map(|index| &candidate_results[index]);
    let diagnostic_status = match selected_result {
        Some(result)
            if result
                .test_collapse
                .as_ref()
                .is_some_and(|collapse| collapse.is_collapsed()) =>
        {
            ProbabilityCollapseDiagnosticStatusV0::Reproduced
        }
        Some(_) => ProbabilityCollapseDiagnosticStatusV0::RootCauseIdentified,
        None => ProbabilityCollapseDiagnosticStatusV0::InsufficientDiagnosticEvidence,
    };
    let root_cause = temporal_generalization
        .as_ref()
        .filter(|result| result.earliest_shift_stage != EarliestTemporalShiftStageV0::None)
        .map(|result| result.root_cause)
        .or_else(|| {
            selected_result
                .and_then(|result| result.test_collapse.as_ref())
                .map(classify_probability_root_cause)
        })
        .unwrap_or(ProbabilityCollapseRootCauseV0::Unknown);
    let selected_checkpoint = selected_result.and_then(|result| result.selected_checkpoint.clone());
    let abstention = if selected_candidate.is_none() {
        Some(ShadowLearningAbstentionV0 {
            agent_id: config.agent_id.clone(),
            campaign_id: config.campaign_id.clone(),
            window_id: window_id.to_string(),
            reason: ShadowLearningAbstentionReasonV0::NoUsableValidationSignal,
            eligible_to_vote: false,
            eligible_to_execute: false,
            eligible_for_promotion: false,
        })
    } else if temporal_generalization
        .as_ref()
        .is_some_and(|result| result.test_support_decision != ShadowSupportDecisionV0::InSupport)
    {
        Some(ShadowLearningAbstentionV0 {
            agent_id: config.agent_id.clone(),
            campaign_id: config.campaign_id.clone(),
            window_id: window_id.to_string(),
            reason: if temporal_generalization.as_ref().is_some_and(|result| {
                result.validation_support_decision == ShadowSupportDecisionV0::OutOfSupport
                    || result.test_support_decision == ShadowSupportDecisionV0::OutOfSupport
            }) {
                ShadowLearningAbstentionReasonV0::TemporalOutOfSupport
            } else {
                ShadowLearningAbstentionReasonV0::TemporalSupportUnavailable
            },
            eligible_to_vote: false,
            eligible_to_execute: false,
            eligible_for_promotion: false,
        })
    } else {
        None
    };
    Ok(MomentumProbabilityCollapseForensicsV0 {
        window_id: window_id.to_string(),
        diagnostic_status,
        root_cause,
        candidates,
        selected_candidate,
        candidate_results,
        test_partition_opened_once,
        selected_checkpoint,
        representation_normalizer_digest: normalizer.digest(),
        abstention,
        temporal_generalization,
    })
}

fn candidate_encoded_partitions(
    candidate: MomentumForensicCandidateV0,
    normalizer: &RepresentationNormalizerV0,
    train: &[EncodedTrainingExampleV0],
    validation: &[EncodedTrainingExampleV0],
) -> Result<(Vec<EncodedTrainingExampleV0>, Vec<EncodedTrainingExampleV0>), CampaignErrorV0> {
    if candidate.uses_representation_normalization() {
        Ok((
            normalizer.transform(train)?,
            normalizer.transform(validation)?,
        ))
    } else {
        Ok((train.to_vec(), validation.to_vec()))
    }
}

fn deterministic_candidate_seed(seed: u64, candidate: MomentumForensicCandidateV0) -> u64 {
    seed ^ match candidate {
        MomentumForensicCandidateV0::C0Reference => 0xC0,
        MomentumForensicCandidateV0::C1RepresentationNormalized => 0xC1,
        MomentumForensicCandidateV0::C2PrevalenceBias => 0xC2,
        MomentumForensicCandidateV0::C3Combined => 0xC3,
    }
}

fn prevalence_logit(examples: &[EncodedTrainingExampleV0]) -> Result<f32, CampaignErrorV0> {
    if examples.is_empty()
        || examples
            .iter()
            .any(|example| !(0.0..=1.0).contains(&example.label))
    {
        return Err(CampaignErrorV0::InsufficientHistory);
    }
    let prevalence =
        examples.iter().map(|example| example.label).sum::<f32>() / examples.len() as f32;
    let prevalence = prevalence.clamp(1e-4, 1.0 - 1e-4);
    let bias = (prevalence / (1.0 - prevalence)).ln();
    if bias.is_finite() {
        Ok(bias)
    } else {
        Err(CampaignErrorV0::Learning)
    }
}

fn train_encoded_head(
    head: &mut LogisticPredictionHeadV0,
    train: &[EncodedTrainingExampleV0],
    validation: &[EncodedTrainingExampleV0],
    config: &HeadTrainingConfigV0,
) -> Result<LogisticPredictionHeadV0, CampaignErrorV0> {
    config.validate()?;
    if train.is_empty() || validation.is_empty() {
        return Err(CampaignErrorV0::InsufficientHistory);
    }
    let mut best = head.clone();
    let mut best_validation = f32::INFINITY;
    let mut stale_epochs = 0usize;
    for _ in 0..config.epochs {
        for batch in train.chunks(config.batch_size) {
            let (_, gradients) = brier_loss_and_gradients_v0(head, batch)?;
            apply_sgd_v0(head, &gradients, &config.optimizer)?;
        }
        let validation_brier = brier_loss_and_gradients_v0(head, validation)?.0;
        if validation_brier + 1e-8 < best_validation {
            best_validation = validation_brier;
            best = head.clone();
            stale_epochs = 0;
        } else {
            stale_epochs += 1;
            if config
                .early_stopping_patience
                .is_some_and(|patience| stale_epochs >= patience)
            {
                break;
            }
        }
    }
    Ok(best)
}

fn trace_encoded_head(
    mut head: LogisticPredictionHeadV0,
    candidate: MomentumForensicCandidateV0,
    train: &[EncodedTrainingExampleV0],
    validation: &[EncodedTrainingExampleV0],
    config: &HeadTrainingConfigV0,
    collapse: &ProbabilityCollapseConfigV0,
    gate: &ValidationSignalGateConfigV0,
) -> Result<(CheckpointTrajectoryV0, Vec<LogisticPredictionHeadV0>), CampaignErrorV0> {
    config.validate()?;
    let prevalence = train.iter().map(|row| row.label).sum::<f32>() / train.len() as f32;
    let constant_brier = validation
        .iter()
        .map(|row| (prevalence - row.label).powi(2))
        .sum::<f32>()
        / validation.len() as f32;
    let mut checkpoints = Vec::new();
    let mut heads = Vec::new();
    let mut stale = 0usize;
    let mut best_brier = f32::INFINITY;
    for epoch in 1..=config.epochs {
        let before = head.clone();
        let mut gradient_norm = 0.0;
        for batch in train.chunks(config.batch_size) {
            let (_, gradients) = brier_loss_and_gradients_v0(&head, batch)?;
            gradient_norm += (gradients
                .weight_gradients
                .iter()
                .map(|value| value * value)
                .sum::<f32>()
                + gradients.bias_gradient.powi(2))
            .sqrt();
            apply_sgd_v0(&mut head, &gradients, &config.optimizer)?;
        }
        let update_norm = (head
            .weights
            .iter()
            .zip(&before.weights)
            .map(|(after, prior)| (after - prior).powi(2))
            .sum::<f32>()
            + (head.bias - before.bias).powi(2))
        .sqrt();
        let train_metrics = forecast_metric_bundle(&head, train, collapse, gate)?;
        let validation_metrics = forecast_metric_bundle(&head, validation, collapse, gate)?;
        let status = validation_status(&validation_metrics, gate);
        let eligibility = checkpoint_eligibility(&validation_metrics, constant_brier, gate);
        let brier = validation_metrics.evaluation.brier_score;
        checkpoints.push(CheckpointObservationV0 {
            epoch,
            train: train_metrics,
            validation: validation_metrics,
            head_digest: head.parameter_digest(),
            weight_norm: head_weight_norm(&head),
            bias: head.bias,
            gradient_norm,
            update_norm,
            finite: head.bias.is_finite() && head.weights.iter().all(|value| value.is_finite()),
            signal_status: status,
            eligibility,
        });
        heads.push(head.clone());
        if brier < best_brier {
            best_brier = brier;
            stale = 0;
        } else {
            stale += 1;
        }
        if config
            .early_stopping_patience
            .is_some_and(|limit| stale >= limit)
        {
            break;
        }
    }
    let mut frontier = checkpoints
        .iter()
        .filter(|row| row.eligibility == CheckpointEligibilityV0::Eligible)
        .map(|row| EligibleCheckpointV0 {
            reference: CheckpointRefV0 {
                candidate,
                epoch: row.epoch,
            },
            validation_brier: row.validation.evaluation.brier_score,
            reliability: row.validation.brier.reliability,
            resolution: row.validation.brier.resolution,
            entropy: row.validation.collapse.mean_entropy,
            probability_stddev: row.validation.collapse.probability_stddev,
            rank_auc: row.validation.rank_auc.value,
            head_digest: row.head_digest.clone(),
            update_norm: row.update_norm,
        })
        .collect::<Vec<_>>();
    frontier.sort_by(|left, right| {
        left.validation_brier
            .total_cmp(&right.validation_brier)
            .then_with(|| right.resolution.total_cmp(&left.resolution))
            .then_with(|| left.reliability.total_cmp(&right.reliability))
            .then_with(|| left.update_norm.total_cmp(&right.update_norm))
            .then_with(|| left.reference.epoch.cmp(&right.reference.epoch))
    });
    let reasons = [
        CheckpointEligibilityV0::RejectedCollapse,
        CheckpointEligibilityV0::RejectedNoResolution,
        CheckpointEligibilityV0::RejectedNoDiscrimination,
        CheckpointEligibilityV0::RejectedConstantLike,
        CheckpointEligibilityV0::RejectedWorseThanConstant,
        CheckpointEligibilityV0::RejectedInsufficientSamples,
        CheckpointEligibilityV0::RejectedSingleClassValidation,
        CheckpointEligibilityV0::RejectedNumericalFailure,
    ]
    .into_iter()
    .map(|reason| {
        (
            reason,
            checkpoints
                .iter()
                .filter(|row| row.eligibility == reason)
                .count(),
        )
    })
    .filter(|(_, count)| *count > 0)
    .collect::<Vec<_>>();
    let digest = stable_hash_string(&format!(
        "{:?}",
        frontier
            .iter()
            .map(|row| (&row.reference, &row.head_digest))
            .collect::<Vec<_>>()
    ));
    let old_best = checkpoints.iter().min_by(|left, right| {
        left.validation
            .evaluation
            .brier_score
            .total_cmp(&right.validation.evaluation.brier_score)
    });
    let status = if !frontier.is_empty() {
        CheckpointTrajectoryStatusV0::HealthyEligibleCheckpointFound
    } else if old_best
        .is_some_and(|row| row.eligibility == CheckpointEligibilityV0::RejectedCollapse)
    {
        CheckpointTrajectoryStatusV0::CollapsedCheckpointSelectedByOldPolicy
    } else {
        CheckpointTrajectoryStatusV0::NoUsableValidationSignal
    };
    let selected_checkpoint = frontier.first().map(|row| row.reference.clone());
    Ok((
        CheckpointTrajectoryV0 {
            candidate,
            checkpoints,
            frontier: EligibleCheckpointFrontierV0 {
                checkpoints: frontier,
                rejected_count_by_reason: reasons,
                digest,
            },
            selected_checkpoint,
            status,
        },
        heads,
    ))
}

fn probabilities_for_head(
    head: &LogisticPredictionHeadV0,
    examples: &[EncodedTrainingExampleV0],
) -> Result<Vec<f32>, CampaignErrorV0> {
    examples
        .iter()
        .map(|example| {
            head.probability(&example.representation)
                .map_err(Into::into)
        })
        .collect()
}

fn select_forensic_candidate(
    candidates: &[MomentumForensicCandidateResultV0],
    epsilon: f32,
) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.eligible_for_selection)
        .min_by(|(_, left), (_, right)| {
            let difference = left.validation.brier_score - right.validation.brier_score;
            if difference.abs() <= epsilon {
                left.candidate.label().cmp(right.candidate.label())
            } else {
                left.validation
                    .brier_score
                    .total_cmp(&right.validation.brier_score)
            }
        })
        .map(|(index, _)| index)
}

fn classify_probability_root_cause(
    metrics: &ProbabilityCollapseMetricsV0,
) -> ProbabilityCollapseRootCauseV0 {
    if metrics
        .subtypes
        .iter()
        .any(|subtype| matches!(subtype, ProbabilityCollapseSubtypeV0::SaturatedProbability))
    {
        ProbabilityCollapseRootCauseV0::ProbabilitySaturation
    } else if metrics.subtypes.iter().any(|subtype| {
        matches!(
            subtype,
            ProbabilityCollapseSubtypeV0::NearConstantProbability
        )
    }) {
        ProbabilityCollapseRootCauseV0::ValidationCheckpointCollapse
    } else {
        ProbabilityCollapseRootCauseV0::Unknown
    }
}

fn mean_f32(values: &[f32]) -> Result<f32, CampaignErrorV0> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err(CampaignErrorV0::Learning);
    }
    Ok(values.iter().sum::<f32>() / values.len() as f32)
}

fn stddev_f32(values: &[f32], mean: f32) -> Result<f32, CampaignErrorV0> {
    if !mean.is_finite() || values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err(CampaignErrorV0::Learning);
    }
    let value = (values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f32>()
        / values.len() as f32)
        .sqrt();
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CampaignErrorV0::Learning)
    }
}

pub fn run_momentum_learning_campaign_v0(
    config: &MomentumLearningCampaignConfigV0,
    snapshots: &[DataSnapshot],
    encoder: &FrozenMamba3EncoderV0,
) -> Result<MomentumLearningCampaignResultV0, CampaignErrorV0> {
    config.validate()?;
    let mut safety_trace = CampaignSafetyTraceV0::default();
    if snapshots.is_empty() {
        trace_rejection(
            &mut safety_trace,
            CampaignSafetyGateV0::ImmutableSanitizedEvidence,
            "no_historical_learning_evidence",
            vec!["snapshot_count=0".to_string()],
        );
        return Ok(empty_result_with_trace(
            config,
            MomentumLearningCampaignStatusV0::NoHistoricalLearningEvidence,
            "no_historical_learning_evidence",
            complete_safety_trace(safety_trace),
        ));
    }
    let evidence = match validate_historical_evidence(snapshots, &mut safety_trace) {
        Ok(evidence) => evidence,
        Err(error) => {
            return Ok(empty_result_with_trace(
                config,
                MomentumLearningCampaignStatusV0::RejectedForSafety,
                error_label(error),
                complete_safety_trace(safety_trace),
            ));
        }
    };
    if evidence.candles.len() < config.minimum_history_rows {
        trace_rejection(
            &mut safety_trace,
            CampaignSafetyGateV0::MinimumHistory,
            "insufficient_historical_rows",
            vec![format!("row_count={}", evidence.candles.len())],
        );
        return Ok(empty_result_with_trace(
            config,
            MomentumLearningCampaignStatusV0::NoHistoricalLearningEvidence,
            "insufficient_historical_rows",
            complete_safety_trace(safety_trace),
        ));
    }
    trace_pass(
        &mut safety_trace,
        CampaignSafetyGateV0::MinimumHistory,
        vec![format!("row_count={}", evidence.candles.len())],
    );
    if !backend_is_permitted(config, encoder) {
        trace_rejection(
            &mut safety_trace,
            CampaignSafetyGateV0::CpuFullInferenceReady,
            "backend_not_full_ready_cpu",
            vec!["cpu_full_inference_ready=false".to_string()],
        );
        return Ok(empty_result_with_trace(
            config,
            MomentumLearningCampaignStatusV0::BackendUnavailable,
            "backend_not_full_ready_cpu",
            complete_safety_trace(safety_trace),
        ));
    }
    trace_pass(
        &mut safety_trace,
        CampaignSafetyGateV0::CpuFullInferenceReady,
        vec!["cpu_full_inference_ready=true".to_string()],
    );
    let windows =
        build_momentum_learning_windows_v0(config, evidence.candles.len(), &evidence.snapshot_ids)?;
    trace_pass(
        &mut safety_trace,
        CampaignSafetyGateV0::PurgedChronologicalWindows,
        vec![format!("window_count={}", windows.len())],
    );
    trace_pass(
        &mut safety_trace,
        CampaignSafetyGateV0::FrozenEncoderCaptured,
        vec!["parameter_digest_present=true".to_string()],
    );
    trace_pass(
        &mut safety_trace,
        CampaignSafetyGateV0::OfflineShadowLearning,
        vec![
            "deployment=shadow_only".to_string(),
            "provider_or_transport_input=false".to_string(),
        ],
    );
    safety_trace.eligibility.offline_shadow_learning = true;
    trace_blocked(
        &mut safety_trace,
        CampaignSafetyGateV0::PromotionEligibility,
        "experimental_internal_reference",
    );
    trace_blocked(
        &mut safety_trace,
        CampaignSafetyGateV0::VotingEligibility,
        "shadow_only",
    );
    trace_blocked(
        &mut safety_trace,
        CampaignSafetyGateV0::ExecutionEligibility,
        "official_oracle_execution_blocked",
    );
    let raw_features = build_momentum_features_v0(&evidence.candles, &config.feature_config)?;
    let encoder_digest = encoder.parameter_digest();
    let mut journal = SandboxModelVersionJournalV0::default();
    let mut results = Vec::new();
    let mut collapse_forensics = Vec::new();
    let mut rejected_windows = Vec::new();
    let mut last_parent: Option<WarmParent> = None;

    for window in windows {
        let train_rows = rows_in_range(&raw_features, &window.train_range);
        let normalizer = FeatureNormalizerV0::fit(&train_rows)?;
        let normalized = normalizer.transform(&raw_features)?;
        let temporal_inputs = TemporalFeaturePartitionsV0 {
            raw_train: train_rows.iter().map(|row| row.values.clone()).collect(),
            raw_test: rows_in_range(&raw_features, &window.test_range)
                .iter()
                .map(|row| row.values.clone())
                .collect(),
            normalized_train: rows_in_range(&normalized, &window.train_range)
                .iter()
                .map(|row| row.values.clone())
                .collect(),
            normalized_test: rows_in_range(&normalized, &window.test_range)
                .iter()
                .map(|row| row.values.clone())
                .collect(),
        };
        let all_examples = build_momentum_sequence_examples_v0(
            &evidence.candles,
            &normalized,
            &config.sequence_config,
            &window.snapshot_ids,
        )?;
        let train = examples_in_range(&all_examples, &window.train_range);
        let validation = examples_in_range(&all_examples, &window.validation_range);
        let test = examples_in_range(&all_examples, &window.test_range);
        if train.is_empty() || validation.is_empty() || test.len() < config.minimum_test_samples {
            rejected_windows.push(RejectedLearningWindowV0 {
                window_id: window.window_id.clone(),
                path: None,
                reason: CampaignErrorV0::InsufficientHistory,
            });
            continue;
        }
        if train
            .last()
            .is_some_and(|row| row.label_index >= validation[0].sequence_start)
            || validation
                .last()
                .is_some_and(|row| row.label_index >= test[0].sequence_start)
        {
            rejected_windows.push(RejectedLearningWindowV0 {
                window_id: window.window_id.clone(),
                path: None,
                reason: CampaignErrorV0::LeakageInvariantFailed,
            });
            continue;
        }
        collapse_forensics.push(
            run_momentum_probability_collapse_forensics_with_temporal_inputs_v0(
                config,
                encoder,
                &train,
                &validation,
                &test,
                &config.collapse_config,
                Some(&temporal_inputs),
                &window.window_id,
            )?,
        );
        let mut paths = Vec::new();
        let requested_paths = match config.initialization_policy {
            HeadInitializationPolicyV0::ColdStartEachWindow => vec![MomentumLearningPathV0::Cold],
            HeadInitializationPolicyV0::WarmStartPreviousEligible => {
                vec![MomentumLearningPathV0::Warm]
            }
            HeadInitializationPolicyV0::CompareColdAndWarm => {
                vec![MomentumLearningPathV0::Cold, MomentumLearningPathV0::Warm]
            }
        };
        for path in requested_paths {
            let initial = match initial_head_for_path(
                config,
                path,
                &window,
                &train,
                encoder,
                last_parent.as_ref(),
            ) {
                Ok(value) => value,
                Err(error) => {
                    rejected_windows.push(RejectedLearningWindowV0 {
                        window_id: window.window_id.clone(),
                        path: Some(path),
                        reason: error,
                    });
                    continue;
                }
            };
            let parent_id = if path == MomentumLearningPathV0::Warm {
                last_parent
                    .as_ref()
                    .map(|parent| parent.version.model_version_id.clone())
            } else {
                None
            };
            match train_path(
                config,
                path,
                parent_id,
                &window,
                &train,
                &validation,
                &test,
                normalizer.digest(),
                initial,
                encoder,
                &encoder_digest,
            ) {
                Ok(path_result) => paths.push(path_result),
                Err(error) => rejected_windows.push(RejectedLearningWindowV0 {
                    window_id: window.window_id.clone(),
                    path: Some(path),
                    reason: error,
                }),
            }
        }
        if paths.is_empty() {
            continue;
        }
        let drift_status = per_window_drift(&paths, &config.drift_config);
        for path in &mut paths {
            path.version.drift_status = Some(format!("{:?}", drift_status));
        }
        let mut journaled_paths = Vec::with_capacity(paths.len());
        for path in paths {
            if journal.insert(path.version.clone()).is_err() {
                rejected_windows.push(RejectedLearningWindowV0 {
                    window_id: window.window_id.clone(),
                    path: Some(path.path),
                    reason: CampaignErrorV0::VersionCycle,
                });
            } else {
                journaled_paths.push(path);
            }
        }
        let paths = journaled_paths;
        if paths.is_empty() {
            continue;
        }
        if let Some(cold) = paths
            .iter()
            .find(|path| path.path == MomentumLearningPathV0::Cold)
        {
            last_parent = Some(WarmParent {
                window_id: window.window_id.clone(),
                version: cold.version.clone(),
                head: cold.final_head.clone(),
            });
        } else if let Some(warm) = paths.first() {
            last_parent = Some(WarmParent {
                window_id: window.window_id.clone(),
                version: warm.version.clone(),
                head: warm.final_head.clone(),
            });
        }
        results.push(MomentumLearningWindowResultV0 {
            window,
            normalizer_digest: normalizer.digest(),
            feature_config_digest: config.feature_config.digest(),
            feature_order: config.feature_config.feature_names(),
            paths,
            drift_status,
        });
    }

    if encoder.parameter_digest() != encoder_digest {
        return Err(CampaignErrorV0::Learning);
    }
    trace_pass(
        &mut safety_trace,
        CampaignSafetyGateV0::FrozenEncoderUnchanged,
        vec!["parameter_digest_unchanged=true".to_string()],
    );
    let generated_versions = results
        .iter()
        .flat_map(|window| window.paths.iter().map(|path| path.version.clone()))
        .collect::<Vec<_>>();
    let aggregate_mamba_evidence = aggregate_mamba_evidence(&results, &config.aggregate_gate);
    let warm_start_evidence = aggregate_warm_start_evidence(&results, &config.aggregate_gate);
    let aggregate_drift = aggregate_drift(&results, &config.drift_config);
    let status = campaign_status(&results, &aggregate_mamba_evidence, aggregate_drift, config);
    let shadow_assessments = results
        .iter()
        .flat_map(|window| {
            window
                .paths
                .iter()
                .map(|path| path.shadow_assessment.clone())
        })
        .collect();
    Ok(MomentumLearningCampaignResultV0 {
        campaign_id: config.campaign_id.clone(),
        status,
        windows: results,
        aggregate_mamba_evidence,
        warm_start_evidence,
        aggregate_drift,
        generated_versions,
        shadow_assessments,
        rejected_windows,
        reason_codes: vec!["offline_shadow_only".to_string(), config.digest()],
        safety_trace: complete_safety_trace(safety_trace),
        collapse_forensics,
        validation_signal_gate: config.validation_signal_gate.clone(),
        support_gate: config.support_gate.clone(),
    })
}

pub fn build_momentum_temporal_diagnostic_report_v0(
    campaign: &MomentumLearningCampaignResultV0,
    evidence_row_count: usize,
    evidence_pack_digest: &str,
) -> MomentumTemporalDiagnosticReportV0 {
    let selected = campaign
        .collapse_forensics
        .iter()
        .find(|forensics| forensics.selected_candidate.is_some());
    let temporal = selected.and_then(|forensics| forensics.temporal_generalization.as_ref());
    let comparisons = warm_cold_trajectory_comparisons_v0(&campaign.windows);
    let warm_start_status =
        warm_start_lock_in_status_v0(&comparisons, &campaign.validation_signal_gate);
    let aggregate = aggregate_temporal_evidence_v0(campaign, warm_start_status);
    let (
        validation_support_decision,
        test_support_decision,
        validation_support_coverage,
        support_decision_digest,
        earliest_shift_stage,
        temporal_root_cause,
        operational_result,
        counterfactual_result,
    ) = if let Some(temporal) = temporal {
        (
            temporal.validation_support_decision,
            temporal.test_support_decision,
            Some(temporal.validation_support_coverage),
            Some(temporal.decision_digest.clone()),
            temporal.earliest_shift_stage,
            temporal.root_cause,
            if temporal.test_support_decision == ShadowSupportDecisionV0::InSupport {
                "shadow_prediction_research_only".to_string()
            } else {
                "shadow_abstain".to_string()
            },
            if temporal.counterfactual_test_evaluated {
                "research_only_test_evaluated".to_string()
            } else {
                "test_sealed".to_string()
            },
        )
    } else {
        (
            ShadowSupportDecisionV0::InsufficientEvidence,
            ShadowSupportDecisionV0::InsufficientEvidence,
            None,
            None,
            EarliestTemporalShiftStageV0::InsufficientEvidence,
            ProbabilityCollapseRootCauseV0::Unknown,
            "shadow_abstain".to_string(),
            "test_sealed".to_string(),
        )
    };
    let final_verdict =
        final_temporal_verdict_v0(campaign, temporal, warm_start_status, &aggregate);
    let mut reason_codes = campaign.reason_codes.clone();
    reason_codes.push(format!(
        "support={}",
        shadow_support_decision_label_v0(test_support_decision)
    ));
    reason_codes.push(format!(
        "verdict={}",
        temporal_verdict_label_v0(final_verdict)
    ));
    reason_codes.sort();
    reason_codes.dedup();
    let campaign_digest = stable_hash_string(&format!(
        "{}:{}:{}:{}",
        campaign.campaign_id,
        evidence_row_count,
        campaign.windows.len(),
        campaign.reason_codes.join(":"),
    ));
    let report_digest = stable_hash_string(&format!(
        "v1:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        campaign_digest,
        evidence_pack_digest,
        selected
            .map(|value| value.window_id.as_str())
            .unwrap_or("none"),
        selected
            .and_then(|value| value.selected_candidate)
            .map(forensic_candidate_label_v0)
            .unwrap_or("none"),
        shadow_support_decision_label_v0(validation_support_decision),
        shadow_support_decision_label_v0(test_support_decision),
        earliest_stage_label_v0(earliest_shift_stage),
        root_cause_label_v0(temporal_root_cause),
        warm_start_status_label_v0(warm_start_status),
        temporal_verdict_label_v0(final_verdict),
        reason_codes.join(":"),
    ));
    MomentumTemporalDiagnosticReportV0 {
        report_version: "momentum-temporal-diagnostic-v1".to_string(),
        campaign_digest,
        evidence_row_count,
        evidence_pack_digest_prefix: evidence_pack_digest.chars().take(12).collect(),
        selected_window_id: selected.map(|value| value.window_id.clone()),
        selected_candidate: selected.and_then(|value| value.selected_candidate),
        selected_checkpoint: selected.and_then(|value| value.selected_checkpoint.clone()),
        raw_feature_shift: temporal.and_then(|value| value.raw_feature_shift.clone()),
        normalized_feature_shift: temporal.and_then(|value| value.normalized_feature_shift.clone()),
        sequence_shift: temporal.map(|value| value.sequence_shift.clone()),
        frozen_representation_shift: temporal
            .map(|value| value.frozen_representation_shift.clone()),
        representation_scale_shift: temporal.map(|value| value.representation_shift.clone()),
        logit_shift: temporal.map(|value| value.logit_shift.clone()),
        probability_shift: temporal.map(|value| value.probability_shift.clone()),
        outcome_shift: temporal.map(|value| value.outcome_shift.clone()),
        validation_support_decision,
        test_support_decision,
        validation_support_coverage,
        support_decision_digest,
        earliest_shift_stage,
        temporal_root_cause,
        warm_start_status,
        warm_cold_comparisons: comparisons,
        aggregate,
        final_verdict,
        operational_result,
        counterfactual_result,
        layered_eligibility: campaign.safety_trace.eligibility.clone(),
        reason_codes,
        report_digest,
    }
}

fn warm_cold_trajectory_comparisons_v0(
    windows: &[MomentumLearningWindowResultV0],
) -> Vec<WarmColdTrajectoryComparisonV0> {
    windows
        .iter()
        .filter_map(|window| {
            let cold = window
                .paths
                .iter()
                .find(|path| path.path == MomentumLearningPathV0::Cold)?;
            let warm = window
                .paths
                .iter()
                .find(|path| path.path == MomentumLearningPathV0::Warm)?;
            let initial_parameter_distance = (cold
                .initial_head
                .weights
                .iter()
                .zip(&warm.initial_head.weights)
                .map(|(left, right)| (left - right).powi(2))
                .sum::<f32>()
                + (cold.initial_head.bias - warm.initial_head.bias).powi(2))
            .sqrt();
            let comparison_epsilon = 1e-6;
            Some(WarmColdTrajectoryComparisonV0 {
                window_id: window.window.window_id.clone(),
                cold_initial_head_digest: cold.initial_head_digest.clone(),
                warm_initial_head_digest: warm.initial_head_digest.clone(),
                warm_parent_version_id: warm.parent_version_id.clone(),
                initial_parameter_distance,
                initial_bias_difference: (cold.initial_bias - warm.initial_bias).abs(),
                cold_initial_probability_mean: cold.initial_probability_mean,
                warm_initial_probability_mean: warm.initial_probability_mean,
                cold_initial_probability_stddev: cold.initial_probability_stddev,
                warm_initial_probability_stddev: warm.initial_probability_stddev,
                training_prevalence: cold.training_prevalence,
                cold_stopped_epoch: cold.stopped_epoch,
                warm_stopped_epoch: warm.stopped_epoch,
                cold_validation_brier: cold.validation.metrics.brier_score,
                warm_validation_brier: warm.validation.metrics.brier_score,
                cold_validation_probability_stddev: cold.validation.probability_stddev,
                warm_validation_probability_stddev: warm.validation.probability_stddev,
                cold_won_validation: cold.validation.metrics.brier_score + comparison_epsilon
                    < warm.validation.metrics.brier_score,
                warm_won_validation: warm.validation.metrics.brier_score + comparison_epsilon
                    < cold.validation.metrics.brier_score,
            })
        })
        .collect()
}

fn warm_start_lock_in_status_v0(
    comparisons: &[WarmColdTrajectoryComparisonV0],
    gate: &ValidationSignalGateConfigV0,
) -> WarmStartLockInStatusV0 {
    if comparisons.is_empty() {
        return WarmStartLockInStatusV0::InsufficientEvidence;
    }
    if comparisons.iter().all(|comparison| {
        comparison.cold_validation_probability_stddev < gate.minimum_probability_stddev
            && comparison.warm_validation_probability_stddev < gate.minimum_probability_stddev
    }) {
        return WarmStartLockInStatusV0::WarmAndColdBothNoSignal;
    }
    let cold_wins = comparisons
        .iter()
        .filter(|comparison| comparison.cold_won_validation)
        .count();
    let warm_wins = comparisons
        .iter()
        .filter(|comparison| comparison.warm_won_validation)
        .count();
    if cold_wins > 0 && warm_wins == 0 {
        WarmStartLockInStatusV0::ColdBetter
    } else if warm_wins > 0 && cold_wins == 0 {
        WarmStartLockInStatusV0::WarmBetter
    } else if cold_wins == 0 && warm_wins == 0 {
        WarmStartLockInStatusV0::NoLockInEvidence
    } else {
        WarmStartLockInStatusV0::Mixed
    }
}

fn aggregate_temporal_evidence_v0(
    campaign: &MomentumLearningCampaignResultV0,
    warm_start_status: WarmStartLockInStatusV0,
) -> AggregateTemporalGeneralizationEvidenceV0 {
    let temporal = campaign
        .collapse_forensics
        .iter()
        .filter_map(|forensics| forensics.temporal_generalization.as_ref())
        .collect::<Vec<_>>();
    let count_stage = |stage| {
        temporal
            .iter()
            .filter(|result| result.earliest_shift_stage == stage)
            .count()
    };
    let accepted_predictive_versions = campaign
        .collapse_forensics
        .iter()
        .filter(|forensics| {
            forensics
                .temporal_generalization
                .as_ref()
                .is_some_and(|result| {
                    result.test_support_decision == ShadowSupportDecisionV0::InSupport
                })
                && forensics
                    .candidate_results
                    .iter()
                    .find(|candidate| Some(candidate.candidate) == forensics.selected_candidate)
                    .and_then(|candidate| candidate.test.as_ref())
                    .is_some_and(|test| test.brier_score.is_finite())
        })
        .count();
    AggregateTemporalGeneralizationEvidenceV0 {
        total_windows: campaign.collapse_forensics.len(),
        no_signal_windows: campaign
            .collapse_forensics
            .iter()
            .filter(|forensics| forensics.selected_candidate.is_none())
            .count(),
        selected_checkpoint_windows: campaign
            .collapse_forensics
            .iter()
            .filter(|forensics| forensics.selected_checkpoint.is_some())
            .count(),
        support_gate_usable_windows: temporal
            .iter()
            .filter(|result| {
                result.validation_support_decision == ShadowSupportDecisionV0::InSupport
            })
            .count(),
        validation_in_support_windows: temporal
            .iter()
            .filter(|result| {
                result.validation_support_decision == ShadowSupportDecisionV0::InSupport
            })
            .count(),
        validation_out_of_support_windows: temporal
            .iter()
            .filter(|result| {
                result.validation_support_decision == ShadowSupportDecisionV0::OutOfSupport
            })
            .count(),
        validation_insufficient_windows: temporal
            .iter()
            .filter(|result| {
                result.validation_support_decision == ShadowSupportDecisionV0::InsufficientEvidence
            })
            .count(),
        validation_gate_unavailable_windows: temporal
            .iter()
            .filter(|result| {
                result.validation_support_decision
                    == ShadowSupportDecisionV0::SupportGateUnavailable
            })
            .count(),
        in_support_windows: temporal
            .iter()
            .filter(|result| result.test_support_decision == ShadowSupportDecisionV0::InSupport)
            .count(),
        out_of_support_windows: temporal
            .iter()
            .filter(|result| result.test_support_decision == ShadowSupportDecisionV0::OutOfSupport)
            .count(),
        support_gate_unavailable_windows: temporal
            .iter()
            .filter(|result| {
                result.test_support_decision == ShadowSupportDecisionV0::SupportGateUnavailable
            })
            .count(),
        temporal_collapse_windows: campaign
            .collapse_forensics
            .iter()
            .filter(|forensics| {
                forensics.candidate_results.iter().any(|candidate| {
                    candidate
                        .test_collapse
                        .as_ref()
                        .is_some_and(ProbabilityCollapseMetricsV0::is_collapsed)
                })
            })
            .count(),
        raw_feature_shift_windows: temporal
            .iter()
            .filter(|result| {
                result.raw_feature_shift.as_ref().is_some_and(|metrics| {
                    has_material_distribution_shift_v0(metrics, &campaign.support_gate)
                })
            })
            .count(),
        normalized_feature_shift_windows: temporal
            .iter()
            .filter(|result| {
                result
                    .normalized_feature_shift
                    .as_ref()
                    .is_some_and(|metrics| {
                        has_material_distribution_shift_v0(metrics, &campaign.support_gate)
                    })
            })
            .count(),
        sequence_shift_windows: count_stage(EarliestTemporalShiftStageV0::Sequences),
        representation_shift_windows: count_stage(
            EarliestTemporalShiftStageV0::FrozenRepresentations,
        ) + count_stage(
            EarliestTemporalShiftStageV0::RepresentationScale,
        ),
        logit_shift_windows: count_stage(EarliestTemporalShiftStageV0::Logits),
        probability_shift_windows: count_stage(EarliestTemporalShiftStageV0::Probabilities),
        outcomes_only_windows: count_stage(EarliestTemporalShiftStageV0::OutcomesOnly),
        warm_lock_in_windows: usize::from(matches!(
            warm_start_status,
            WarmStartLockInStatusV0::LockInConfirmed | WarmStartLockInStatusV0::LockInSuspected
        )),
        operational_abstentions: campaign
            .collapse_forensics
            .iter()
            .filter(|forensics| forensics.abstention.is_some())
            .count(),
        counterfactual_evaluations: temporal
            .iter()
            .filter(|result| result.counterfactual_test_evaluated)
            .count(),
        accepted_predictive_versions,
    }
}

fn final_temporal_verdict_v0(
    campaign: &MomentumLearningCampaignResultV0,
    temporal: Option<&TemporalGeneralizationResultV0>,
    warm_start_status: WarmStartLockInStatusV0,
    aggregate: &AggregateTemporalGeneralizationEvidenceV0,
) -> SupportGatedMomentumSeriesVerdictV0 {
    if matches!(
        campaign.status,
        MomentumLearningCampaignStatusV0::BackendUnavailable
            | MomentumLearningCampaignStatusV0::LeakageInvariantFailed
            | MomentumLearningCampaignStatusV0::RejectedForSafety
    ) {
        return SupportGatedMomentumSeriesVerdictV0::CampaignFailed;
    }
    if aggregate.selected_checkpoint_windows == 0 {
        return SupportGatedMomentumSeriesVerdictV0::NoUsableValidationSignal;
    }
    if matches!(
        warm_start_status,
        WarmStartLockInStatusV0::LockInConfirmed | WarmStartLockInStatusV0::LockInSuspected
    ) {
        return SupportGatedMomentumSeriesVerdictV0::WarmStartLockInRisk;
    }
    let Some(temporal) = temporal else {
        return SupportGatedMomentumSeriesVerdictV0::InsufficientEvidence;
    };
    if temporal.test_support_decision != ShadowSupportDecisionV0::InSupport {
        return if temporal.earliest_shift_stage
            == EarliestTemporalShiftStageV0::FrozenRepresentations
        {
            SupportGatedMomentumSeriesVerdictV0::FrozenRepresentationShiftRisk
        } else {
            SupportGatedMomentumSeriesVerdictV0::TemporalOutOfSupportAbstention
        };
    }
    let delta_vs_linear = campaign
        .collapse_forensics
        .iter()
        .find(|forensics| {
            forensics
                .temporal_generalization
                .as_ref()
                .is_some_and(|result| result.decision_digest == temporal.decision_digest)
        })
        .and_then(|forensics| {
            campaign
                .windows
                .iter()
                .find(|window| window.window.window_id == forensics.window_id)
        })
        .and_then(|window| {
            window
                .paths
                .iter()
                .find(|path| path.path == MomentumLearningPathV0::Cold)
                .or_else(|| window.paths.first())
        })
        .map(|path| {
            path.baselines.frozen_mamba.brier_score - path.baselines.linear_momentum.brier_score
        });
    match delta_vs_linear {
        Some(delta) if delta < -1e-6 => {
            SupportGatedMomentumSeriesVerdictV0::InSupportUsableSignalAndMambaHelpedOnThisSeries
        }
        Some(delta) if delta > 1e-6 => {
            SupportGatedMomentumSeriesVerdictV0::InSupportUsableSignalButLinearStrongerOnThisSeries
        }
        _ => SupportGatedMomentumSeriesVerdictV0::InSupportMixedEvidence,
    }
}

fn forensic_candidate_label_v0(candidate: MomentumForensicCandidateV0) -> &'static str {
    candidate.label()
}

fn earliest_stage_label_v0(stage: EarliestTemporalShiftStageV0) -> &'static str {
    match stage {
        EarliestTemporalShiftStageV0::None => "none",
        EarliestTemporalShiftStageV0::RawFeatures => "raw_features",
        EarliestTemporalShiftStageV0::NormalizedFeatures => "normalized_features",
        EarliestTemporalShiftStageV0::Sequences => "sequences",
        EarliestTemporalShiftStageV0::FrozenRepresentations => "frozen_representations",
        EarliestTemporalShiftStageV0::RepresentationScale => "representation_scale",
        EarliestTemporalShiftStageV0::Logits => "logits",
        EarliestTemporalShiftStageV0::Probabilities => "probabilities",
        EarliestTemporalShiftStageV0::OutcomesOnly => "outcomes_only",
        EarliestTemporalShiftStageV0::MultipleStages => "multiple_stages",
        EarliestTemporalShiftStageV0::InsufficientEvidence => "insufficient_evidence",
    }
}

fn root_cause_label_v0(cause: ProbabilityCollapseRootCauseV0) -> &'static str {
    match cause {
        ProbabilityCollapseRootCauseV0::RawFeatureCollapse => "raw_feature_collapse",
        ProbabilityCollapseRootCauseV0::NormalizedFeatureCollapse => "normalized_feature_collapse",
        ProbabilityCollapseRootCauseV0::SequenceDiversityCollapse => "sequence_diversity_collapse",
        ProbabilityCollapseRootCauseV0::EncoderRepresentationCollapse => {
            "encoder_representation_collapse"
        }
        ProbabilityCollapseRootCauseV0::RepresentationScaleMismatch => {
            "representation_scale_mismatch"
        }
        ProbabilityCollapseRootCauseV0::FeatureScaleDrift => "feature_scale_drift",
        ProbabilityCollapseRootCauseV0::SequenceSupportBreach => "sequence_support_breach",
        ProbabilityCollapseRootCauseV0::FrozenRepresentationSupportBreach => {
            "frozen_representation_support_breach"
        }
        ProbabilityCollapseRootCauseV0::RepresentationScaleDrift => "representation_scale_drift",
        ProbabilityCollapseRootCauseV0::HeadBiasSensitivity => "head_bias_sensitivity",
        ProbabilityCollapseRootCauseV0::LogitVarianceCollapse => "logit_variance_collapse",
        ProbabilityCollapseRootCauseV0::OutcomePrevalenceShift => "outcome_prevalence_shift",
        ProbabilityCollapseRootCauseV0::ImplementationBug => "implementation_bug",
        ProbabilityCollapseRootCauseV0::HeadInitializationCollapse => {
            "head_initialization_collapse"
        }
        ProbabilityCollapseRootCauseV0::GradientVanishing => "gradient_vanishing",
        ProbabilityCollapseRootCauseV0::GradientExplosion => "gradient_explosion",
        ProbabilityCollapseRootCauseV0::OptimizerInstability => "optimizer_instability",
        ProbabilityCollapseRootCauseV0::BiasDominatedPrediction => "bias_dominated_prediction",
        ProbabilityCollapseRootCauseV0::ValidationCheckpointCollapse => {
            "validation_checkpoint_collapse"
        }
        ProbabilityCollapseRootCauseV0::WarmStartLockIn => "warm_start_lock_in",
        ProbabilityCollapseRootCauseV0::ClassPrevalenceDominance => "class_prevalence_dominance",
        ProbabilityCollapseRootCauseV0::ProbabilitySaturation => "probability_saturation",
        ProbabilityCollapseRootCauseV0::CalibrationOnlyFailure => "calibration_only_failure",
        ProbabilityCollapseRootCauseV0::Mixed => "mixed",
        ProbabilityCollapseRootCauseV0::Unknown => "unknown",
    }
}

fn warm_start_status_label_v0(status: WarmStartLockInStatusV0) -> &'static str {
    match status {
        WarmStartLockInStatusV0::NoLockInEvidence => "no_lock_in_evidence",
        WarmStartLockInStatusV0::LockInSuspected => "lock_in_suspected",
        WarmStartLockInStatusV0::LockInConfirmed => "lock_in_confirmed",
        WarmStartLockInStatusV0::WarmAndColdBothNoSignal => "warm_and_cold_both_no_signal",
        WarmStartLockInStatusV0::WarmBetter => "warm_better",
        WarmStartLockInStatusV0::ColdBetter => "cold_better",
        WarmStartLockInStatusV0::Mixed => "mixed",
        WarmStartLockInStatusV0::InsufficientEvidence => "insufficient_evidence",
    }
}

fn temporal_verdict_label_v0(verdict: SupportGatedMomentumSeriesVerdictV0) -> &'static str {
    match verdict {
        SupportGatedMomentumSeriesVerdictV0::InSupportUsableSignalAndMambaHelpedOnThisSeries => {
            "in_support_mamba_helped"
        }
        SupportGatedMomentumSeriesVerdictV0::InSupportUsableSignalButLinearStrongerOnThisSeries => {
            "in_support_linear_stronger"
        }
        SupportGatedMomentumSeriesVerdictV0::InSupportMixedEvidence => "in_support_mixed_evidence",
        SupportGatedMomentumSeriesVerdictV0::TemporalOutOfSupportAbstention => {
            "temporal_out_of_support_abstention"
        }
        SupportGatedMomentumSeriesVerdictV0::FrozenRepresentationShiftRisk => {
            "frozen_representation_shift_risk"
        }
        SupportGatedMomentumSeriesVerdictV0::WarmStartLockInRisk => "warm_start_lock_in_risk",
        SupportGatedMomentumSeriesVerdictV0::NoUsableValidationSignal => {
            "no_usable_validation_signal"
        }
        SupportGatedMomentumSeriesVerdictV0::InsufficientEvidence => "insufficient_evidence",
        SupportGatedMomentumSeriesVerdictV0::CampaignFailed => "campaign_failed",
    }
}

pub fn momentum_temporal_diagnostic_report_json_v0(
    report: &MomentumTemporalDiagnosticReportV0,
) -> String {
    let shift = |metrics: &Option<DistributionShiftMetricBundleV0>| {
        metrics.as_ref().map(|metrics| {
            serde_json::json!({
                "reference_samples": metrics.sample_count_reference,
                "target_samples": metrics.sample_count_target,
                "dimensions": metrics.dimensions,
                "mean_standardized_shift": metrics.mean_absolute_standardized_mean_shift,
                "max_standardized_shift": metrics.maximum_absolute_standardized_mean_shift,
                "mean_log_variance_ratio": metrics.mean_absolute_log_variance_ratio,
                "max_log_variance_ratio": metrics.maximum_absolute_log_variance_ratio,
                "out_of_support_fraction": metrics.out_of_support_fraction,
                "dimensions_out_of_support": metrics.dimensions_out_of_support,
                "finite": metrics.finite,
            })
        })
    };
    let comparisons = report
        .warm_cold_comparisons
        .iter()
        .map(|comparison| {
            serde_json::json!({
                "window_id": comparison.window_id,
                "cold_initial_head_digest": comparison.cold_initial_head_digest,
                "warm_initial_head_digest": comparison.warm_initial_head_digest,
                "warm_parent_version_id": comparison.warm_parent_version_id,
                "initial_parameter_distance": comparison.initial_parameter_distance,
                "initial_bias_difference": comparison.initial_bias_difference,
                "cold_initial_probability_mean": comparison.cold_initial_probability_mean,
                "warm_initial_probability_mean": comparison.warm_initial_probability_mean,
                "cold_initial_probability_stddev": comparison.cold_initial_probability_stddev,
                "warm_initial_probability_stddev": comparison.warm_initial_probability_stddev,
                "training_prevalence": comparison.training_prevalence,
                "cold_stopped_epoch": comparison.cold_stopped_epoch,
                "warm_stopped_epoch": comparison.warm_stopped_epoch,
                "cold_validation_brier": comparison.cold_validation_brier,
                "warm_validation_brier": comparison.warm_validation_brier,
                "cold_validation_probability_stddev": comparison.cold_validation_probability_stddev,
                "warm_validation_probability_stddev": comparison.warm_validation_probability_stddev,
                "cold_won_validation": comparison.cold_won_validation,
                "warm_won_validation": comparison.warm_won_validation,
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "report_version": report.report_version,
        "campaign_digest": report.campaign_digest,
        "evidence": {
            "row_count": report.evidence_row_count,
            "pack_digest_prefix": report.evidence_pack_digest_prefix,
        },
        "reproduction": {
            "total_windows": report.aggregate.total_windows,
            "no_signal_windows": report.aggregate.no_signal_windows,
            "selected_checkpoint_windows": report.aggregate.selected_checkpoint_windows,
        },
        "selected_path": {
            "window_id": report.selected_window_id,
            "candidate": report.selected_candidate.map(forensic_candidate_label_v0),
            "checkpoint_epoch": report.selected_checkpoint.as_ref().map(|checkpoint| checkpoint.epoch),
        },
        "shift": {
            "raw_feature": shift(&report.raw_feature_shift),
            "normalized_feature": shift(&report.normalized_feature_shift),
            "sequence": shift(&report.sequence_shift),
            "frozen_representation": shift(&report.frozen_representation_shift),
            "representation_scale": shift(&report.representation_scale_shift),
            "logit": shift(&report.logit_shift),
            "probability": shift(&report.probability_shift),
            "outcome": shift(&report.outcome_shift),
            "earliest_stage": earliest_stage_label_v0(report.earliest_shift_stage),
            "root_cause": root_cause_label_v0(report.temporal_root_cause),
        },
        "support": {
            "validation_decision": shadow_support_decision_label_v0(report.validation_support_decision),
            "validation_coverage": report.validation_support_coverage,
            "sealed_test_decision": shadow_support_decision_label_v0(report.test_support_decision),
            "decision_digest": report.support_decision_digest,
        },
        "operational_result": report.operational_result,
        "counterfactual_result": report.counterfactual_result,
        "warm_start": {
            "status": warm_start_status_label_v0(report.warm_start_status),
            "comparisons": comparisons,
        },
        "aggregate": {
            "support_gate_usable_windows": report.aggregate.support_gate_usable_windows,
            "in_support_windows": report.aggregate.in_support_windows,
            "out_of_support_windows": report.aggregate.out_of_support_windows,
            "support_gate_unavailable_windows": report.aggregate.support_gate_unavailable_windows,
            "temporal_collapse_windows": report.aggregate.temporal_collapse_windows,
            "raw_feature_shift_windows": report.aggregate.raw_feature_shift_windows,
            "normalized_feature_shift_windows": report.aggregate.normalized_feature_shift_windows,
            "sequence_shift_windows": report.aggregate.sequence_shift_windows,
            "representation_shift_windows": report.aggregate.representation_shift_windows,
            "logit_shift_windows": report.aggregate.logit_shift_windows,
            "probability_shift_windows": report.aggregate.probability_shift_windows,
            "outcomes_only_windows": report.aggregate.outcomes_only_windows,
            "warm_lock_in_windows": report.aggregate.warm_lock_in_windows,
            "operational_abstentions": report.aggregate.operational_abstentions,
            "counterfactual_evaluations": report.aggregate.counterfactual_evaluations,
            "accepted_predictive_versions": report.aggregate.accepted_predictive_versions,
        },
        "final_verdict": temporal_verdict_label_v0(report.final_verdict),
        "permissions": {
            "offline_shadow_learning": report.layered_eligibility.offline_shadow_learning,
            "promotion": report.layered_eligibility.promotion,
            "voting": report.layered_eligibility.voting,
            "execution": report.layered_eligibility.execution,
        },
        "reason_codes": report.reason_codes,
        "report_digest": report.report_digest,
    })
    .to_string()
}

pub fn momentum_temporal_diagnostic_report_text_v0(
    report: &MomentumTemporalDiagnosticReportV0,
) -> String {
    [
        format!("report_version={}", report.report_version),
        format!("evidence_rows={}", report.evidence_row_count),
        format!("windows={}", report.aggregate.total_windows),
        format!("no_signal_windows={}", report.aggregate.no_signal_windows),
        format!(
            "selected_candidate={}",
            report
                .selected_candidate
                .map(forensic_candidate_label_v0)
                .unwrap_or("none")
        ),
        format!(
            "validation_support={}",
            shadow_support_decision_label_v0(report.validation_support_decision)
        ),
        format!(
            "sealed_test_support={}",
            shadow_support_decision_label_v0(report.test_support_decision)
        ),
        format!(
            "earliest_shift_stage={}",
            earliest_stage_label_v0(report.earliest_shift_stage)
        ),
        format!(
            "temporal_root_cause={}",
            root_cause_label_v0(report.temporal_root_cause)
        ),
        format!(
            "warm_start_status={}",
            warm_start_status_label_v0(report.warm_start_status)
        ),
        format!("operational_result={}", report.operational_result),
        format!("counterfactual_result={}", report.counterfactual_result),
        format!(
            "final_verdict={}",
            temporal_verdict_label_v0(report.final_verdict)
        ),
        format!("report_digest={}", report.report_digest),
    ]
    .join("\n")
}

fn validate_historical_evidence(
    snapshots: &[DataSnapshot],
    safety_trace: &mut CampaignSafetyTraceV0,
) -> Result<ValidatedEvidence, CampaignErrorV0> {
    if snapshots.iter().any(|snapshot| {
        !snapshot.read_only
            || !snapshot.sanitized
            || !snapshot.provenance.sanitized
            || !snapshot.provenance.credential_free
            || snapshot.provenance.provider_id.trim().is_empty()
            || snapshot.provenance.fetch_receipt_id.trim().is_empty()
            || !snapshot.quality_summary.accepted
            || !snapshot
                .reason_codes
                .iter()
                .any(|code| matches!(code, crate::core::ReasonCode::DataSnapshotImmutable))
    }) {
        trace_rejection(
            safety_trace,
            CampaignSafetyGateV0::ImmutableSanitizedEvidence,
            error_label(CampaignErrorV0::MutableEvidence),
            vec![format!("snapshot_count={}", snapshots.len())],
        );
        return Err(CampaignErrorV0::MutableEvidence);
    }
    trace_pass(
        safety_trace,
        CampaignSafetyGateV0::ImmutableSanitizedEvidence,
        vec![format!("snapshot_count={}", snapshots.len())],
    );
    if snapshots.iter().any(|snapshot| {
        !matches!(
            snapshot.dataset_kind,
            DatasetKind::DailyOhlcv | DatasetKind::AdjustedDailyOhlcv
        ) || matches!(snapshot.provenance.source_type, SnapshotSourceType::Mock)
            || unsafe_evidence_text(&format!(
                "{} {} {} {}",
                snapshot.provider_id,
                snapshot.provenance.provider_id,
                snapshot.normalized_dataset.symbol,
                snapshot.normalized_dataset.source
            ))
    }) {
        trace_rejection(
            safety_trace,
            CampaignSafetyGateV0::RealHistoricalEvidence,
            error_label(CampaignErrorV0::UnsafeEvidence),
            vec![format!("snapshot_count={}", snapshots.len())],
        );
        return Err(CampaignErrorV0::UnsafeEvidence);
    }
    trace_pass(
        safety_trace,
        CampaignSafetyGateV0::RealHistoricalEvidence,
        vec!["mock_source=false".to_string()],
    );
    if snapshots.iter().any(|snapshot| {
        historical_replay_dataset_digest_v0(&snapshot.normalized_dataset) != snapshot.content_digest
    }) {
        trace_rejection(
            safety_trace,
            CampaignSafetyGateV0::CanonicalSemanticDigest,
            "canonical_semantic_digest_mismatch",
            vec![
                format!("snapshot_count={}", snapshots.len()),
                "semantic_digest_match=false".to_string(),
            ],
        );
        return Err(CampaignErrorV0::CorruptEvidence);
    }
    trace_pass(
        safety_trace,
        CampaignSafetyGateV0::CanonicalSemanticDigest,
        vec!["semantic_digest_match=true".to_string()],
    );
    let mut candles = Vec::new();
    let mut ids = BTreeSet::new();
    let mut expected_symbol: Option<String> = None;
    let mut prior_timestamp = None;
    for snapshot in snapshots {
        let symbol = snapshot.normalized_dataset.symbol.clone();
        if expected_symbol
            .as_ref()
            .is_some_and(|value| value != &symbol)
        {
            trace_rejection(
                safety_trace,
                CampaignSafetyGateV0::ChronologicalEvidence,
                error_label(CampaignErrorV0::IncompatibleEvidence),
                vec!["single_symbol=false".to_string()],
            );
            return Err(CampaignErrorV0::IncompatibleEvidence);
        }
        expected_symbol = Some(symbol.clone());
        for row in &snapshot.normalized_dataset.rows {
            if row.symbol != symbol
                || unsafe_evidence_text(&row.symbol)
                || prior_timestamp.is_some_and(|prior| row.timestamp_ms < prior)
            {
                trace_rejection(
                    safety_trace,
                    CampaignSafetyGateV0::ChronologicalEvidence,
                    error_label(CampaignErrorV0::NonMonotonicEvidence),
                    vec!["strictly_monotonic=false".to_string()],
                );
                return Err(CampaignErrorV0::NonMonotonicEvidence);
            }
            if prior_timestamp == Some(row.timestamp_ms) {
                trace_rejection(
                    safety_trace,
                    CampaignSafetyGateV0::ChronologicalEvidence,
                    error_label(CampaignErrorV0::DuplicateTimestamp),
                    vec!["duplicate_timestamp=false".to_string()],
                );
                return Err(CampaignErrorV0::DuplicateTimestamp);
            }
            let values = [row.open, row.high, row.low, row.close, row.volume];
            if values.iter().any(|value| !value.is_finite())
                || row.open <= 0.0
                || row.high < row.open.max(row.close)
                || row.low <= 0.0
                || row.low > row.open.min(row.close)
                || row.volume < 0.0
            {
                trace_rejection(
                    safety_trace,
                    CampaignSafetyGateV0::FiniteOhlcvValues,
                    error_label(CampaignErrorV0::UnsafeEvidence),
                    vec!["finite_valid_ohlcv=false".to_string()],
                );
                return Err(CampaignErrorV0::UnsafeEvidence);
            }
            let timestamp = i64::try_from(row.timestamp_ms).map_err(|_| {
                trace_rejection(
                    safety_trace,
                    CampaignSafetyGateV0::FiniteOhlcvValues,
                    error_label(CampaignErrorV0::UnsafeEvidence),
                    vec!["timestamp_representable=false".to_string()],
                );
                CampaignErrorV0::UnsafeEvidence
            })?;
            candles.push(MomentumCandleV0 {
                timestamp,
                open: row.open as f32,
                high: row.high as f32,
                low: row.low as f32,
                close: row.close as f32,
                volume: row.volume as f32,
            });
            prior_timestamp = Some(row.timestamp_ms);
        }
        ids.insert(snapshot.snapshot_id.clone());
    }
    if candles.is_empty() || ids.is_empty() {
        trace_rejection(
            safety_trace,
            CampaignSafetyGateV0::MinimumHistory,
            "insufficient_historical_rows",
            vec!["row_count=0".to_string()],
        );
        return Err(CampaignErrorV0::InsufficientHistory);
    }
    trace_pass(
        safety_trace,
        CampaignSafetyGateV0::ChronologicalEvidence,
        vec!["strictly_monotonic=true".to_string()],
    );
    trace_pass(
        safety_trace,
        CampaignSafetyGateV0::FiniteOhlcvValues,
        vec!["finite_valid_ohlcv=true".to_string()],
    );
    Ok(ValidatedEvidence {
        candles,
        snapshot_ids: ids.into_iter().collect(),
    })
}

fn rows_in_range(
    rows: &[super::MomentumFeatureRowV0],
    range: &IndexRangeV0,
) -> Vec<super::MomentumFeatureRowV0> {
    rows.iter()
        .filter(|row| row.source_index >= range.start && row.source_index < range.end)
        .cloned()
        .collect()
}

fn examples_in_range(
    examples: &[SequenceExampleV0],
    range: &IndexRangeV0,
) -> Vec<SequenceExampleV0> {
    examples
        .iter()
        .filter(|example| example.sequence_start >= range.start && example.label_index < range.end)
        .cloned()
        .collect()
}

fn initial_head_for_path(
    config: &MomentumLearningCampaignConfigV0,
    path: MomentumLearningPathV0,
    window: &MomentumLearningWindowV0,
    train: &[SequenceExampleV0],
    encoder: &FrozenMamba3EncoderV0,
    parent: Option<&WarmParent>,
) -> Result<LogisticPredictionHeadV0, CampaignErrorV0> {
    match path {
        MomentumLearningPathV0::Cold => {
            let dimension = encoder
                .encode_sequence(&train[0].input)?
                .representation
                .len();
            LogisticPredictionHeadV0::seeded(
                dimension,
                deterministic_seed(config.campaign_seed, &window.window_id, path),
            )
            .map_err(Into::into)
        }
        MomentumLearningPathV0::Warm => {
            let parent = parent.ok_or(CampaignErrorV0::InvalidWarmParent)?;
            if parent.window_id == window.window_id
                || parent.version.feature_config_digest != config.feature_config.digest()
                || parent.version.encoder_parameter_digest != encoder.parameter_digest()
                || parent.version.deployment_status != ModelAgentDeploymentStatus::ShadowOnly
                || parent.version.parent_version_id.as_deref()
                    == Some(&parent.version.model_version_id)
                || parent.version.head_parameter_digest != parent.head.parameter_digest()
                || parent.head.weights.len()
                    != encoder
                        .encode_sequence(&train[0].input)?
                        .representation
                        .len()
            {
                return Err(CampaignErrorV0::InvalidWarmParent);
            }
            parent.head.validate()?;
            Ok(parent.head.clone())
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn train_path(
    config: &MomentumLearningCampaignConfigV0,
    path: MomentumLearningPathV0,
    parent_version_id: Option<String>,
    window: &MomentumLearningWindowV0,
    train: &[SequenceExampleV0],
    validation: &[SequenceExampleV0],
    test: &[SequenceExampleV0],
    normalizer_digest: String,
    initial_head: LogisticPredictionHeadV0,
    encoder: &FrozenMamba3EncoderV0,
    encoder_digest: &str,
) -> Result<MomentumLearningPathResultV0, CampaignErrorV0> {
    let initial_head_digest = initial_head.parameter_digest();
    let initial_encoded = encoder.encode_batch(train)?;
    let initial_probabilities = probabilities_for_head(&initial_head, &initial_encoded)?;
    let initial_probability_mean = mean_f32(&initial_probabilities)?;
    let initial_probability_stddev = stddev_f32(&initial_probabilities, initial_probability_mean)?;
    let training_prevalence = mean_f32(
        &train
            .iter()
            .map(|example| example.label)
            .collect::<Vec<_>>(),
    )?;
    let training = train_frozen_mamba_head_v0(
        encoder,
        initial_head.clone(),
        train,
        validation,
        &config.training_config,
    )?;
    if training.encoder_digest_before != *encoder_digest
        || training.encoder_digest_after != *encoder_digest
    {
        return Err(CampaignErrorV0::Learning);
    }
    let train_diagnostics = diagnostics(&training.final_head, encoder, train)?;
    let validation_diagnostics = diagnostics(&training.final_head, encoder, validation)?;
    let test_diagnostics = diagnostics(&training.final_head, encoder, test)?;
    let constant = ConstantProbabilityBaselineV0::fit(train)?;
    let linear = LinearMomentumBaselineV0::train(train, validation, &config.training_config)?;
    let baselines = BaselineComparisonV0 {
        constant_probability: constant.evaluate(test)?,
        linear_momentum: linear.evaluate(test)?,
        frozen_mamba: test_diagnostics.metrics.clone(),
        current_deterministic_momentum_policy: None,
    };
    if baselines.constant_probability.sample_count != test_diagnostics.metrics.sample_count
        || baselines.linear_momentum.sample_count != test_diagnostics.metrics.sample_count
    {
        return Err(CampaignErrorV0::LeakageInvariantFailed);
    }
    let value_status = mamba_representation_value_status_v0(
        &baselines.frozen_mamba,
        &baselines.constant_probability,
        &baselines.linear_momentum,
        config.aggregate_gate.minimum_test_samples,
    );
    let version = SandboxModelVersionV0::new(
        parent_version_id.clone(),
        config.agent_id.clone(),
        config.feature_config.digest(),
        normalizer_digest,
        encoder_digest.to_string(),
        training.final_head.parameter_digest(),
        config.training_config.digest(),
        &window.snapshot_ids,
        window.train_range.clone(),
        window.validation_range.clone(),
        window.test_range.clone(),
        training.backend,
        SandboxModelMetricsV0 {
            train: train_diagnostics.metrics.clone(),
            validation: validation_diagnostics.metrics.clone(),
            test: test_diagnostics.metrics.clone(),
            mamba_value_status: value_status,
        },
    )
    .with_campaign_metadata(
        config.campaign_id.clone(),
        window.window_id.clone(),
        path.label(),
        initial_head_digest.clone(),
        encoder
            .runtime
            .backend_selection
            .fallback_reason
            .map(|reason| format!("{:?}", reason)),
        format!("{:?}", ModelDriftStatusV0::InsufficientEvidence),
        vec![
            "offline_training_complete".to_string(),
            "shadow_only".to_string(),
        ],
    );
    let probability_up = training
        .final_head
        .probability(&encoder.encode_sequence(&test[0].input)?.representation)?;
    let suggested_action = if probability_up >= 0.6 {
        CampaignShadowSuggestedActionV0::UpwardWatch
    } else if probability_up <= 0.4 {
        CampaignShadowSuggestedActionV0::DownwardWatch
    } else {
        CampaignShadowSuggestedActionV0::Abstain
    };
    let shadow_assessment = CampaignShadowAssessmentV0 {
        probability_up,
        confidence: (probability_up - 0.5).abs() * 2.0,
        suggested_action,
        model_version_id: version.model_version_id.clone(),
        evidence_snapshot_ids: window.snapshot_ids.clone(),
        backend: training.backend,
        mathematical_status: ModelMathematicalStatus::ExperimentalInternalReference,
        deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
        eligible_to_vote: false,
        eligible_to_execute: false,
    };
    Ok(MomentumLearningPathResultV0 {
        path,
        parent_version_id,
        initial_head_digest,
        initial_head: initial_head.clone(),
        initial_weight_norm: head_weight_norm(&initial_head),
        initial_bias: initial_head.bias,
        initial_probability_mean,
        initial_probability_stddev,
        training_prevalence,
        final_head_digest: training.final_head.parameter_digest(),
        final_head: training.final_head,
        stopped_epoch: training.stopped_epoch,
        train: train_diagnostics,
        validation: validation_diagnostics,
        test: test_diagnostics,
        baselines,
        frozen_linear_comparator: linear,
        frozen_constant_comparator: constant,
        version,
        shadow_assessment,
    })
}

fn diagnostics(
    head: &LogisticPredictionHeadV0,
    encoder: &FrozenMamba3EncoderV0,
    examples: &[SequenceExampleV0],
) -> Result<PredictionDiagnosticsV0, CampaignErrorV0> {
    let encoded = encoder.encode_batch(examples)?;
    let metrics = evaluate_head_v0(head, &encoded)?;
    let probabilities = encoded
        .iter()
        .map(|example| head.probability(&example.representation))
        .collect::<Result<Vec<_>, _>>()?;
    let mean = probabilities.iter().sum::<f32>() / probabilities.len() as f32;
    let probability_stddev = (probabilities
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f32>()
        / probabilities.len() as f32)
        .sqrt();
    let low_confidence_correct_count = probabilities
        .iter()
        .zip(&encoded)
        .filter(|(probability, example)| {
            let probability = **probability;
            (probability - 0.5).abs() <= 0.1 && ((probability >= 0.5) == (example.label >= 0.5))
        })
        .count();
    Ok(PredictionDiagnosticsV0 {
        metrics,
        probability_stddev,
        minimum_probability: probabilities.iter().copied().fold(f32::INFINITY, f32::min),
        maximum_probability: probabilities
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max),
        low_confidence_correct_count,
    })
}

fn per_window_drift(
    paths: &[MomentumLearningPathResultV0],
    config: &ModelDriftConfigV0,
) -> ModelDriftStatusV0 {
    if paths.is_empty() {
        return ModelDriftStatusV0::InsufficientEvidence;
    }
    if paths
        .iter()
        .any(|path| path.test.probability_stddev < config.probability_stddev_floor)
    {
        ModelDriftStatusV0::ProbabilityCollapse
    } else if paths.len() > 1
        && paths[0].test.metrics.high_confidence_error_count + config.high_confidence_error_increase
            < paths[1].test.metrics.high_confidence_error_count
    {
        ModelDriftStatusV0::OverconfidenceIncrease
    } else {
        ModelDriftStatusV0::Stable
    }
}

fn aggregate_mamba_evidence(
    windows: &[MomentumLearningWindowResultV0],
    gate: &AggregateMambaGateConfigV0,
) -> AggregateMambaValueEvidenceV0 {
    let paths = windows
        .iter()
        .filter_map(|window| {
            window
                .paths
                .iter()
                .find(|path| path.path == MomentumLearningPathV0::Cold)
                .or_else(|| window.paths.first())
        })
        .collect::<Vec<_>>();
    let sufficient = paths
        .iter()
        .filter(|path| path.test.metrics.sample_count >= gate.minimum_test_samples)
        .copied()
        .collect::<Vec<_>>();
    let deltas = sufficient
        .iter()
        .map(|path| path.test.metrics.brier_score - path.baselines.linear_momentum.brier_score)
        .collect::<Vec<_>>();
    let mamba_beats_linear_count = deltas
        .iter()
        .filter(|delta| **delta < -gate.comparison_epsilon)
        .count();
    let linear_beats_mamba_count = deltas
        .iter()
        .filter(|delta| **delta > gate.comparison_epsilon)
        .count();
    let mamba_ties_linear_count =
        deltas.len() - mamba_beats_linear_count - linear_beats_mamba_count;
    let mamba_beats_constant_count = sufficient
        .iter()
        .filter(|path| {
            path.test.metrics.brier_score + gate.comparison_epsilon
                < path.baselines.constant_probability.brier_score
        })
        .count();
    let mean = mean_or_zero(&deltas);
    let median = median_or_zero(deltas.clone());
    let hce_delta = sufficient
        .iter()
        .map(|path| {
            path.test.metrics.high_confidence_error_count as i64
                - path.baselines.linear_momentum.high_confidence_error_count as i64
        })
        .sum();
    let status = if sufficient.len() < gate.minimum_windows {
        MambaRepresentationValueStatusV0::InsufficientEvidence
    } else if mamba_beats_linear_count as f32 / sufficient.len() as f32 >= gate.minimum_win_fraction
        && mean <= gate.maximum_mean_degradation
        && mamba_beats_constant_count >= mamba_beats_linear_count
    {
        MambaRepresentationValueStatusV0::Helped
    } else if linear_beats_mamba_count > mamba_beats_linear_count
        && mean > gate.maximum_mean_degradation
    {
        MambaRepresentationValueStatusV0::Failed
    } else {
        MambaRepresentationValueStatusV0::Mixed
    };
    AggregateMambaValueEvidenceV0 {
        evaluated_windows: paths.len(),
        sufficient_windows: sufficient.len(),
        mamba_beats_constant_count,
        mamba_beats_linear_count,
        linear_beats_mamba_count,
        mamba_ties_linear_count,
        mean_brier_delta_vs_linear: mean,
        median_brier_delta_vs_linear: median,
        high_confidence_error_delta_vs_linear: hce_delta,
        status,
    }
}

fn aggregate_warm_start_evidence(
    windows: &[MomentumLearningWindowResultV0],
    gate: &AggregateMambaGateConfigV0,
) -> Option<WarmStartValueEvidenceV0> {
    let pairs = windows
        .iter()
        .filter_map(|window| {
            let cold = window
                .paths
                .iter()
                .find(|path| path.path == MomentumLearningPathV0::Cold)?;
            let warm = window
                .paths
                .iter()
                .find(|path| path.path == MomentumLearningPathV0::Warm)?;
            Some((cold, warm))
        })
        .collect::<Vec<_>>();
    if pairs.is_empty() {
        return None;
    }
    let epsilon = gate.comparison_epsilon;
    let deltas = pairs
        .iter()
        .map(|(cold, warm)| warm.test.metrics.brier_score - cold.test.metrics.brier_score)
        .collect::<Vec<_>>();
    let warm_beats_cold_count = deltas.iter().filter(|delta| **delta < -epsilon).count();
    let cold_beats_warm_count = deltas.iter().filter(|delta| **delta > epsilon).count();
    let tie_count = deltas.len() - warm_beats_cold_count - cold_beats_warm_count;
    let epoch_deltas = pairs
        .iter()
        .map(|(cold, warm)| warm.stopped_epoch as f32 - cold.stopped_epoch as f32)
        .collect::<Vec<_>>();
    let parameter_deltas = pairs
        .iter()
        .map(|(cold, warm)| {
            (digest_distance(&warm.final_head_digest, &cold.final_head_digest)) as f32
        })
        .collect::<Vec<_>>();
    let status = if pairs.len() < gate.minimum_windows {
        WarmStartValueStatusV0::InsufficientEvidence
    } else if warm_beats_cold_count > cold_beats_warm_count {
        WarmStartValueStatusV0::Helped
    } else if cold_beats_warm_count > warm_beats_cold_count {
        WarmStartValueStatusV0::Failed
    } else {
        WarmStartValueStatusV0::Mixed
    };
    Some(WarmStartValueEvidenceV0 {
        compared_windows: pairs.len(),
        warm_beats_cold_count,
        cold_beats_warm_count,
        tie_count,
        mean_test_brier_delta: mean_or_zero(&deltas),
        mean_convergence_epoch_delta: mean_or_zero(&epoch_deltas),
        mean_parameter_drift_delta: mean_or_zero(&parameter_deltas),
        status,
    })
}

fn aggregate_drift(
    windows: &[MomentumLearningWindowResultV0],
    config: &ModelDriftConfigV0,
) -> ModelDriftStatusV0 {
    let paths = windows
        .iter()
        .flat_map(|window| window.paths.iter())
        .collect::<Vec<_>>();
    if paths.len() < 2 {
        return ModelDriftStatusV0::InsufficientEvidence;
    }
    if paths
        .iter()
        .any(|path| path.test.probability_stddev < config.probability_stddev_floor)
    {
        return ModelDriftStatusV0::ProbabilityCollapse;
    }
    let mut performance = false;
    let mut calibration = false;
    let mut overconfidence = false;
    let mut parameter_instability = false;
    for pair in paths.windows(2) {
        let prior = pair[0];
        let current = pair[1];
        if current.test.metrics.brier_score
            > prior.test.metrics.brier_score + config.brier_degradation_limit
        {
            performance = true;
        }
        if (current.test.metrics.mean_predicted_probability
            - prior.test.metrics.mean_predicted_probability)
            .abs()
            > config.mean_probability_shift_limit
        {
            calibration = true;
        }
        if current.test.metrics.high_confidence_error_count
            > prior.test.metrics.high_confidence_error_count + config.high_confidence_error_increase
        {
            overconfidence = true;
        }
        if (head_weight_norm(&current.final_head) - head_weight_norm(&prior.final_head)).abs()
            > config.head_weight_norm_change_limit
        {
            parameter_instability = true;
        }
    }
    match [
        performance,
        calibration,
        overconfidence,
        parameter_instability,
    ]
    .into_iter()
    .filter(|value| *value)
    .count()
    {
        0 => ModelDriftStatusV0::Stable,
        1 if performance => ModelDriftStatusV0::PerformanceDegradation,
        1 if calibration => ModelDriftStatusV0::CalibrationDrift,
        1 if overconfidence => ModelDriftStatusV0::OverconfidenceIncrease,
        1 => ModelDriftStatusV0::ParameterInstability,
        _ => ModelDriftStatusV0::Mixed,
    }
}

fn campaign_status(
    windows: &[MomentumLearningWindowResultV0],
    evidence: &AggregateMambaValueEvidenceV0,
    drift: ModelDriftStatusV0,
    config: &MomentumLearningCampaignConfigV0,
) -> MomentumLearningCampaignStatusV0 {
    if windows.len() < config.minimum_evaluated_windows
        || evidence.status == MambaRepresentationValueStatusV0::InsufficientEvidence
    {
        MomentumLearningCampaignStatusV0::InsufficientEvidence
    } else if drift != ModelDriftStatusV0::Stable {
        MomentumLearningCampaignStatusV0::DriftDetected
    } else if evidence.status == MambaRepresentationValueStatusV0::Failed {
        MomentumLearningCampaignStatusV0::FailedBaselines
    } else if evidence.status == MambaRepresentationValueStatusV0::Mixed {
        MomentumLearningCampaignStatusV0::Mixed
    } else {
        MomentumLearningCampaignStatusV0::Completed
    }
}

fn empty_result(
    config: &MomentumLearningCampaignConfigV0,
    status: MomentumLearningCampaignStatusV0,
    reason: &str,
) -> MomentumLearningCampaignResultV0 {
    empty_result_with_trace(config, status, reason, CampaignSafetyTraceV0::default())
}

fn empty_result_with_trace(
    config: &MomentumLearningCampaignConfigV0,
    status: MomentumLearningCampaignStatusV0,
    reason: &str,
    safety_trace: CampaignSafetyTraceV0,
) -> MomentumLearningCampaignResultV0 {
    MomentumLearningCampaignResultV0 {
        campaign_id: config.campaign_id.clone(),
        status,
        windows: vec![],
        aggregate_mamba_evidence: AggregateMambaValueEvidenceV0 {
            evaluated_windows: 0,
            sufficient_windows: 0,
            mamba_beats_constant_count: 0,
            mamba_beats_linear_count: 0,
            linear_beats_mamba_count: 0,
            mamba_ties_linear_count: 0,
            mean_brier_delta_vs_linear: 0.0,
            median_brier_delta_vs_linear: 0.0,
            high_confidence_error_delta_vs_linear: 0,
            status: MambaRepresentationValueStatusV0::InsufficientEvidence,
        },
        warm_start_evidence: None,
        aggregate_drift: ModelDriftStatusV0::InsufficientEvidence,
        generated_versions: vec![],
        shadow_assessments: vec![],
        rejected_windows: vec![],
        reason_codes: vec![reason.to_string(), "offline_shadow_only".to_string()],
        safety_trace,
        collapse_forensics: vec![],
        validation_signal_gate: config.validation_signal_gate.clone(),
        support_gate: config.support_gate.clone(),
    }
}

const CAMPAIGN_SAFETY_GATE_ORDER: [CampaignSafetyGateV0; 14] = [
    CampaignSafetyGateV0::ImmutableSanitizedEvidence,
    CampaignSafetyGateV0::RealHistoricalEvidence,
    CampaignSafetyGateV0::CanonicalSemanticDigest,
    CampaignSafetyGateV0::ChronologicalEvidence,
    CampaignSafetyGateV0::FiniteOhlcvValues,
    CampaignSafetyGateV0::MinimumHistory,
    CampaignSafetyGateV0::PurgedChronologicalWindows,
    CampaignSafetyGateV0::CpuFullInferenceReady,
    CampaignSafetyGateV0::FrozenEncoderCaptured,
    CampaignSafetyGateV0::OfflineShadowLearning,
    CampaignSafetyGateV0::PromotionEligibility,
    CampaignSafetyGateV0::VotingEligibility,
    CampaignSafetyGateV0::ExecutionEligibility,
    CampaignSafetyGateV0::FrozenEncoderUnchanged,
];

fn trace_pass(
    trace: &mut CampaignSafetyTraceV0,
    gate: CampaignSafetyGateV0,
    sanitized_facts: Vec<String>,
) {
    trace.gates.push(CampaignSafetyGateEvaluationV0 {
        gate,
        outcome: CampaignSafetyGateOutcomeV0::Passed,
        reason_code: None,
        sanitized_facts,
    });
}

fn trace_rejection(
    trace: &mut CampaignSafetyTraceV0,
    gate: CampaignSafetyGateV0,
    reason_code: &str,
    sanitized_facts: Vec<String>,
) {
    trace.gates.push(CampaignSafetyGateEvaluationV0 {
        gate,
        outcome: CampaignSafetyGateOutcomeV0::Rejected,
        reason_code: Some(reason_code.to_string()),
        sanitized_facts,
    });
    if trace.first_rejecting_gate.is_none() {
        trace.first_rejecting_gate = Some(gate);
        trace.first_reason_code = Some(reason_code.to_string());
    }
}

fn trace_blocked(trace: &mut CampaignSafetyTraceV0, gate: CampaignSafetyGateV0, reason_code: &str) {
    trace.gates.push(CampaignSafetyGateEvaluationV0 {
        gate,
        outcome: CampaignSafetyGateOutcomeV0::Blocked,
        reason_code: Some(reason_code.to_string()),
        sanitized_facts: vec![],
    });
}

fn complete_safety_trace(mut trace: CampaignSafetyTraceV0) -> CampaignSafetyTraceV0 {
    for gate in CAMPAIGN_SAFETY_GATE_ORDER {
        if !trace.gates.iter().any(|evaluation| evaluation.gate == gate) {
            trace.gates.push(CampaignSafetyGateEvaluationV0 {
                gate,
                outcome: CampaignSafetyGateOutcomeV0::NotEvaluatedAfterEarlierRejection,
                reason_code: None,
                sanitized_facts: vec![],
            });
        }
    }
    trace.gates.sort_by_key(|evaluation| {
        CAMPAIGN_SAFETY_GATE_ORDER
            .iter()
            .position(|gate| *gate == evaluation.gate)
            .unwrap_or(CAMPAIGN_SAFETY_GATE_ORDER.len())
    });
    trace
}

fn deterministic_seed(base: u64, window_id: &str, path: MomentumLearningPathV0) -> u64 {
    window_id.bytes().fold(
        base ^ if path == MomentumLearningPathV0::Cold {
            0xC01D
        } else {
            0xA11C
        },
        |state, byte| {
            state
                .wrapping_mul(1_099_511_628_211)
                .wrapping_add(byte as u64)
        },
    )
}

fn backend_is_permitted(
    config: &MomentumLearningCampaignConfigV0,
    encoder: &FrozenMamba3EncoderV0,
) -> bool {
    if encoder.runtime.backend_selection.selected != Mamba3BackendKind::CpuReference {
        return false;
    }
    matches!(
        config.backend_preference,
        BackendPreference::Auto | BackendPreference::Cpu
    ) || config.fallback_policy == BackendFallbackPolicy::AllowCpuFallback
}

fn unsafe_evidence_text(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    [
        "authorization",
        "bearer",
        "account",
        "order",
        "api_key",
        "secret",
        "token",
        "raw_response",
        "http://",
        "https://",
        "ws://",
        "wss://",
    ]
    .iter()
    .any(|marker| lowered.contains(marker))
}

fn mean_or_zero(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

fn median_or_zero(mut values: Vec<f32>) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f32::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    }
}

fn digest_distance(left: &str, right: &str) -> usize {
    left.bytes()
        .zip(right.bytes())
        .filter(|(a, b)| a != b)
        .count()
        + left.len().abs_diff(right.len())
}

fn head_weight_norm(head: &LogisticPredictionHeadV0) -> f32 {
    (head.weights.iter().map(|value| value * value).sum::<f32>() + head.bias * head.bias).sqrt()
}

fn error_label(error: CampaignErrorV0) -> &'static str {
    match error {
        CampaignErrorV0::UnsafeEvidence => "unsafe_historical_evidence",
        CampaignErrorV0::MutableEvidence => "mutable_historical_evidence",
        CampaignErrorV0::CorruptEvidence => "corrupt_historical_evidence",
        CampaignErrorV0::NonMonotonicEvidence => "non_monotonic_historical_evidence",
        CampaignErrorV0::DuplicateTimestamp => "duplicate_historical_timestamp",
        CampaignErrorV0::IncompatibleEvidence => "incompatible_historical_evidence",
        _ => "historical_evidence_rejected",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::ReasonCode,
        data::{AcquisitionMarketScope, DataLookback, SnapshotProvenance, SnapshotQualitySummary},
        league::{HistoricalOhlcvRow, HistoricalReplayDataset},
        model::{
            AgentModelRuntimeV0, Mamba3SisoConfigV0, Mamba3SisoPrecisionV0,
            Mamba3SisoRopeFractionV0, SequencePooling, SystemBackendCapabilityProbe,
            TinyMamba3SisoV0, mamba3_siso_params_from_seed_v0,
        },
    };

    fn snapshot(rows: usize) -> DataSnapshot {
        let dataset = HistoricalReplayDataset {
            symbol: "SOMA".to_string(),
            source: "sanitized-local-replay".to_string(),
            rows: (0..rows)
                .map(|index| {
                    let close =
                        100.0 + index as f64 * 0.2 + if index % 5 == 0 { -0.15 } else { 0.1 };
                    HistoricalOhlcvRow {
                        symbol: "SOMA".to_string(),
                        timestamp_ms: index as u64 + 1,
                        open: close - 0.1,
                        high: close + 0.2,
                        low: close - 0.2,
                        close,
                        volume: 1_000.0 + (index % 7) as f64 * 10.0,
                        trade_value: None,
                    }
                })
                .collect(),
            reason_codes: vec![],
        };
        let content_digest = historical_replay_dataset_digest_v0(&dataset);
        DataSnapshot {
            snapshot_id: "snapshot-soma".to_string(),
            request_key: "daily:SOMA".to_string(),
            provider_id: "local-replay".to_string(),
            dataset_kind: DatasetKind::DailyOhlcv,
            market_scope: AcquisitionMarketScope::UsStocks,
            symbols: vec!["SOMA".to_string()],
            requested_lookback: DataLookback {
                bars: rows,
                start_timestamp_ms: Some(1),
                end_timestamp_ms: Some(rows as u64),
            },
            actual_start_timestamp_ms: Some(1),
            actual_end_timestamp_ms: Some(rows as u64),
            fetched_at_ms: rows as u64,
            normalized_at_ms: rows as u64,
            schema_version: 1,
            row_count: rows,
            quality_summary: SnapshotQualitySummary {
                accepted: true,
                row_count: rows,
                reason_codes: vec![],
            },
            content_digest,
            sanitized: true,
            read_only: true,
            compatibility: None,
            normalized_dataset: dataset,
            provenance: SnapshotProvenance {
                provider_id: "local-replay".to_string(),
                acquisition_request_id: "request-soma".to_string(),
                fetch_receipt_id: "receipt-soma".to_string(),
                source_type: SnapshotSourceType::LocalSnapshotReplay,
                sanitized: true,
                credential_free: true,
                reason_codes: vec![],
            },
            reason_codes: vec![ReasonCode::DataSnapshotImmutable],
        }
    }

    fn encoder() -> FrozenMamba3EncoderV0 {
        let config = Mamba3SisoConfigV0 {
            input_dim: 6,
            state_dim: 8,
            head_dim: 2,
            expansion: 1,
            rope_fraction: Mamba3SisoRopeFractionV0::Half,
            norm_epsilon: 1e-5,
            a_floor: 1e-4,
            mimo_rank: 1,
            precision: Mamba3SisoPrecisionV0::F32,
            short_convolution_enabled: false,
        };
        FrozenMamba3EncoderV0 {
            model: TinyMamba3SisoV0::new(
                config.clone(),
                mamba3_siso_params_from_seed_v0(&config, 97).unwrap(),
            )
            .unwrap(),
            runtime: AgentModelRuntimeV0::select(
                &SystemBackendCapabilityProbe,
                BackendPreference::Auto,
                BackendFallbackPolicy::AllowCpuFallback,
            )
            .unwrap(),
            pooling: SequencePooling::LastOutput,
        }
    }

    #[test]
    fn expanding_windows_have_two_purged_boundaries() {
        let config = MomentumLearningCampaignConfigV0 {
            minimum_history_rows: 40,
            train_rows: 18,
            validation_rows: 8,
            test_rows: 8,
            step_rows: 8,
            purge_gap_rows: 8,
            sequence_config: MomentumSequenceConfigV0 {
                sequence_length: 8,
                prediction_horizon: 1,
                ..MomentumSequenceConfigV0::default()
            },
            ..MomentumLearningCampaignConfigV0::default()
        };
        let windows =
            build_momentum_learning_windows_v0(&config, 80, &["snapshot-a".to_string()]).unwrap();
        assert!(windows.len() > 1);
        for window in windows {
            assert!(
                window.train_range.end + config.purge_gap_rows <= window.validation_range.start
            );
            assert!(window.validation_range.end + config.purge_gap_rows <= window.test_range.start);
        }
    }

    #[test]
    fn unsafe_or_mock_evidence_is_rejected_without_campaign_results() {
        let config = MomentumLearningCampaignConfigV0::default();
        let result = empty_result(
            &config,
            MomentumLearningCampaignStatusV0::RejectedForSafety,
            "unsafe_historical_evidence",
        );
        assert_eq!(
            result.status,
            MomentumLearningCampaignStatusV0::RejectedForSafety
        );
        assert!(result.generated_versions.is_empty());
        assert!(result.shadow_assessments.is_empty());
    }

    #[test]
    fn canonical_digest_rejection_records_the_first_safety_gate() {
        let config = MomentumLearningCampaignConfigV0::default();
        let mut invalid_snapshot = snapshot(128);
        invalid_snapshot.content_digest = "invalid-semantic-digest".to_string();

        let result =
            run_momentum_learning_campaign_v0(&config, &[invalid_snapshot], &encoder()).unwrap();

        assert_eq!(
            result.status,
            MomentumLearningCampaignStatusV0::RejectedForSafety
        );
        assert_eq!(
            result.safety_trace.first_rejecting_gate,
            Some(CampaignSafetyGateV0::CanonicalSemanticDigest)
        );
        assert_eq!(
            result.safety_trace.first_reason_code.as_deref(),
            Some("canonical_semantic_digest_mismatch")
        );
        assert!(result.generated_versions.is_empty());
    }

    #[test]
    fn probability_collapse_contract_detects_constant_without_hardcoding_outcome() {
        let config = ProbabilityCollapseConfigV0::default();
        let collapsed =
            probability_collapse_metrics_v0(&[0.5, 0.5, 0.5, 0.5], &[0.0, 1.0, 0.0, 1.0], &config)
                .unwrap();
        assert!(collapsed.is_collapsed());
        assert!(
            collapsed
                .subtypes
                .contains(&ProbabilityCollapseSubtypeV0::NearConstantProbability)
        );

        let diverse =
            probability_collapse_metrics_v0(&[0.1, 0.3, 0.7, 0.9], &[0.0, 0.0, 1.0, 1.0], &config)
                .unwrap();
        assert!(!diverse.is_collapsed());
    }

    #[test]
    fn aggregate_gate_does_not_allow_one_winning_window_to_help() {
        let gate = AggregateMambaGateConfigV0 {
            minimum_windows: 2,
            ..AggregateMambaGateConfigV0::default()
        };
        let empty = aggregate_mamba_evidence(&[], &gate);
        assert_eq!(
            empty.status,
            MambaRepresentationValueStatusV0::InsufficientEvidence
        );
    }

    #[test]
    fn campaign_creates_only_shadow_versions_and_assessments() {
        let config = MomentumLearningCampaignConfigV0 {
            minimum_history_rows: 140,
            train_rows: 56,
            validation_rows: 24,
            test_rows: 24,
            step_rows: 20,
            purge_gap_rows: 8,
            minimum_test_samples: 4,
            minimum_evaluated_windows: 2,
            training_config: HeadTrainingConfigV0 {
                epochs: 2,
                batch_size: 8,
                ..HeadTrainingConfigV0::default()
            },
            aggregate_gate: AggregateMambaGateConfigV0 {
                minimum_windows: 2,
                minimum_test_samples: 4,
                ..AggregateMambaGateConfigV0::default()
            },
            ..MomentumLearningCampaignConfigV0::default()
        };
        let result =
            run_momentum_learning_campaign_v0(&config, &[snapshot(180)], &encoder()).unwrap();
        assert!(result.windows.len() >= 2);
        assert!(result.generated_versions.iter().all(|version| {
            version.deployment_status == ModelAgentDeploymentStatus::ShadowOnly
                && version.campaign_id.as_deref() == Some(config.campaign_id.as_str())
                && version.window_id.is_some()
                && version.initial_head_parameter_digest.is_some()
        }));
        assert!(result.shadow_assessments.iter().all(|assessment| {
            !assessment.eligible_to_vote
                && !assessment.eligible_to_execute
                && assessment.deployment_status == ModelAgentDeploymentStatus::ShadowOnly
        }));
        assert!(
            result
                .rejected_windows
                .iter()
                .any(|rejected| rejected.path == Some(MomentumLearningPathV0::Warm))
        );
        assert_eq!(result.safety_trace.first_rejecting_gate, None);
        assert!(result.safety_trace.eligibility.offline_shadow_learning);
        assert!(!result.safety_trace.eligibility.promotion);
        assert!(!result.safety_trace.eligibility.voting);
        assert!(!result.safety_trace.eligibility.execution);
        assert!(result.safety_trace.gates.iter().any(|gate| {
            gate.gate == CampaignSafetyGateV0::CanonicalSemanticDigest
                && gate.outcome == CampaignSafetyGateOutcomeV0::Passed
        }));
        assert!(result.collapse_forensics.iter().all(|forensics| {
            forensics.test_partition_opened_once == forensics.selected_checkpoint.is_some()
                && forensics
                    .candidate_results
                    .iter()
                    .filter(|candidate| candidate.test.is_some())
                    .count()
                    <= 1
                && forensics.abstention.as_ref().is_none_or(|abstention| {
                    !abstention.eligible_to_vote
                        && !abstention.eligible_to_execute
                        && !abstention.eligible_for_promotion
                })
        }));
    }

    #[test]
    fn checkpoint_gate_rejects_constant_predictions_and_keeps_test_sealed() {
        let collapse = ProbabilityCollapseConfigV0::default();
        let gate = ValidationSignalGateConfigV0::default();
        let metrics =
            probability_collapse_metrics_v0(&[0.5; 4], &[0.0, 1.0, 0.0, 1.0], &collapse).unwrap();
        assert!(metrics.is_collapsed());
        let decomposition = brier_decomposition_v0(&[0.5; 4], &[0.0, 1.0, 0.0, 1.0], 5).unwrap();
        assert!(decomposition.resolution.abs() < 1e-6);
        assert!(gate.validate().is_ok());
    }

    #[test]
    fn rank_auc_is_deterministic_for_ties_and_single_class_is_explicit() {
        let tied = binary_rank_auc_v0(&[0.2, 0.2, 0.8, 0.8], &[0.0, 1.0, 0.0, 1.0]).unwrap();
        assert_eq!(tied.status, BinaryRankAucStatusV0::Defined);
        assert_eq!(tied.value, Some(0.5));
        let single = binary_rank_auc_v0(&[0.2, 0.8], &[1.0, 1.0]).unwrap();
        assert_eq!(single.status, BinaryRankAucStatusV0::UndefinedSingleClass);
    }

    #[test]
    fn train_fitted_support_envelope_detects_shift_without_labels() {
        let gate = ShadowSupportGateConfigV0::default();
        let train = vec![
            vec![0.0, 1.0],
            vec![0.1, 1.1],
            vec![-0.1, 0.9],
            vec![0.0, 1.0],
        ];
        let envelope = DistributionSupportEnvelopeV0::fit(&train, &gate).unwrap();
        let stable = distribution_shift_metrics_v0(&train, &train, &envelope).unwrap();
        assert_eq!(
            support_decision(&stable, &gate, true),
            ShadowSupportDecisionV0::InSupport
        );
        let shifted = vec![
            vec![10.0, 1.0],
            vec![11.0, 1.1],
            vec![9.0, 0.9],
            vec![10.0, 1.0],
        ];
        let metrics = distribution_shift_metrics_v0(&train, &shifted, &envelope).unwrap();
        assert_eq!(
            support_decision(&metrics, &gate, false),
            ShadowSupportDecisionV0::OutOfSupport
        );
    }

    #[test]
    fn validation_coverage_breach_is_rejection_not_gate_unavailability() {
        let gate = ShadowSupportGateConfigV0::default();
        let metrics = DistributionShiftMetricBundleV0 {
            sample_count_reference: gate.minimum_samples,
            sample_count_target: gate.minimum_samples,
            dimensions: 2,
            mean_absolute_standardized_mean_shift: 0.0,
            maximum_absolute_standardized_mean_shift: 0.0,
            mean_absolute_log_variance_ratio: 0.0,
            maximum_absolute_log_variance_ratio: 0.0,
            out_of_support_fraction: 0.25,
            dimensions_out_of_support: 1,
            finite: true,
        };
        assert_eq!(
            support_gate_applicability_v0(&metrics, &gate),
            SupportGateApplicabilityStatusV0::Applicable
        );
        assert_eq!(
            support_decision(&metrics, &gate, true),
            ShadowSupportDecisionV0::OutOfSupport
        );
        let trace = support_metric_evaluations_v0(&metrics, &gate);
        assert_eq!(trace[1].metric_id, SupportMetricIdV0::ValidationCoverage);
        assert_eq!(trace[1].decision, SupportMetricDecisionV0::Breached);
    }

    #[test]
    fn train_history_audit_uses_fixed_label_free_chronological_folds() {
        let gate = ShadowSupportGateConfigV0::default();
        let rows = vec![vec![1.0, -1.0]; 10];
        let audit = train_history_support_audit_v0(&rows, &gate);
        assert_eq!(audit.fixed_chronological_fold_count, 2);
        assert_eq!(audit.in_support_fold_count, 2);
        assert_eq!(audit.out_of_support_fold_count, 0);
        assert_eq!(
            audit.status,
            TrainHistorySupportAuditStatusV0::SelfConsistent
        );
    }

    #[test]
    fn earliest_temporal_stage_uses_data_driven_precedence() {
        let gate = ShadowSupportGateConfigV0::default();
        let stable = DistributionShiftMetricBundleV0 {
            sample_count_reference: 4,
            sample_count_target: 4,
            dimensions: 1,
            mean_absolute_standardized_mean_shift: 0.0,
            maximum_absolute_standardized_mean_shift: 0.0,
            mean_absolute_log_variance_ratio: 0.0,
            maximum_absolute_log_variance_ratio: 0.0,
            out_of_support_fraction: 0.0,
            dimensions_out_of_support: 0,
            finite: true,
        };
        let shifted = DistributionShiftMetricBundleV0 {
            mean_absolute_standardized_mean_shift: gate.maximum_mean_standardized_shift + 1.0,
            ..stable.clone()
        };
        assert_eq!(
            earliest_temporal_shift_stage_v0(
                Some(&shifted),
                Some(&shifted),
                &shifted,
                &shifted,
                &shifted,
                &shifted,
                &shifted,
                &shifted,
                &gate,
            ),
            EarliestTemporalShiftStageV0::RawFeatures
        );
        assert_eq!(
            earliest_temporal_shift_stage_v0(
                None, None, &stable, &stable, &stable, &stable, &stable, &shifted, &gate,
            ),
            EarliestTemporalShiftStageV0::OutcomesOnly
        );
    }

    #[test]
    fn temporal_report_is_deterministic_and_redacted() {
        let config = MomentumLearningCampaignConfigV0 {
            minimum_history_rows: 140,
            train_rows: 56,
            validation_rows: 24,
            test_rows: 24,
            step_rows: 20,
            purge_gap_rows: 8,
            minimum_test_samples: 4,
            minimum_evaluated_windows: 2,
            training_config: HeadTrainingConfigV0 {
                epochs: 2,
                batch_size: 8,
                ..HeadTrainingConfigV0::default()
            },
            aggregate_gate: AggregateMambaGateConfigV0 {
                minimum_windows: 2,
                minimum_test_samples: 4,
                ..AggregateMambaGateConfigV0::default()
            },
            ..MomentumLearningCampaignConfigV0::default()
        };
        let campaign =
            run_momentum_learning_campaign_v0(&config, &[snapshot(180)], &encoder()).unwrap();
        let first = build_momentum_temporal_diagnostic_report_v0(&campaign, 180, "pack-digest");
        let second = build_momentum_temporal_diagnostic_report_v0(&campaign, 180, "pack-digest");
        let first_json = momentum_temporal_diagnostic_report_json_v0(&first);
        assert_eq!(first.report_digest, second.report_digest);
        assert_eq!(
            first_json,
            momentum_temporal_diagnostic_report_json_v0(&second)
        );
        assert!(!first_json.contains("/Users/"));
        assert!(!first_json.contains("\"weights\""));
        assert!(first_json.contains("sealed_test_decision"));
    }
}
