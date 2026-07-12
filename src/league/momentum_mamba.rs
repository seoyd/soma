//! Shadow-only adapter for the experimental frozen-Mamba momentum model.
//!
//! This module deliberately does not implement the active committee voting trait
//! and is not included in `default_league_votes`.

use serde::{Deserialize, Serialize};

use crate::{
    core::ReasonCode,
    data::acquisition::AgentEvidenceBundle,
    model::{
        FrozenMamba3EncoderV0, LearningError, LogisticPredictionHeadV0, Mamba3BackendKind,
        ModelAgentDeploymentStatus, SandboxModelVersionV0,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowSuggestedActionV0 {
    UpwardWatch,
    DownwardWatch,
    Abstain,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShadowAgentAssessmentV0 {
    pub agent_id: String,
    pub probability_up: f32,
    pub confidence: f32,
    pub suggested_action: ShadowSuggestedActionV0,
    pub evidence_snapshot_ids: Vec<String>,
    pub model_version_id: String,
    pub backend: Mamba3BackendKind,
    pub deployment_status: ModelAgentDeploymentStatus,
    pub eligible_to_vote: bool,
    pub eligible_to_execute: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug)]
pub struct MomentumMambaSandboxAgentV0 {
    pub agent_id: String,
    pub encoder: FrozenMamba3EncoderV0,
    pub head: LogisticPredictionHeadV0,
    pub model_version: SandboxModelVersionV0,
    pub upward_threshold: f32,
    pub downward_threshold: f32,
}

impl MomentumMambaSandboxAgentV0 {
    pub fn new(
        agent_id: impl Into<String>,
        encoder: FrozenMamba3EncoderV0,
        head: LogisticPredictionHeadV0,
        model_version: SandboxModelVersionV0,
        upward_threshold: f32,
        downward_threshold: f32,
    ) -> Result<Self, LearningError> {
        let agent_id = agent_id.into();
        if model_version.agent_id != agent_id
            || model_version.deployment_status != ModelAgentDeploymentStatus::ShadowOnly
            || !upward_threshold.is_finite()
            || !downward_threshold.is_finite()
            || !(0.5..=1.0).contains(&upward_threshold)
            || !(0.0..=0.5).contains(&downward_threshold)
        {
            return Err(LearningError::InvalidConfig);
        }
        head.validate()?;
        Ok(Self {
            agent_id,
            encoder,
            head,
            model_version,
            upward_threshold,
            downward_threshold,
        })
    }

    pub fn assess(
        &self,
        evidence: &AgentEvidenceBundle,
        sequence: &[Vec<f32>],
    ) -> Result<ShadowAgentAssessmentV0, LearningError> {
        if evidence.agent_id != self.agent_id
            || !evidence.frozen
            || !evidence.missing_required_datasets.is_empty()
            || evidence.required_snapshot_ids.is_empty()
            || evidence
                .required_snapshot_ids
                .iter()
                .any(|id| !self.model_version.data_snapshot_ids.contains(id))
        {
            return Err(LearningError::InvalidConfig);
        }
        let encoded = self.encoder.encode_sequence(sequence)?;
        let probability_up = self.head.probability(&encoded.representation)?;
        let suggested_action = if probability_up >= self.upward_threshold {
            ShadowSuggestedActionV0::UpwardWatch
        } else if probability_up <= self.downward_threshold {
            ShadowSuggestedActionV0::DownwardWatch
        } else {
            ShadowSuggestedActionV0::Abstain
        };
        Ok(ShadowAgentAssessmentV0 {
            agent_id: self.agent_id.clone(),
            probability_up,
            confidence: (probability_up - 0.5).abs() * 2.0,
            suggested_action,
            evidence_snapshot_ids: evidence.required_snapshot_ids.clone(),
            model_version_id: self.model_version.model_version_id.clone(),
            backend: encoded.backend,
            deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
            eligible_to_vote: false,
            eligible_to_execute: false,
            reason_codes: vec![
                ReasonCode::DeterministicPath,
                ReasonCode::ShadowEvaluationPending,
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::acquisition::{EvidenceDecisionGate, EvidenceFreshnessStatus},
        model::{
            AgentModelRuntimeV0, BackendFallbackPolicy, BackendPreference, EvaluationMetricsV0,
            Mamba3SisoConfigV0, Mamba3SisoPrecisionV0, Mamba3SisoRopeFractionV0,
            MambaRepresentationValueStatusV0, ModelMathematicalStatus, SandboxModelMetricsV0,
            SequencePooling, SystemBackendCapabilityProbe, TinyMamba3SisoV0,
            mamba3_siso_params_from_seed_v0,
        },
    };

    fn metrics() -> EvaluationMetricsV0 {
        EvaluationMetricsV0 {
            brier_score: 0.2,
            sample_count: 10,
            accuracy: 0.6,
            positive_label_rate: 0.5,
            mean_predicted_probability: 0.5,
            high_confidence_error_count: 1,
            abstention_count: 0,
            calibration_buckets: vec![],
        }
    }

    #[test]
    fn shadow_assessment_cannot_vote_or_execute() {
        let config = Mamba3SisoConfigV0 {
            input_dim: 2,
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
            config.clone(),
            mamba3_siso_params_from_seed_v0(&config, 17).unwrap(),
        )
        .unwrap();
        let encoder = FrozenMamba3EncoderV0 {
            model,
            runtime: AgentModelRuntimeV0::select(
                &SystemBackendCapabilityProbe,
                BackendPreference::Auto,
                BackendFallbackPolicy::AllowCpuFallback,
            )
            .unwrap(),
            pooling: SequencePooling::LastOutput,
        };
        let metric = metrics();
        let version = SandboxModelVersionV0::new(
            None,
            "momentum_mamba_shadow",
            "feature".to_string(),
            "normalizer".to_string(),
            encoder.parameter_digest(),
            "head".to_string(),
            "training".to_string(),
            &["snapshot-1".to_string()],
            crate::model::IndexRangeV0 { start: 0, end: 1 },
            crate::model::IndexRangeV0 { start: 2, end: 3 },
            crate::model::IndexRangeV0 { start: 4, end: 5 },
            Mamba3BackendKind::CpuReference,
            SandboxModelMetricsV0 {
                train: metric.clone(),
                validation: metric.clone(),
                test: metric,
                mamba_value_status: MambaRepresentationValueStatusV0::InsufficientEvidence,
            },
        );
        assert_eq!(
            version.mathematical_status,
            ModelMathematicalStatus::ExperimentalInternalReference
        );
        let agent = MomentumMambaSandboxAgentV0::new(
            "momentum_mamba_shadow",
            encoder,
            LogisticPredictionHeadV0::seeded(2, 5).unwrap(),
            version,
            0.6,
            0.4,
        )
        .unwrap();
        let evidence = AgentEvidenceBundle {
            agent_id: "momentum_mamba_shadow".to_string(),
            requested_datasets: vec![],
            required_snapshot_ids: vec!["snapshot-1".to_string()],
            optional_snapshot_ids: vec![],
            missing_required_datasets: vec![],
            missing_optional_datasets: vec![],
            freshness_status: EvidenceFreshnessStatus::Fresh,
            provenance_receipts: vec![],
            frozen: true,
            decision_gate: EvidenceDecisionGate::Ready,
            reason_codes: vec![],
        };
        let assessment = agent
            .assess(&evidence, &[vec![0.1, -0.2], vec![0.3, 0.2]])
            .unwrap();
        assert!(!assessment.eligible_to_vote);
        assert!(!assessment.eligible_to_execute);
        assert_eq!(
            assessment.deployment_status,
            ModelAgentDeploymentStatus::ShadowOnly
        );
    }
}
