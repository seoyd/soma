//! ReversalDistortion feature, target, loss, and output policy.

use super::{
    m3_micro::{
        AbstentionPolicy, AgentId, ConfidencePolicy, FormulaId, LossPolicy,
        M3MicroDevelopmentExample, M3MicroError, M3MicroPrediction, M3MicroTarget,
        PROBABILITY_EPSILON, TargetPolicy, bce_loss_and_gradient, distribution_loss_and_gradient,
        empty_prediction, inverse_tanh, probability_logit, sigmoid, softmax, valid_distribution,
        valid_probability,
    },
    m3_micro_capability::{M3MicroRolePolicyV1, RoleFormulaSelectionV1, RolePolicyDescriptorV1},
};

pub(crate) struct ReversalRolePolicyV1;
pub(crate) const REVERSAL_ROLE_POLICY_V1: ReversalRolePolicyV1 = ReversalRolePolicyV1;

impl M3MicroRolePolicyV1 for ReversalRolePolicyV1 {
    fn descriptor(&self) -> RolePolicyDescriptorV1 {
        RolePolicyDescriptorV1 {
            agent_id: AgentId::ReversalDistortion,
            output_dim: 6,
            target_policy: TargetPolicy::ReversalDistortionAndDirection,
            loss_policy: LossPolicy::ReversalReturnRpsAndCalibration,
            confidence_policy: ConfidencePolicy::ReversalOrDirectionMaximum,
            abstention_policy: AbstentionPolicy::NeutralOrLowReversalConfidence,
        }
    }

    fn formulas(&self) -> RoleFormulaSelectionV1 {
        RoleFormulaSelectionV1 {
            active: vec![
                FormulaId::ReturnZScore20,
                FormulaId::PriceDeviation20,
                FormulaId::WickRejectionStructure,
                FormulaId::FailedBreakout20,
                FormulaId::VolumeExhaustion20,
                FormulaId::ShortHorizonReversal,
                FormulaId::RangeNormalizedDisplacement,
                FormulaId::LogReturn1,
            ],
            rejected: vec![
                FormulaId::LiquidityDistortion,
                FormulaId::OrderFlowImbalance,
            ],
        }
    }

    fn validate_target(&self, target: &M3MicroTarget) -> bool {
        target.agent_id == AgentId::ReversalDistortion
            && valid_probability(target.reversal)
            && valid_probability(target.failed_breakout)
            && target.future_return.is_some_and(f32::is_finite)
            && valid_distribution(target.direction_distribution)
            && target.continuation.is_none()
            && target.future_variance.is_none()
            && target.volatility_regime.is_none()
            && target.risk_abstention.is_none()
    }

    fn prediction_from_raw(&self, raw: &[f32]) -> Result<M3MicroPrediction, M3MicroError> {
        if raw.len() != 6 || raw.iter().any(|value| !value.is_finite()) {
            return Err(M3MicroError::InvalidShape);
        }
        let distribution = softmax(&raw[3..6])?;
        let reversal = sigmoid(raw[0]);
        let mut prediction = empty_prediction(AgentId::ReversalDistortion);
        prediction.reversal_probability = Some(reversal);
        prediction.failed_breakout_probability = Some(sigmoid(raw[1]));
        prediction.expected_return = Some(raw[2].tanh());
        prediction.direction_distribution =
            Some([distribution[0], distribution[1], distribution[2]]);
        prediction.confidence =
            reversal.max(distribution.iter().copied().reduce(f32::max).unwrap());
        prediction.abstain =
            distribution[1] >= distribution[0].max(distribution[2]) || reversal < 0.5;
        Ok(prediction)
    }

    fn loss_and_output_gradient(
        &self,
        raw: &[f32],
        target: &M3MicroTarget,
    ) -> Result<(f32, Vec<f32>), M3MicroError> {
        if raw.len() != 6 || !self.validate_target(target) {
            return Err(M3MicroError::InvalidShape);
        }
        let calibration = 0.05;
        let (reversal_loss, reversal_gradient) =
            bce_loss_and_gradient(raw[0], target.reversal.unwrap(), calibration);
        let (breakout_loss, breakout_gradient) =
            bce_loss_and_gradient(raw[1], target.failed_breakout.unwrap(), calibration);
        let prediction = raw[2].tanh();
        let residual = prediction - target.future_return.unwrap();
        let (direction_loss, direction_gradient) = distribution_loss_and_gradient(
            &raw[3..6],
            &target.direction_distribution.unwrap(),
            calibration,
        )?;
        let mut gradient = vec![0.0; 6];
        gradient[0] = reversal_gradient;
        gradient[1] = breakout_gradient;
        gradient[2] = 2.0 * residual * (1.0 - prediction * prediction);
        gradient[3..6].copy_from_slice(&direction_gradient);
        Ok((
            reversal_loss + breakout_loss + residual * residual + direction_loss,
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
        let mut reversal = 0.0;
        let mut breakout = 0.0;
        let mut future_return = 0.0;
        for example in development {
            for (mean, target) in distribution
                .iter_mut()
                .zip(example.target.direction_distribution.unwrap())
            {
                *mean += target / count;
            }
            reversal += example.target.reversal.unwrap() / count;
            breakout += example.target.failed_breakout.unwrap() / count;
            future_return += example.target.future_return.unwrap() / count;
        }
        Ok(vec![
            probability_logit(reversal),
            probability_logit(breakout),
            inverse_tanh(future_return),
            distribution[0].max(PROBABILITY_EPSILON).ln(),
            distribution[1].max(PROBABILITY_EPSILON).ln(),
            distribution[2].max(PROBABILITY_EPSILON).ln(),
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
        let second = last[1].clamp(-8.0, 8.0);
        Ok(vec![
            first.abs() + second,
            second.abs(),
            -first,
            first,
            -first.abs(),
            -first,
        ])
    }
}
