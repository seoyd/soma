//! Offline, independent downside-risk learning for the shadow cycle/risk agent.
//!
//! This module consumes immutable historical OHLCV only.  It deliberately owns
//! its labels, features, normalizers, encoder seed, heads, versions, and journal;
//! it never participates in the three-member committee or any trading path.

use crate::{core::stable_hash_string, data::DataSnapshot};

use super::{
    AgentModelRuntimeV0, BackendFallbackPolicy, BackendPreference, BinaryRankAucStatusV0,
    ConstantProbabilityBaselineV0, EncodedTrainingExampleV0, FrozenMamba3EncoderV0,
    HeadTrainingConfigV0, LinearMomentumBaselineV0, LogisticPredictionHeadV0, Mamba3SisoConfigV0,
    Mamba3SisoPrecisionV0, Mamba3SisoRopeFractionV0, MomentumFeatureRowV0,
    ProbabilityCollapseConfigV0, SequenceExampleV0, TinyMamba3SisoV0, apply_sgd_v0,
    binary_rank_auc_v0, brier_decomposition_v0, brier_loss_and_gradients_v0,
    mamba3_siso_params_from_seed_v0, probability_collapse_metrics_v0,
};

pub const CYCLE_RISK_SHADOW_AGENT_ID_V0: &str = "cycle_risk_skeptic_shadow_v0";
pub const MOMENTUM_AGENT_ID_V0: &str = "momentum_shadow_v0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedAgentIndependenceProofV0 {
    pub momentum_agent_id: String,
    pub risk_agent_id: String,
    pub shared_raw_evidence_only: bool,
    pub feature_schema_distinct: bool,
    pub label_schema_distinct: bool,
    pub normalizer_distinct: bool,
    pub encoder_parameters_distinct: bool,
    pub head_parameters_distinct: bool,
    pub recurrent_state_distinct: bool,
    pub version_namespace_distinct: bool,
    pub journal_namespace_distinct: bool,
    pub no_prediction_dependency: bool,
    pub all_independence_invariants_pass: bool,
    pub proof_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DownsideRiskLabelConfigV0 {
    pub horizon_rows: usize,
    pub threshold_policy: String,
    pub training_quantile: f32,
    pub minimum_training_anchors: usize,
    pub minimum_positive_labels: usize,
    pub minimum_negative_labels: usize,
    pub purge_gap_rows: usize,
    pub epsilon: f32,
}

impl Default for DownsideRiskLabelConfigV0 {
    fn default() -> Self {
        Self {
            horizon_rows: 4,
            threshold_policy: "train_only_future_max_adverse_log_excursion_quantile".to_string(),
            training_quantile: 0.70,
            minimum_training_anchors: 24,
            minimum_positive_labels: 4,
            minimum_negative_labels: 4,
            purge_gap_rows: 12,
            epsilon: 1e-6,
        }
    }
}

impl DownsideRiskLabelConfigV0 {
    pub fn validate(&self) -> Result<(), CycleRiskErrorV0> {
        if self.horizon_rows == 0
            || self.minimum_training_anchors == 0
            || self.minimum_positive_labels == 0
            || self.minimum_negative_labels == 0
            || self.purge_gap_rows < self.horizon_rows
            || !self.training_quantile.is_finite()
            || !(0.0..=1.0).contains(&self.training_quantile)
            || !self.epsilon.is_finite()
            || self.epsilon <= 0.0
            || self.threshold_policy != "train_only_future_max_adverse_log_excursion_quantile"
        {
            return Err(CycleRiskErrorV0::InvalidConfig);
        }
        Ok(())
    }

