//! Offline, walk-forward learning orchestration for the shadow momentum model.
//!
//! The campaign owns evidence validation, chronological partitioning, and immutable
//! result records. It never joins the active committee or submits an order.

use std::collections::BTreeSet;

use crate::{
    core::stable_hash_string,
    data::{DataSnapshot, DatasetKind, SnapshotSourceType},
};

use super::{
    BackendFallbackPolicy, BackendPreference, BaselineComparisonV0, ConstantProbabilityBaselineV0,
    EvaluationMetricsV0, FeatureNormalizerV0, FrozenMamba3EncoderV0, HeadTrainingConfigV0,
    IndexRangeV0, LearningError, LinearMomentumBaselineV0, LogisticPredictionHeadV0,
    Mamba3BackendKind, MambaRepresentationValueStatusV0, ModelAgentDeploymentStatus,
    ModelMathematicalStatus, MomentumCandleV0, MomentumFeatureConfigV0, MomentumSequenceConfigV0,
    SandboxModelMetricsV0, SandboxModelVersionJournalV0, SandboxModelVersionV0, SequenceExampleV0,
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

    fn digest(&self) -> String {
        stable_hash_string(&format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}:{:?}:{}:{}:{}:{:?}:{:?}",
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub final_head_digest: String,
    pub final_head: LogisticPredictionHeadV0,
    pub stopped_epoch: usize,
    pub train: PredictionDiagnosticsV0,
    pub validation: PredictionDiagnosticsV0,
    pub test: PredictionDiagnosticsV0,
    pub baselines: BaselineComparisonV0,
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

pub fn run_momentum_learning_campaign_v0(
    config: &MomentumLearningCampaignConfigV0,
    snapshots: &[DataSnapshot],
    encoder: &FrozenMamba3EncoderV0,
) -> Result<MomentumLearningCampaignResultV0, CampaignErrorV0> {
    config.validate()?;
    if snapshots.is_empty() {
        return Ok(empty_result(
            config,
            MomentumLearningCampaignStatusV0::NoHistoricalLearningEvidence,
            "no_historical_learning_evidence",
        ));
    }
    let evidence = match validate_historical_evidence(snapshots) {
        Ok(evidence) => evidence,
        Err(error) => {
            return Ok(empty_result(
                config,
                MomentumLearningCampaignStatusV0::RejectedForSafety,
                error_label(error),
            ));
        }
    };
    if evidence.candles.len() < config.minimum_history_rows {
        return Ok(empty_result(
            config,
            MomentumLearningCampaignStatusV0::NoHistoricalLearningEvidence,
            "insufficient_historical_rows",
        ));
    }
    if !backend_is_permitted(config, encoder) {
        return Ok(empty_result(
            config,
            MomentumLearningCampaignStatusV0::BackendUnavailable,
            "backend_not_full_ready_cpu",
        ));
    }
    let windows =
        build_momentum_learning_windows_v0(config, evidence.candles.len(), &evidence.snapshot_ids)?;
    let raw_features = build_momentum_features_v0(&evidence.candles, &config.feature_config)?;
    let encoder_digest = encoder.parameter_digest();
    let mut journal = SandboxModelVersionJournalV0::default();
    let mut results = Vec::new();
    let mut rejected_windows = Vec::new();
    let mut last_parent: Option<WarmParent> = None;

    for window in windows {
        let train_rows = rows_in_range(&raw_features, &window.train_range);
        let normalizer = FeatureNormalizerV0::fit(&train_rows)?;
        let normalized = normalizer.transform(&raw_features)?;
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
    })
}

fn validate_historical_evidence(
    snapshots: &[DataSnapshot],
) -> Result<ValidatedEvidence, CampaignErrorV0> {
    let mut candles = Vec::new();
    let mut ids = BTreeSet::new();
    let mut expected_symbol: Option<String> = None;
    let mut prior_timestamp = None;
    for snapshot in snapshots {
        if !snapshot.read_only
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
        {
            return Err(CampaignErrorV0::MutableEvidence);
        }
        if !matches!(
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
        {
            return Err(CampaignErrorV0::UnsafeEvidence);
        }
        let digest = serde_json::to_string(&snapshot.normalized_dataset)
            .map(|text| stable_hash_string(&text))
            .map_err(|_| CampaignErrorV0::CorruptEvidence)?;
        if digest != snapshot.content_digest {
            return Err(CampaignErrorV0::CorruptEvidence);
        }
        let symbol = snapshot.normalized_dataset.symbol.clone();
        if expected_symbol
            .as_ref()
            .is_some_and(|value| value != &symbol)
        {
            return Err(CampaignErrorV0::IncompatibleEvidence);
        }
        expected_symbol = Some(symbol.clone());
        for row in &snapshot.normalized_dataset.rows {
            if row.symbol != symbol
                || unsafe_evidence_text(&row.symbol)
                || prior_timestamp.is_some_and(|prior| row.timestamp_ms < prior)
            {
                return Err(CampaignErrorV0::NonMonotonicEvidence);
            }
            if prior_timestamp == Some(row.timestamp_ms) {
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
                return Err(CampaignErrorV0::UnsafeEvidence);
            }
            let timestamp =
                i64::try_from(row.timestamp_ms).map_err(|_| CampaignErrorV0::UnsafeEvidence)?;
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
        return Err(CampaignErrorV0::InsufficientHistory);
    }
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
    let training = train_frozen_mamba_head_v0(
        encoder,
        initial_head,
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
        final_head_digest: training.final_head.parameter_digest(),
        final_head: training.final_head,
        stopped_epoch: training.stopped_epoch,
        train: train_diagnostics,
        validation: validation_diagnostics,
        test: test_diagnostics,
        baselines,
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
    }
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
        let content_digest = stable_hash_string(&serde_json::to_string(&dataset).unwrap());
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
    }
}
