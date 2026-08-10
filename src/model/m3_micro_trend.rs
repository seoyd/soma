//! TrendContinuation feature, target, loss, and output policy.

use super::{
    m3_micro::{
        AbstentionPolicy, AgentId, ConfidencePolicy, FormulaId, LossPolicy,
        M3MicroDevelopmentExample, M3MicroError, M3MicroPrediction, M3MicroTarget,
        PROBABILITY_EPSILON, TargetPolicy, bce_loss_and_gradient, distribution_loss_and_gradient,
        empty_prediction, inverse_tanh, probability_logit, softmax, valid_distribution,
        valid_probability,
    },
    m3_micro_capability::{M3MicroRolePolicyV1, RoleFormulaSelectionV1, RolePolicyDescriptorV1},
};

pub(crate) struct TrendRolePolicyV1;
pub(crate) const TREND_ROLE_POLICY_V1: TrendRolePolicyV1 = TrendRolePolicyV1;

impl M3MicroRolePolicyV1 for TrendRolePolicyV1 {
    fn descriptor(&self) -> RolePolicyDescriptorV1 {
        RolePolicyDescriptorV1 {
            agent_id: AgentId::TrendContinuation,
            output_dim: 5,
            target_policy: TargetPolicy::CostEvidenceDirectionContinuation,
            loss_policy: LossPolicy::DirectionRpsReturnAndCalibration,
            confidence_policy: ConfidencePolicy::DirectionMaximumProbability,
            abstention_policy: AbstentionPolicy::NeutralDominant,
        }
    }

    fn formulas(&self) -> RoleFormulaSelectionV1 {
        RoleFormulaSelectionV1 {
            active: vec![
                FormulaId::LogReturn1,
                FormulaId::LogReturn5,
                FormulaId::LogReturn20,
                FormulaId::VolatilityAdjustedMomentum,
                FormulaId::NormalizedCloseSlope20,
                FormulaId::ReturnSignAgreement5,
                FormulaId::VolumeConfirmedMovement5,
                FormulaId::BreakoutDistance20,
            ],
            rejected: vec![],
        }
    }

    fn validate_target(&self, target: &M3MicroTarget) -> bool {
        target.agent_id == AgentId::TrendContinuation
            && valid_distribution(target.direction_distribution)
            && target.future_return.is_some_and(f32::is_finite)
            && valid_probability(target.continuation)
            && target.future_variance.is_none()
            && target.volatility_regime.is_none()
            && target.risk_abstention.is_none()
            && target.reversal.is_none()
            && target.failed_breakout.is_none()
    }

    fn prediction_from_raw(&self, raw: &[f32]) -> Result<M3MicroPrediction, M3MicroError> {
        if raw.len() != 5 || raw.iter().any(|value| !value.is_finite()) {
            return Err(M3MicroError::InvalidShape);
        }
        let distribution = softmax(&raw[..3])?;
        let mut prediction = empty_prediction(AgentId::TrendContinuation);
        prediction.direction_distribution =
            Some([distribution[0], distribution[1], distribution[2]]);
        prediction.continuation_probability = Some(super::m3_micro::sigmoid(raw[3]));
        prediction.expected_return = Some(raw[4].tanh());
        prediction.confidence = distribution.iter().copied().reduce(f32::max).unwrap();
        prediction.abstain = distribution[1] >= distribution[0].max(distribution[2]);
        Ok(prediction)
    }

    fn loss_and_output_gradient(
        &self,
        raw: &[f32],
        target: &M3MicroTarget,
    ) -> Result<(f32, Vec<f32>), M3MicroError> {
        if raw.len() != 5 || !self.validate_target(target) {
            return Err(M3MicroError::InvalidShape);
        }
        let calibration = 0.05;
        let (distribution_loss, distribution_gradient) = distribution_loss_and_gradient(
            &raw[..3],
            &target.direction_distribution.unwrap(),
            calibration,
        )?;
        let (continuation_loss, continuation_gradient) =
            bce_loss_and_gradient(raw[3], target.continuation.unwrap(), calibration);
        let prediction = raw[4].tanh();
        let residual = prediction - target.future_return.unwrap();
        let mut gradient = vec![0.0; 5];
        gradient[..3].copy_from_slice(&distribution_gradient);
        gradient[3] = continuation_gradient;
        gradient[4] = 2.0 * residual * (1.0 - prediction * prediction);
        Ok((
            distribution_loss + continuation_loss + residual * residual,
            gradient,
        ))
    }

    fn constant_baseline_raw(
        &self,
        development: &[&M3MicroDevelopmentExample],
    ) -> Result<Vec<f32>, M3MicroError> {
        if development.is_empty() {
            return Err(M3MicroError::InvalidShape);
        }
        let count = development.len() as f32;
        let mut distribution = [0.0; 3];
        let mut continuation = 0.0;
        let mut future_return = 0.0;
        for example in development {
            for (mean, target) in distribution
                .iter_mut()
                .zip(example.target.direction_distribution.unwrap())
            {
                *mean += target / count;
            }
            continuation += example.target.continuation.unwrap() / count;
            future_return += example.target.future_return.unwrap() / count;
        }
        Ok(vec![
            distribution[0].max(PROBABILITY_EPSILON).ln(),
            distribution[1].max(PROBABILITY_EPSILON).ln(),
            distribution[2].max(PROBABILITY_EPSILON).ln(),
            probability_logit(continuation),
            inverse_tanh(future_return),
        ])
    }

    fn mathematical_baseline_raw(
        &self,
        normalized_sequence: &[Vec<f32>],
    ) -> Result<Vec<f32>, M3MicroError> {
        let last = normalized_sequence
            .last()
            .ok_or(M3MicroError::InvalidShape)?;
        if last.len() < 2 || last.iter().any(|value| !value.is_finite()) {
            return Err(M3MicroError::InvalidShape);
        }
        let first = last[0].clamp(-8.0, 8.0);
        Ok(vec![-first, -first.abs(), first, first.abs(), first])
    }
}