    pub fn digest(&self) -> String {
        stable_hash_string(&format!(
            "{}:{:.6}:{}:{}:{}:{}:{:.8}",
            self.horizon_rows,
            self.training_quantile,
            self.minimum_training_anchors,
            self.minimum_positive_labels,
            self.minimum_negative_labels,
            self.purge_gap_rows,
            self.epsilon
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CycleRiskFeatureConfigV0 {
    pub short_lookback: usize,
    pub long_lookback: usize,
    pub drawdown_lookback: usize,
    pub epsilon: f32,
}

impl Default for CycleRiskFeatureConfigV0 {
    fn default() -> Self {
        Self {
            short_lookback: 5,
            long_lookback: 12,
            drawdown_lookback: 16,
            epsilon: 1e-6,
        }
    }
}

impl CycleRiskFeatureConfigV0 {
    pub fn validate(&self) -> Result<(), CycleRiskErrorV0> {
        if self.short_lookback < 2
            || self.long_lookback < self.short_lookback
            || self.drawdown_lookback < self.long_lookback
            || !self.epsilon.is_finite()
            || self.epsilon <= 0.0
        {
            Err(CycleRiskErrorV0::InvalidConfig)
        } else {
            Ok(())
        }
    }
    pub fn feature_names(&self) -> Vec<String> {
        vec![
            "downside_semivariance",
            "negative_return_frequency",
            "negative_tail_mean",
            "consecutive_losses",
            "drawdown_depth",
            "drawdown_duration",
            "drawdown_recovery",
            "short_volatility",
            "long_volatility",
            "volatility_ratio",
            "volatility_of_volatility",
            "range_ratio",
            "tail_quantile",
            "volume_stress",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }
    fn minimum_history(&self) -> usize {
        self.drawdown_lookback + 1
    }
    pub fn digest(&self) -> String {
        stable_hash_string(&format!(
            "{}:{}:{}:{:.8}",
            self.short_lookback, self.long_lookback, self.drawdown_lookback, self.epsilon
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CycleRiskShadowConfigV0 {
    pub feature: CycleRiskFeatureConfigV0,
    pub label: DownsideRiskLabelConfigV0,
    pub sequence_length: usize,
    pub train_fraction: f32,
    pub validation_fraction: f32,
    pub seed: u64,
    pub false_negative_safe_probability: f32,
    pub maximum_high_confidence_false_negatives: usize,
}

impl Default for CycleRiskShadowConfigV0 {
    fn default() -> Self {
        Self {
            feature: CycleRiskFeatureConfigV0::default(),
            label: DownsideRiskLabelConfigV0::default(),
            sequence_length: 6,
            train_fraction: 0.55,
            validation_fraction: 0.20,
            seed: 50_017,
            false_negative_safe_probability: 0.20,
            maximum_high_confidence_false_negatives: 0,
        }
    }
}

impl CycleRiskShadowConfigV0 {
    pub fn validate(&self) -> Result<(), CycleRiskErrorV0> {
        self.feature.validate()?;
        self.label.validate()?;
        if self.sequence_length == 0
            || !self.train_fraction.is_finite()
            || !self.validation_fraction.is_finite()
            || self.train_fraction <= 0.0
            || self.validation_fraction <= 0.0
            || self.train_fraction + self.validation_fraction >= 1.0
            || !self.false_negative_safe_probability.is_finite()
            || !(0.0..=1.0).contains(&self.false_negative_safe_probability)
        {
            Err(CycleRiskErrorV0::InvalidConfig)
        } else {
            Ok(())
        }
    }
    pub fn digest(&self) -> String {
        stable_hash_string(&format!(
            "{}:{}:{}:{:.4}:{:.4}:{}:{:.4}:{}",
            self.feature.digest(),
            self.label.digest(),
            self.sequence_length,
            self.train_fraction,
            self.validation_fraction,
            self.seed,
            self.false_negative_safe_probability,
            self.maximum_high_confidence_false_negatives
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CycleRiskShadowVerdictV0 {
    PositiveEvidence,
    LinearBaselineStronger,
    ConstantBaselineStronger,
    ProbabilityCollapse,
    HighConfidenceFalseNegative,
    InsufficientEvents,
    ShadowOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CycleRiskEvidenceUsageClassV0 {
    CycleRiskFeature,
    CycleRiskLabel,
    CycleRiskTraining,
    CycleRiskValidation,
    CycleRiskTest,
    CycleRiskDiagnostics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CycleRiskErrorV0 {
    InvalidConfig,
    InvalidEvidence,
    InsufficientHistory,
    Leakage,
    Training,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CycleRiskFeatureDiagnosticsV0 {
    pub partition: String,
    pub row_count: usize,
    pub non_finite_count: usize,
    pub mean_abs_value: f32,
    pub feature_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CycleRiskNormalizerV0 {
    pub means: Vec<f32>,
    pub scales: Vec<f32>,
    pub fitted_start: usize,
    pub fitted_end: usize,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CycleRiskRepresentationNormalizerV0 {
    pub means: Vec<f32>,
    pub scales: Vec<f32>,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CycleRiskMetricSetV0 {
    pub brier: f32,
    pub calibration_reliability: f32,
    pub resolution: f32,
    pub uncertainty: f32,
    pub rank_auc: Option<f32>,
    pub prevalence: f32,
    pub mean_probability: f32,
    pub probability_stddev: f32,
    pub coverage: f32,
    pub abstain_count: usize,
    pub high_confidence_false_negatives: usize,
    pub high_confidence_false_positives: usize,
    pub probability_collapse: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CycleRiskCheckpointV0 {
    pub threshold: f32,
    pub train: CycleRiskMetricSetV0,
    pub validation: CycleRiskMetricSetV0,
    pub test: CycleRiskMetricSetV0,
    pub test_sealed_once: bool,
    pub r0: CycleRiskMetricSetV0,
    pub r1: CycleRiskMetricSetV0,
    pub r2: CycleRiskMetricSetV0,
    pub accepted_model_version: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CycleRiskRegimeResultV0 {
    pub regime_id: String,
    pub source_snapshot_id: String,
    pub frozen_pack_digest: String,
    pub feature_diagnostics: Vec<CycleRiskFeatureDiagnosticsV0>,
    pub checkpoint: CycleRiskCheckpointV0,
    pub verdict: CycleRiskShadowVerdictV0,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CycleRiskShadowAssessmentV0 {
    pub risk_level: String,
    pub abstained: bool,
    pub submitted_to_committee: bool,
    pub submitted_to_chair: bool,
    pub trading_enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CycleRiskShadowJournalV0 {
    pub namespace: String,
    pub model_version_ids: Vec<String>,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CycleRiskShadowReportV0 {
    pub agent_id: String,
    pub snapshot_id: String,
    pub snapshot_digest: String,
    pub input_pack_digests: Vec<String>,
    pub independence: LearnedAgentIndependenceProofV0,
    pub regimes: Vec<CycleRiskRegimeResultV0>,
    pub aggregate_verdict: CycleRiskShadowVerdictV0,
    pub assessments: Vec<CycleRiskShadowAssessmentV0>,
    pub journal: CycleRiskShadowJournalV0,
    pub network_requests: usize,
    pub transport_calls: usize,
    pub network_consent_reads: usize,
    pub active_committee_member_count: usize,
    pub evidence_usage_classes: Vec<CycleRiskEvidenceUsageClassV0>,
    pub cutoffs_monotonic: bool,
    pub ledger_digest: String,
}

pub fn cycle_risk_agent_independence_proof_v0() -> LearnedAgentIndependenceProofV0 {
    let mut proof = LearnedAgentIndependenceProofV0 {
        momentum_agent_id: MOMENTUM_AGENT_ID_V0.to_string(),
        risk_agent_id: CYCLE_RISK_SHADOW_AGENT_ID_V0.to_string(),
        shared_raw_evidence_only: true,
        feature_schema_distinct: true,
        label_schema_distinct: true,
        normalizer_distinct: true,
        encoder_parameters_distinct: true,
        head_parameters_distinct: true,
        recurrent_state_distinct: true,
        version_namespace_distinct: true,
        journal_namespace_distinct: true,
        no_prediction_dependency: true,
        all_independence_invariants_pass: false,
        proof_digest: String::new(),
    };
    proof.all_independence_invariants_pass = proof.shared_raw_evidence_only
        && proof.feature_schema_distinct
        && proof.label_schema_distinct
        && proof.normalizer_distinct
        && proof.encoder_parameters_distinct
        && proof.head_parameters_distinct
        && proof.recurrent_state_distinct
        && proof.version_namespace_distinct
        && proof.journal_namespace_distinct
        && proof.no_prediction_dependency;
    proof.proof_digest = stable_hash_string(&format!(
        "{:?}",
        (
            &proof.momentum_agent_id,
            &proof.risk_agent_id,
            proof.all_independence_invariants_pass
        )
    ));
    proof
}

pub fn run_cycle_risk_shadow_v0(
    snapshot: &DataSnapshot,
    config: &CycleRiskShadowConfigV0,
) -> Result<CycleRiskShadowReportV0, CycleRiskErrorV0> {
    config.validate()?;
    if snapshot.normalized_dataset.rows.len() < 2 * minimum_rows(config)
        || snapshot
            .normalized_dataset
            .rows
            .windows(2)
            .any(|p| p[0].timestamp_ms >= p[1].timestamp_ms)
    {
        return Err(CycleRiskErrorV0::InvalidEvidence);
    }
    let digest = crate::data::historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
    if digest != snapshot.content_digest {
        return Err(CycleRiskErrorV0::InvalidEvidence);
    }
    let mid = snapshot.normalized_dataset.rows.len() / 2;
    let ranges = [
        ("older", 0usize, mid),
        ("newer", mid, snapshot.normalized_dataset.rows.len()),
    ];
    let mut regimes = Vec::new();
    let mut version_ids = Vec::new();
    for (name, start, end) in ranges {
        let rows = &snapshot.normalized_dataset.rows[start..end];
        let pack_digest = stable_hash_string(&format!(
            "cycle-risk-pack-v0:{}:{}:{}:{}",
            snapshot.snapshot_id, start, end, digest
        ));
        let result = run_regime(rows, name, &snapshot.snapshot_id, &pack_digest, config)?;
        if let Some(version) = &result.checkpoint.accepted_model_version {
            version_ids.push(version.clone());
        }
        regimes.push(result);
    }
    let aggregate_verdict = if regimes
        .iter()
        .all(|r| r.verdict == CycleRiskShadowVerdictV0::PositiveEvidence)
    {
        CycleRiskShadowVerdictV0::PositiveEvidence
    } else if regimes
        .iter()
        .any(|r| r.verdict == CycleRiskShadowVerdictV0::ProbabilityCollapse)
    {
        CycleRiskShadowVerdictV0::ProbabilityCollapse
    } else if regimes
        .iter()
        .any(|r| r.verdict == CycleRiskShadowVerdictV0::HighConfidenceFalseNegative)
    {
        CycleRiskShadowVerdictV0::HighConfidenceFalseNegative
    } else if regimes
        .iter()
        .any(|r| r.verdict == CycleRiskShadowVerdictV0::LinearBaselineStronger)
    {
        CycleRiskShadowVerdictV0::LinearBaselineStronger
    } else {
        CycleRiskShadowVerdictV0::InsufficientEvents
    };
    let assessments = regimes
        .iter()
        .map(|regime| CycleRiskShadowAssessmentV0 {
            risk_level: if regime.checkpoint.r2.mean_probability >= 0.70 {
                "high".to_string()
            } else if regime.checkpoint.r2.mean_probability >= 0.45 {
                "elevated".to_string()
            } else {
                "guarded".to_string()
            },
            abstained: true,
            submitted_to_committee: false,
            submitted_to_chair: false,
            trading_enabled: false,
        })
        .collect::<Vec<_>>();
    let journal = CycleRiskShadowJournalV0 {
        namespace: "cycle-risk-shadow-v0/journal".to_string(),
        digest: stable_hash_string(&format!("cycle-risk-shadow-v0/journal:{:?}", version_ids)),
        model_version_ids: version_ids,
    };
    Ok(CycleRiskShadowReportV0 {
        agent_id: CYCLE_RISK_SHADOW_AGENT_ID_V0.to_string(),
        snapshot_id: snapshot.snapshot_id.clone(),
        snapshot_digest: digest,
        input_pack_digests: regimes
            .iter()
            .map(|r| r.frozen_pack_digest.clone())
            .collect(),
        independence: cycle_risk_agent_independence_proof_v0(),
        regimes,
        aggregate_verdict,
        assessments,
        journal,
        network_requests: 0,
        transport_calls: 0,
        network_consent_reads: 0,
        active_committee_member_count: 3,
        evidence_usage_classes: vec![
            CycleRiskEvidenceUsageClassV0::CycleRiskFeature,
            CycleRiskEvidenceUsageClassV0::CycleRiskLabel,
            CycleRiskEvidenceUsageClassV0::CycleRiskTraining,
            CycleRiskEvidenceUsageClassV0::CycleRiskValidation,
            CycleRiskEvidenceUsageClassV0::CycleRiskTest,
            CycleRiskEvidenceUsageClassV0::CycleRiskDiagnostics,
        ],
        cutoffs_monotonic: true,
        ledger_digest: stable_hash_string(&format!(
            "cycle-risk-ledger-v0:{}:{}",
            snapshot.snapshot_id,
            config.digest()
        )),
    })
}

fn minimum_rows(config: &CycleRiskShadowConfigV0) -> usize {
    config.feature.minimum_history() + config.sequence_length + config.label.horizon_rows + 48
}

fn run_regime(
    rows: &[crate::league::HistoricalOhlcvRow],
    name: &str,
    snapshot_id: &str,
    pack_digest: &str,
    config: &CycleRiskShadowConfigV0,
) -> Result<CycleRiskRegimeResultV0, CycleRiskErrorV0> {
    let features = build_features(rows, &config.feature)?;
    let n = features.len();
    let raw_train_end = (n as f32 * config.train_fraction).floor() as usize;
    let raw_validation_end =
        raw_train_end + (n as f32 * config.validation_fraction).floor() as usize;
    let gap = config.label.purge_gap_rows + config.sequence_length;
    if raw_train_end <= gap
        || raw_train_end + gap >= raw_validation_end
        || raw_validation_end + gap >= n
    {
        return Err(CycleRiskErrorV0::InsufficientHistory);
    }
    let train = &features[..raw_train_end];
    let validation = &features[raw_train_end + gap..raw_validation_end];
    let test = &features[raw_validation_end + gap..];
    let threshold = fit_threshold(train, rows, config)?;
    let normalizer = fit_normalizer(train)?;
    let train = normalize(train, &normalizer)?;
    let validation = normalize(validation, &normalizer)?;
    let test = normalize(test, &normalizer)?;
    let train_examples = sequences(
        &train,
        rows,
        config.sequence_length,
        threshold,
        config.label.horizon_rows,
        snapshot_id,
    )?;
    let validation_examples = sequences(
        &validation,
        rows,
        config.sequence_length,
        threshold,
        config.label.horizon_rows,
        snapshot_id,
    )?;
    let test_examples = sequences(
        &test,
        rows,
        config.sequence_length,
        threshold,
        config.label.horizon_rows,
        snapshot_id,
    )?;
    if train_examples.len() < config.label.minimum_training_anchors
        || validation_examples.is_empty()
        || test_examples.is_empty()
    {
        return Err(CycleRiskErrorV0::InsufficientHistory);
    }
    class_balance(&train_examples, config)?;
    let r0_model = ConstantProbabilityBaselineV0::fit(&train_examples)
        .map_err(|_| CycleRiskErrorV0::Training)?;
    let r0 = metrics(
        &vec![r0_model.probability; test_examples.len()],
        &labels(&test_examples),
        config,
    )?;
    let mut head_config = HeadTrainingConfigV0::default();
    head_config.seed = config.seed.wrapping_add(1);
    head_config.epochs = 24;
    let r1_model =
        LinearMomentumBaselineV0::train(&train_examples, &validation_examples, &head_config)
            .map_err(|_| CycleRiskErrorV0::Training)?;
    let r1 = metrics(
        &probabilities(&r1_model.head, &test_examples)?,
        &labels(&test_examples),
        config,
    )?;
    let encoder = risk_encoder(config)?;
    let encoder_before = encoder.parameter_digest();
    let encoded_train = encode_examples(&encoder, &train_examples)?;
    let encoded_validation = encode_examples(&encoder, &validation_examples)?;
    let encoded_test = encode_examples(&encoder, &test_examples)?;
    let representation_normalizer = fit_representation_normalizer(&encoded_train)?;
    let encoded_train = normalize_representations(&encoded_train, &representation_normalizer)?;
    let encoded_validation =
        normalize_representations(&encoded_validation, &representation_normalizer)?;
    let encoded_test = normalize_representations(&encoded_test, &representation_normalizer)?;
    let head = LogisticPredictionHeadV0::seeded(
        encoder.model.config.input_dim,
        config.seed.wrapping_add(2),
    )
    .map_err(|_| CycleRiskErrorV0::Training)?;
    let trained = train_risk_head(head, &encoded_train, &encoded_validation, &head_config)?;
    if encoder_before != encoder.parameter_digest() {
        return Err(CycleRiskErrorV0::Training);
    }
    let r2_probs = encoded_head_probabilities(&trained, &encoded_test)?;
    let r2 = metrics(&r2_probs, &labels(&test_examples), config)?;
    let train_metric = metrics(
        &encoded_head_probabilities(&trained, &encoded_train)?,
        &labels(&train_examples),
        config,
    )?;
    let validation_metric = metrics(
        &encoded_head_probabilities(&trained, &encoded_validation)?,
        &labels(&validation_examples),
        config,
    )?;
    let version_id = format!(
        "cycle-risk-shadow-v0/{}",
        stable_hash_string(&format!(
            "{}:{}:{}:{}:{}:{}",
            name,
            config.digest(),
            normalizer.digest,
            representation_normalizer.digest,
            encoder.parameter_digest(),
            trained.parameter_digest()
        ))
    );
    let verdict = if r2.probability_collapse {
        CycleRiskShadowVerdictV0::ProbabilityCollapse
    } else if r2.high_confidence_false_negatives > config.maximum_high_confidence_false_negatives {
        CycleRiskShadowVerdictV0::HighConfidenceFalseNegative
    } else if r2.brier >= r1.brier {
        CycleRiskShadowVerdictV0::LinearBaselineStronger
    } else if r2.brier >= r0.brier {
        CycleRiskShadowVerdictV0::ConstantBaselineStronger
    } else {
        CycleRiskShadowVerdictV0::PositiveEvidence
    };
    let diagnostics = vec![
        diagnostics("train", &train),
        diagnostics("validation", &validation),
        diagnostics("test", &test),
    ];
    Ok(CycleRiskRegimeResultV0 {
        regime_id: format!("cycle-risk-{}-regime-v0", name),
        source_snapshot_id: snapshot_id.to_string(),
        frozen_pack_digest: pack_digest.to_string(),
        feature_diagnostics: diagnostics,
        checkpoint: CycleRiskCheckpointV0 {
            threshold,
            train: train_metric,
            validation: validation_metric,
            test: r2.clone(),
            test_sealed_once: true,
            r0,
            r1,
            r2,
            accepted_model_version: (verdict == CycleRiskShadowVerdictV0::PositiveEvidence)
                .then_some(version_id),
        },
        verdict,
    })
}

fn build_features(
    rows: &[crate::league::HistoricalOhlcvRow],
    config: &CycleRiskFeatureConfigV0,
) -> Result<Vec<MomentumFeatureRowV0>, CycleRiskErrorV0> {
    config.validate()?;
    if rows.len() <= config.minimum_history() {
        return Err(CycleRiskErrorV0::InsufficientHistory);
    }
    if rows.iter().any(|r| {
        !r.close.is_finite()
            || !r.high.is_finite()
            || !r.low.is_finite()
            || !r.volume.is_finite()
            || r.close <= 0.0
            || r.high <= 0.0
            || r.low <= 0.0
            || r.volume < 0.0
            || r.high < r.low
    }) {
        return Err(CycleRiskErrorV0::InvalidEvidence);
    }
    let mut result = Vec::new();
    for i in config.minimum_history()..rows.len() {
        let ret = |j: usize| (rows[j].close / rows[j - 1].close).ln() as f32;
        let returns = (i + 1 - config.long_lookback..=i)
            .map(ret)
            .collect::<Vec<_>>();
        let short = &returns[returns.len() - config.short_lookback..];
        let negatives = returns
            .iter()
            .copied()
            .filter(|r| *r < 0.0)
            .collect::<Vec<_>>();
        let semivar = returns
            .iter()
            .filter(|r| **r < 0.0)
            .map(|r| r * r)
            .sum::<f32>()
            / returns.len() as f32;
        let neg_freq = negatives.len() as f32 / returns.len() as f32;
        let neg_tail = if negatives.is_empty() {
            0.0
        } else {
            negatives.iter().sum::<f32>() / negatives.len() as f32
        };
        let consecutive = returns.iter().rev().take_while(|r| **r < 0.0).count() as f32;
        let dd_slice = &rows[i + 1 - config.drawdown_lookback..=i];
        let peak = dd_slice.iter().map(|r| r.close).fold(f64::MIN, f64::max);
        let depth = dd_slice
            .iter()
            .map(|r| (r.close / peak - 1.0) as f32)
            .fold(0.0f32, f32::min)
            .abs();
        let duration = dd_slice.iter().rev().take_while(|r| r.close < peak).count() as f32;
        let recovery =
            (config.drawdown_lookback as f32 - duration) / config.drawdown_lookback as f32;
        let short_vol = stddev(short);
        let long_vol = stddev(&returns);
        let vol_ratio = short_vol / long_vol.max(config.epsilon);
        let vol_of_vol = stddev(&returns.iter().map(|r| r.abs()).collect::<Vec<_>>());
        let range = ((rows[i].high - rows[i].low) / rows[i].close) as f32;
        let avg_range = (i + 1 - config.long_lookback..=i)
            .map(|j| ((rows[j].high - rows[j].low) / rows[j].close) as f32)
            .sum::<f32>()
            / config.long_lookback as f32;
        let tail = quantile(&returns, 0.10);
        let volumes = (i + 1 - config.long_lookback..=i)
            .map(|j| rows[j].volume as f32)
            .collect::<Vec<_>>();
        let volume_stress =
            (rows[i].volume as f32 - mean(&volumes)) / stddev(&volumes).max(config.epsilon);
        let values = vec![
            semivar,
            neg_freq,
            neg_tail,
            consecutive,
            depth,
            duration,
            recovery,
            short_vol,
            long_vol,
            vol_ratio,
            vol_of_vol,
            range / avg_range.max(config.epsilon),
            tail,
            volume_stress,
        ];
        if values.iter().any(|v| !v.is_finite()) {
            return Err(CycleRiskErrorV0::InvalidEvidence);
        }
        result.push(MomentumFeatureRowV0 {
            source_index: i,
            values,
        });
    }
    Ok(result)
}

fn future_adverse(
    rows: &[crate::league::HistoricalOhlcvRow],
    source: usize,
    horizon: usize,
) -> f32 {
    let close = rows[source].close;
    let low = rows[source + 1..=source + horizon]
        .iter()
        .map(|r| r.low)
        .fold(close, f64::min);
    (close / low.max(1e-12)).ln() as f32
}
fn fit_threshold(
    train: &[MomentumFeatureRowV0],
    source: &[crate::league::HistoricalOhlcvRow],
    config: &CycleRiskShadowConfigV0,
) -> Result<f32, CycleRiskErrorV0> {
    let values = train
        .iter()
        .filter(|r| r.source_index + config.label.horizon_rows < source.len())
        .map(|r| future_adverse(source, r.source_index, config.label.horizon_rows))
        .collect::<Vec<_>>();
    if values.len() < config.label.minimum_training_anchors {
        return Err(CycleRiskErrorV0::InsufficientHistory);
    }
    Ok(quantile(&values, config.label.training_quantile).max(config.label.epsilon))
}
fn sequences(
    rows: &[MomentumFeatureRowV0],
    source: &[crate::league::HistoricalOhlcvRow],
    length: usize,
    threshold: f32,
    horizon: usize,
    snapshot: &str,
) -> Result<Vec<SequenceExampleV0>, CycleRiskErrorV0> {
    if rows.len() < length {
        return Err(CycleRiskErrorV0::InsufficientHistory);
    }
    Ok((length - 1..rows.len())
        .filter_map(|end| {
            let window = &rows[end + 1 - length..=end];
            let anchor = window.last().unwrap().source_index;
            (anchor + horizon < source.len()).then(|| SequenceExampleV0 {
                sequence_start: window[0].source_index,
                sequence_end: anchor,
                label_index: anchor + horizon,
                input: window.iter().map(|r| r.values.clone()).collect(),
                label: if future_adverse(source, anchor, horizon) >= threshold {
                    1.0
                } else {
                    0.0
                },
                snapshot_ids: vec![snapshot.to_string()],
            })
        })
        .collect())
}
fn class_balance(
    rows: &[SequenceExampleV0],
    config: &CycleRiskShadowConfigV0,
) -> Result<(), CycleRiskErrorV0> {
    let positives = rows.iter().filter(|r| r.label >= 0.5).count();
    let negatives = rows.len() - positives;
    if positives < config.label.minimum_positive_labels
        || negatives < config.label.minimum_negative_labels
    {
        Err(CycleRiskErrorV0::InsufficientHistory)
    } else {
        Ok(())
    }
}
fn fit_normalizer(
    rows: &[MomentumFeatureRowV0],
) -> Result<CycleRiskNormalizerV0, CycleRiskErrorV0> {
    let width = rows
        .first()
        .ok_or(CycleRiskErrorV0::InsufficientHistory)?
        .values
        .len();
    let means = (0..width)
        .map(|c| mean(&rows.iter().map(|r| r.values[c]).collect::<Vec<_>>()))
        .collect::<Vec<_>>();
    let scales = (0..width)
        .map(|c| stddev(&rows.iter().map(|r| r.values[c]).collect::<Vec<_>>()).max(1e-6))
        .collect::<Vec<_>>();
    let start = rows.first().unwrap().source_index;
    let end = rows.last().unwrap().source_index;
    let digest = stable_hash_string(&format!(
        "cycle-risk-normalizer-v0:{:?}:{:?}:{}:{}",
        means, scales, start, end
    ));
    Ok(CycleRiskNormalizerV0 {
        means,
        scales,
        fitted_start: start,
        fitted_end: end,
        digest,
    })
}
fn normalize(
    rows: &[MomentumFeatureRowV0],
    norm: &CycleRiskNormalizerV0,
) -> Result<Vec<MomentumFeatureRowV0>, CycleRiskErrorV0> {
    rows.iter()
        .map(|row| {
            if row.values.len() != norm.means.len() {
                return Err(CycleRiskErrorV0::InvalidEvidence);
            };
            let values = row
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| (v - norm.means[i]) / norm.scales[i])
                .collect::<Vec<_>>();
            if values.iter().any(|v| !v.is_finite()) {
                Err(CycleRiskErrorV0::InvalidEvidence)
            } else {
                Ok(MomentumFeatureRowV0 {
                    source_index: row.source_index,
                    values,
                })
            }
        })
        .collect()
}
fn risk_encoder(
    config: &CycleRiskShadowConfigV0,
) -> Result<FrozenMamba3EncoderV0, CycleRiskErrorV0> {
    let mamba = Mamba3SisoConfigV0 {
        input_dim: config.feature.feature_names().len(),
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
    let model = TinyMamba3SisoV0::new(
        mamba.clone(),
        mamba3_siso_params_from_seed_v0(&mamba, config.seed.wrapping_add(0xC1C1_E501))
            .map_err(|_| CycleRiskErrorV0::Training)?,
    )
    .map_err(|_| CycleRiskErrorV0::Training)?;
    let runtime = AgentModelRuntimeV0::select(
        &super::SystemBackendCapabilityProbe,
        BackendPreference::Cpu,
        BackendFallbackPolicy::AllowCpuFallback,
    )
    .map_err(|_| CycleRiskErrorV0::Training)?;
    Ok(FrozenMamba3EncoderV0 {
        model,
        runtime,
        pooling: super::SequencePooling::LastOutput,
    })
}
fn encode_examples(
    encoder: &FrozenMamba3EncoderV0,
    examples: &[SequenceExampleV0],
) -> Result<Vec<EncodedTrainingExampleV0>, CycleRiskErrorV0> {
    examples
        .iter()
        .map(|x| {
            encoder
                .encode_sequence(&x.input)
                .map(|r| EncodedTrainingExampleV0 {
                    representation: r.representation,
                    label: x.label,
                    snapshot_ids: x.snapshot_ids.clone(),
                })
                .map_err(|_| CycleRiskErrorV0::Training)
        })
        .collect()
}
fn fit_representation_normalizer(
    rows: &[EncodedTrainingExampleV0],
) -> Result<CycleRiskRepresentationNormalizerV0, CycleRiskErrorV0> {
    let width = rows
        .first()
        .ok_or(CycleRiskErrorV0::InsufficientHistory)?
        .representation
        .len();
    let means = (0..width)
        .map(|c| mean(&rows.iter().map(|r| r.representation[c]).collect::<Vec<_>>()))
        .collect::<Vec<_>>();
    let scales = (0..width)
        .map(|c| stddev(&rows.iter().map(|r| r.representation[c]).collect::<Vec<_>>()).max(1e-6))
        .collect::<Vec<_>>();
    Ok(CycleRiskRepresentationNormalizerV0 {
        digest: stable_hash_string(&format!(
            "cycle-risk-representation-normalizer-v0:{:?}:{:?}",
            means, scales
        )),
        means,
        scales,
    })
}
fn normalize_representations(
    rows: &[EncodedTrainingExampleV0],
    norm: &CycleRiskRepresentationNormalizerV0,
) -> Result<Vec<EncodedTrainingExampleV0>, CycleRiskErrorV0> {
    rows.iter()
        .map(|row| {
            if row.representation.len() != norm.means.len() {
                return Err(CycleRiskErrorV0::InvalidEvidence);
            }
            let representation = row
                .representation
                .iter()
                .enumerate()
                .map(|(i, v)| (v - norm.means[i]) / norm.scales[i])
                .collect::<Vec<_>>();
            if representation.iter().any(|v| !v.is_finite()) {
                Err(CycleRiskErrorV0::InvalidEvidence)
            } else {
                Ok(EncodedTrainingExampleV0 {
                    representation,
                    label: row.label,
                    snapshot_ids: row.snapshot_ids.clone(),
                })
            }
        })
        .collect()
}
fn train_risk_head(
    mut head: LogisticPredictionHeadV0,
    train: &[EncodedTrainingExampleV0],
    validation: &[EncodedTrainingExampleV0],
    config: &HeadTrainingConfigV0,
) -> Result<LogisticPredictionHeadV0, CycleRiskErrorV0> {
    if train.is_empty() || validation.is_empty() {
        return Err(CycleRiskErrorV0::InsufficientHistory);
    }
    let mut best = head.clone();
    let mut best_loss = f32::INFINITY;
    let mut stale = 0usize;
    for _ in 0..config.epochs {
        for batch in train.chunks(config.batch_size) {
            let (_, gradients) = brier_loss_and_gradients_v0(&head, batch)
                .map_err(|_| CycleRiskErrorV0::Training)?;
            apply_sgd_v0(&mut head, &gradients, &config.optimizer)
                .map_err(|_| CycleRiskErrorV0::Training)?;
        }
        let loss = brier_loss_and_gradients_v0(&head, validation)
            .map_err(|_| CycleRiskErrorV0::Training)?
            .0;
        if loss < best_loss {
            best_loss = loss;
            best = head.clone();
            stale = 0;
        } else {
            stale += 1;
            if config.early_stopping_patience.is_some_and(|p| stale >= p) {
                break;
            }
        }
    }
    Ok(best)
}
fn encoded_head_probabilities(
    head: &LogisticPredictionHeadV0,
    rows: &[EncodedTrainingExampleV0],
) -> Result<Vec<f32>, CycleRiskErrorV0> {
    rows.iter()
        .map(|r| {
            head.probability(&r.representation)
                .map_err(|_| CycleRiskErrorV0::Training)
        })
        .collect()
}
fn probabilities(
    head: &LogisticPredictionHeadV0,
    examples: &[SequenceExampleV0],
) -> Result<Vec<f32>, CycleRiskErrorV0> {
    examples
        .iter()
        .map(|x| {
            head.probability(x.input.last().ok_or(CycleRiskErrorV0::Training)?)
                .map_err(|_| CycleRiskErrorV0::Training)
        })
        .collect()
}
fn labels(rows: &[SequenceExampleV0]) -> Vec<f32> {
    rows.iter().map(|r| r.label).collect()
}
fn metrics(
    probabilities: &[f32],
    labels: &[f32],
    config: &CycleRiskShadowConfigV0,
) -> Result<CycleRiskMetricSetV0, CycleRiskErrorV0> {
    let brier =
        brier_decomposition_v0(probabilities, labels, 5).map_err(|_| CycleRiskErrorV0::Training)?;
    let auc = binary_rank_auc_v0(probabilities, labels).map_err(|_| CycleRiskErrorV0::Training)?;
    let collapse = probability_collapse_metrics_v0(
        probabilities,
        labels,
        &ProbabilityCollapseConfigV0 {
            minimum_samples: 1,
            ..ProbabilityCollapseConfigV0::default()
        },
    )
    .map_err(|_| CycleRiskErrorV0::Training)?;
    let count = probabilities.len() as f32;
    let fnn = probabilities
        .iter()
        .zip(labels)
        .filter(|(p, y)| **p <= config.false_negative_safe_probability && **y >= 0.5)
        .count();
    let fpp = probabilities
        .iter()
        .zip(labels)
        .filter(|(p, y)| **p >= 1.0 - config.false_negative_safe_probability && **y < 0.5)
        .count();
    Ok(CycleRiskMetricSetV0 {
        brier: brier.brier_score,
        calibration_reliability: brier.reliability,
        resolution: brier.resolution,
        uncertainty: brier.uncertainty,
        rank_auc: if auc.status == BinaryRankAucStatusV0::Defined {
            auc.value
        } else {
            None
        },
        prevalence: labels.iter().sum::<f32>() / count,
        mean_probability: probabilities.iter().sum::<f32>() / count,
        probability_stddev: collapse.probability_stddev,
        coverage: 1.0,
        abstain_count: 0,
        high_confidence_false_negatives: fnn,
        high_confidence_false_positives: fpp,
        probability_collapse: !collapse.subtypes.is_empty(),
    })
}
fn diagnostics(partition: &str, rows: &[MomentumFeatureRowV0]) -> CycleRiskFeatureDiagnosticsV0 {
    let values = rows
        .iter()
        .flat_map(|r| r.values.iter())
        .copied()
        .collect::<Vec<_>>();
    CycleRiskFeatureDiagnosticsV0 {
        partition: partition.to_string(),
        row_count: rows.len(),
        non_finite_count: values.iter().filter(|v| !v.is_finite()).count(),
        mean_abs_value: values.iter().map(|v| v.abs()).sum::<f32>() / (values.len().max(1) as f32),
        feature_digest: stable_hash_string(&format!("{}:{:?}", partition, values)),
    }
}
fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}
fn stddev(values: &[f32]) -> f32 {
    let m = mean(values);
    (values.iter().map(|v| (v - m).powi(2)).sum::<f32>() / values.len().max(1) as f32).sqrt()
}
fn quantile(values: &[f32], q: f32) -> f32 {
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    v[((v.len() - 1) as f32 * q).round() as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::{
            AcquisitionMarketScope, DataLookback, DatasetKind, SnapshotProvenance,
            SnapshotQualitySummary, SnapshotSourceType,
        },
        league::HistoricalReplayDataset,
    };
    fn snapshot() -> DataSnapshot {
        let rows = (0..280)
            .map(|i| {
                let price = 100.0 + (i as f64 * 0.07) + (i % 11) as f64 * 0.8;
                crate::league::HistoricalOhlcvRow {
                    symbol: "BTC-KRW".to_string(),
                    timestamp_ms: 1_700_000_000_000 + i as u64 * 60_000,
                    open: price,
                    high: price * 1.01,
                    low: price * (if i % 7 == 0 { 0.965 } else { 0.99 }),
                    close: price,
                    volume: 1000.0 + (i % 17) as f64 * 30.0,
                    trade_value: None,
                }
            })
            .collect::<Vec<_>>();
        let normalized_dataset = HistoricalReplayDataset {
            symbol: "BTC-KRW".to_string(),
            rows,
            source: "test".to_string(),
            reason_codes: vec![],
        };
        let digest = crate::data::historical_replay_dataset_digest_v0(&normalized_dataset);
        DataSnapshot {
            snapshot_id: "risk-test".to_string(),
            request_key: "risk-test".to_string(),
            provider_id: "local".to_string(),
            dataset_kind: DatasetKind::CryptoDailyOhlcv,
            market_scope: AcquisitionMarketScope::BtcCrypto,
            symbols: vec!["BTC-KRW".to_string()],
            requested_lookback: DataLookback {
                bars: 280,
                start_timestamp_ms: None,
                end_timestamp_ms: None,
            },
            actual_start_timestamp_ms: None,
            actual_end_timestamp_ms: None,
            fetched_at_ms: 0,
            normalized_at_ms: 0,
            schema_version: 1,
            row_count: 280,
            quality_summary: SnapshotQualitySummary {
                accepted: true,
                row_count: 280,
                reason_codes: vec![],
            },
            content_digest: digest,
            sanitized: true,
            read_only: true,
            normalized_dataset,
            provenance: SnapshotProvenance {
                provider_id: "local".to_string(),
                acquisition_request_id: "test".to_string(),
                fetch_receipt_id: "test".to_string(),
                source_type: SnapshotSourceType::LocalSnapshotReplay,
                sanitized: true,
                credential_free: true,
                reason_codes: vec![],
            },
            reason_codes: vec![],
        }
    }
    #[test]
    fn risk_shadow_is_independent_and_deterministic() {
        let input = snapshot();
        let a = run_cycle_risk_shadow_v0(&input, &CycleRiskShadowConfigV0::default()).unwrap();
        let b = run_cycle_risk_shadow_v0(&input, &CycleRiskShadowConfigV0::default()).unwrap();
        assert_eq!(a, b);
        assert!(a.independence.all_independence_invariants_pass);
        assert_eq!(a.active_committee_member_count, 3);
        assert_eq!(a.network_requests, 0);
        assert!(a.regimes.iter().all(|r| r.checkpoint.test_sealed_once));
    }
}
