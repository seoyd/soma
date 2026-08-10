//! VolatilityRegime feature, target, loss, and output policy.

use super::{
    m3_micro::{
        AbstentionPolicy, AgentId, ConfidencePolicy, FormulaId, LossPolicy,
        M3MicroDevelopmentExample, M3MicroError, M3MicroPrediction, M3MicroTarget,
        PROBABILITY_EPSILON, TargetPolicy, bce_loss_and_gradient, distribution_loss_and_gradient,
        empty_prediction, inverse_softplus, probability_logit, sigmoid, softmax, softplus,
        valid_distribution, valid_probability,
    },
    m3_micro_capability::{M3MicroRolePolicyV1, RoleFormulaSelectionV1, RolePolicyDescriptorV1},
};

pub(crate) struct VolatilityRolePolicyV1;
pub(crate) const VOLATILITY_ROLE_POLICY_V1: VolatilityRolePolicyV1 = VolatilityRolePolicyV1;

impl M3MicroRolePolicyV1 for VolatilityRolePolicyV1 {
    fn descriptor(&self) -> RolePolicyDescriptorV1 {
        RolePolicyDescriptorV1 {
            agent_id: AgentId::VolatilityRegime,
            output_dim: 5,
            target_policy: TargetPolicy::FutureVarianceAndRegime,
            loss_policy: LossPolicy::QlikeRegimeAndCalibration,
            confidence_policy: ConfidencePolicy::RegimeMaximumProbability,
            abstention_policy: AbstentionPolicy::RiskProbabilityAtLeastHalf,
        }
    }

    fn formulas(&self) -> RoleFormulaSelectionV1 {
        RoleFormulaSelectionV1 {
            active: vec![
                FormulaId::RealizedVolatility5,
                FormulaId::RealizedVolatility20,
                FormulaId::HighLowRangeEstimator,
                FormulaId::VolatilityOfVolatility10,
                FormulaId::RangeExpansion5,
                FormulaId::VolumeShock20,
                FormulaId::CrossTimeframeVolatilityRatio,
                FormulaId::RegimeDuration20,
            ],
            rejected: vec![],
        }
    }

    fn validate_target(&self, target: &M3MicroTarget) -> bool {
        target.agent_id == AgentId::VolatilityRegime
            && target
                .future_variance
                .is_some_and(|value| value.is_finite() && value >= 0.0)
            && valid_distribution(target.volatility_regime)
            && valid_probability(target.risk_abstention)
            && target.direction_distribution.is_none()
            && target.future_return.is_none()
            && target.continuation.is_none()
            && target.reversal.is_none()
            && target.failed_breakout.is_none()
    }

    fn prediction_from_raw(&self, raw: &[f32]) -> Result<M3MicroPrediction, M3MicroError> {
        if raw.len() != 5 || raw.iter().any(|value| !value.is_finite()) {
            return Err(M3MicroError::InvalidShape);
        }
        let distribution = softmax(&raw[1..4])?;
        let risk = sigmoid(raw[4]);
        let mut prediction = empty_prediction(AgentId::VolatilityRegime);
        prediction.predicted_variance = Some(softplus(raw[0]) + PROBABILITY_EPSILON);
        prediction.volatility_regime = Some([distribution[0], distribution[1], distribution[2]]);
        prediction.risk_abstention_probability = Some(risk);
        prediction.confidence = distribution.iter().copied().reduce(f32::max).unwrap();
        prediction.abstain = risk >= 0.5;
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
        let prediction = softplus(raw[0]) + PROBABILITY_EPSILON;
        let observed = target.future_variance.unwrap() + PROBABILITY_EPSILON;
        let qlike = prediction.ln() + observed / prediction;
        let (regime_loss, regime_gradient) = distribution_loss_and_gradient(
            &raw[1..4],
            &target.volatility_regime.unwrap(),
            calibration,
        )?;
        let (risk_loss, risk_gradient) =
            bce_loss_and_gradient(raw[4], target.risk_abstention.unwrap(), calibration);
        let mut gradient = vec![0.0; 5];
        gradient[0] = (1.0 / prediction - observed / (prediction * prediction)) * sigmoid(raw[0]);
        gradient[1..4].copy_from_slice(&regime_gradient);
        gradient[4] = risk_gradient;
        Ok((qlike + regime_loss + risk_loss, gradient))
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
        let mut variance = 0.0;
        let mut risk = 0.0;
        for example in development {
            for (mean, target) in distribution
                .iter_mut()
                .zip(example.target.volatility_regime.unwrap())
            {
                *mean += target / count;
            }
            variance += example.target.future_variance.unwrap() / count;
            risk += example.target.risk_abstention.unwrap() / count;
        }
        Ok(vec![
            inverse_softplus(variance),
            distribution[0].max(PROBABILITY_EPSILON).ln(),
            distribution[1].max(PROBABILITY_EPSILON).ln(),
            distribution[2].max(PROBABILITY_EPSILON).ln(),
            probability_logit(risk),
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
        let volatility = last[0].clamp(-8.0, 8.0).abs();
        let second = last[1].clamp(-8.0, 8.0);
        Ok(vec![
            inverse_softplus(volatility + PROBABILITY_EPSILON),
            -volatility,
            1.0 - volatility,
            volatility,
            volatility + second.abs(),
        ])
    }
}
