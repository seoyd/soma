//! Versioned functional contract and role-policy composition for M3-Micro.
//!
//! The recurrent mathematics remains in `m3_micro`. Role modules own only
//! feature, target, loss, and output interpretation policy.

use serde::{Deserialize, Serialize};

use crate::core::stable_hash_string;

use super::{
    m3_micro::{
        AbstentionPolicy, AgentId, ConfidencePolicy, FormulaId, LossPolicy,
        M3MicroDevelopmentExample, M3MicroError, M3MicroPrediction, M3MicroTarget, TargetPolicy,
    },
    m3_micro_reversal::REVERSAL_ROLE_POLICY_V1,
    m3_micro_trend::TREND_ROLE_POLICY_V1,
    m3_micro_volatility::VOLATILITY_ROLE_POLICY_V1,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityRequirementV1 {
    Required,
    Optional,
    IntentionallyExcluded,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct M3MicroCapabilityContractV1 {
    pub causal_streaming: CapabilityRequirementV1,
    pub linear_sequence_work: CapabilityRequirementV1,
    pub input_dependent_retention: CapabilityRequirementV1,
    pub input_dependent_state_update: CapabilityRequirementV1,
    pub long_range_signal_retention: CapabilityRequirementV1,
    pub selective_forgetting: CapabilityRequirementV1,
    pub bounded_finite_state: CapabilityRequirementV1,
    pub end_to_end_trainability: CapabilityRequirementV1,
    pub deterministic_replay: CapabilityRequirementV1,
    pub per_agent_state_isolation: CapabilityRequirementV1,
    pub contract_digest: String,
}

pub fn m3_micro_capability_contract_v1() -> M3MicroCapabilityContractV1 {
    let required = CapabilityRequirementV1::Required;
    let mut contract = M3MicroCapabilityContractV1 {
        causal_streaming: required,
        linear_sequence_work: required,
        input_dependent_retention: required,
        input_dependent_state_update: required,
        long_range_signal_retention: required,
        selective_forgetting: required,
        bounded_finite_state: required,
        end_to_end_trainability: required,
        deterministic_replay: required,
        per_agent_state_isolation: required,
        contract_digest: String::new(),
    };
    contract.contract_digest = stable_hash_string(&format!(
        "m3-micro-capability-contract-v1:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}",
        contract.causal_streaming,
        contract.linear_sequence_work,
        contract.input_dependent_retention,
        contract.input_dependent_state_update,
        contract.long_range_signal_retention,
        contract.selective_forgetting,
        contract.bounded_finite_state,
        contract.end_to_end_trainability,
        contract.deterministic_replay,
        contract.per_agent_state_isolation,
    ));
    contract
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExclusionReasonV1 {
    NotRequiredForInvestmentSequence,
    NoCurrentConsumer,
    ResourceCostNotJustified,
    OutsideMicroScope,
    RequiresSeparateResearch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentionallyExcludedCapabilityV1 {
    pub capability_identity: String,
    pub reason: ExclusionReasonV1,
    pub required_by_active_agent: bool,
}

pub fn intentionally_excluded_m3_micro_capabilities_v1() -> Vec<IntentionallyExcludedCapabilityV1> {
    vec![
        IntentionallyExcludedCapabilityV1 {
            capability_identity: "official-mamba3-parameter-parity".to_string(),
            reason: ExclusionReasonV1::OutsideMicroScope,
            required_by_active_agent: false,
        },
        IntentionallyExcludedCapabilityV1 {
            capability_identity: "quadratic-pairwise-sequence-interaction".to_string(),
            reason: ExclusionReasonV1::ResourceCostNotJustified,
            required_by_active_agent: false,
        },
        IntentionallyExcludedCapabilityV1 {
            capability_identity: "general-purpose-multimodal-routing".to_string(),
            reason: ExclusionReasonV1::NotRequiredForInvestmentSequence,
            required_by_active_agent: false,
        },
        IntentionallyExcludedCapabilityV1 {
            capability_identity: "shared-trainable-agent-backbone".to_string(),
            reason: ExclusionReasonV1::NoCurrentConsumer,
            required_by_active_agent: false,
        },
        IntentionallyExcludedCapabilityV1 {
            capability_identity: "official-training-performance-parity".to_string(),
            reason: ExclusionReasonV1::RequiresSeparateResearch,
            required_by_active_agent: false,
        },
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum M3MicroConformanceStatusV1 {
    Conformant,
    PartiallyConformant,
    NonConformant,
    NotVerified,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct M3MicroWorkCountersV1 {
    pub state_transition_count: usize,
    pub projection_operation_count: usize,
    pub sequence_step_count: usize,
    /// Compatibility field: this counts zero-state initialization performed by a helper.
    /// It does not count heap allocations made inside the recurrent implementation.
    pub state_allocation_count: usize,
}

impl M3MicroWorkCountersV1 {
    pub fn checked_add(&mut self, other: &Self) -> Result<(), M3MicroError> {
        self.state_transition_count = self
            .state_transition_count
            .checked_add(other.state_transition_count)
            .ok_or(M3MicroError::StateExplosion)?;
        self.projection_operation_count = self
            .projection_operation_count
            .checked_add(other.projection_operation_count)
            .ok_or(M3MicroError::StateExplosion)?;
        self.sequence_step_count = self
            .sequence_step_count
            .checked_add(other.sequence_step_count)
            .ok_or(M3MicroError::StateExplosion)?;
        self.state_allocation_count = self
            .state_allocation_count
            .checked_add(other.state_allocation_count)
            .ok_or(M3MicroError::StateExplosion)?;
        Ok(())
    }

    pub fn precise_semantics_v2(&self) -> M3MicroWorkCountersV2 {
        M3MicroWorkCountersV2 {
            state_transition_count: self.state_transition_count,
            projection_operation_count: self.projection_operation_count,
            sequence_step_count: self.sequence_step_count,
            zero_state_initialization_count: self.state_allocation_count,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct M3MicroWorkCountersV2 {
    pub state_transition_count: usize,
    pub projection_operation_count: usize,
    pub sequence_step_count: usize,
    pub zero_state_initialization_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DelayedRecallEvidencePolicyV2 {
    pub sequence_lengths: [usize; 3],
    pub balanced_classes: bool,
    pub fixed_training_budget: usize,
    pub frozen_evaluation_examples_per_class: usize,
    pub minimum_accuracy: f32,
    pub minimum_base_vs_no_state_accuracy_gap: f32,
    pub minimum_carried_prediction_separation: f32,
    pub maximum_reset_prediction_separation: f32,
}

pub fn delayed_recall_evidence_policy_v2() -> DelayedRecallEvidencePolicyV2 {
    DelayedRecallEvidencePolicyV2 {
        sequence_lengths: [8, 16, 32],
        balanced_classes: true,
        fixed_training_budget: 6,
        frozen_evaluation_examples_per_class: 2,
        minimum_accuracy: 0.75,
        minimum_base_vs_no_state_accuracy_gap: 0.25,
        minimum_carried_prediction_separation: 0.005,
        maximum_reset_prediction_separation: 1e-6,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidationIndexPolicyV2 {
    Fixed(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelectiveForgettingEvidencePolicyV2 {
    pub sequence_length: usize,
    pub invalidation_index_policy: InvalidationIndexPolicyV2,
    pub fixed_training_budget: usize,
    pub evaluation_examples_per_case: usize,
    pub minimum_preserve_accuracy: f32,
    pub minimum_noise_accuracy: f32,
    pub minimum_invalidation_accuracy: f32,
    pub minimum_base_vs_no_forgetting_gap: f32,
}

pub fn selective_forgetting_evidence_policy_v2() -> SelectiveForgettingEvidencePolicyV2 {
    SelectiveForgettingEvidencePolicyV2 {
        sequence_length: 16,
        invalidation_index_policy: InvalidationIndexPolicyV2::Fixed(11),
        fixed_training_budget: 6,
        evaluation_examples_per_case: 2,
        minimum_preserve_accuracy: 0.75,
        minimum_noise_accuracy: 0.75,
        minimum_invalidation_accuracy: 0.75,
        minimum_base_vs_no_forgetting_gap: 0.25,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NumericalGradientConformancePolicyV1 {
    pub relative_step_scales: [f32; 3],
    pub absolute_tolerance: f32,
    pub relative_tolerance: f32,
    pub sign_check_floor: f32,
    pub boundary_output_tolerance: f32,
    pub minimum_stable_step_pair: usize,
    pub single_step_sequence_length: usize,
    pub multi_step_sequence_length: usize,
}

pub fn numerical_gradient_conformance_policy_v1() -> NumericalGradientConformancePolicyV1 {
    NumericalGradientConformancePolicyV1 {
        relative_step_scales: [1.0e-2, 5.0e-3, 2.5e-3],
        absolute_tolerance: 3.0e-3,
        relative_tolerance: 7.5e-2,
        sign_check_floor: 1.0e-5,
        boundary_output_tolerance: 0.05,
        minimum_stable_step_pair: 2,
        single_step_sequence_length: 1,
        multi_step_sequence_length: 4,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidationPositionV3 {
    Early,
    Middle,
    Late,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelectiveForgettingEvidencePolicyV3 {
    pub sequence_lengths: [usize; 2],
    pub invalidation_positions: [InvalidationPositionV3; 3],
    pub seed_identities: [String; 3],
    pub development_examples_per_class: usize,
    pub frozen_examples_per_class: usize,
    pub fixed_training_budget: usize,
    pub minimum_preserve_accuracy: f32,
    pub minimum_noise_accuracy: f32,
    pub minimum_invalidation_accuracy: f32,
    pub minimum_per_seed_invalidation_accuracy: f32,
    pub minimum_accuracy_gap: f32,
    pub minimum_mean_nll_improvement: f32,
    pub minimum_target_margin_improvement: f32,
    pub minimum_paired_win_rate: f32,
}

pub fn selective_forgetting_evidence_policy_v3() -> SelectiveForgettingEvidencePolicyV3 {
    SelectiveForgettingEvidencePolicyV3 {
        sequence_lengths: [16, 32],
        invalidation_positions: [
            InvalidationPositionV3::Early,
            InvalidationPositionV3::Middle,
            InvalidationPositionV3::Late,
        ],
        seed_identities: [
            "m3-r2-seed-a".to_string(),
            "m3-r2-seed-b".to_string(),
            "m3-r2-seed-c".to_string(),
        ],
        development_examples_per_class: 16,
        frozen_examples_per_class: 32,
        fixed_training_budget: 6,
        minimum_preserve_accuracy: 0.90,
        minimum_noise_accuracy: 0.90,
        minimum_invalidation_accuracy: 0.80,
        minimum_per_seed_invalidation_accuracy: 0.70,
        minimum_accuracy_gap: 0.10,
        minimum_mean_nll_improvement: 0.02,
        minimum_target_margin_improvement: 0.02,
        minimum_paired_win_rate: 0.60,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndeterminateActionV2 {
    Revert,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReinforcementAttributionPolicyV2 {
    pub minimum_accuracy_gain: f32,
    pub minimum_mean_nll_gain: f32,
    pub minimum_target_margin_gain: f32,
    pub maximum_protected_accuracy_regression: f32,
    pub maximum_protected_nll_regression: f32,
    pub require_full_support_panel: bool,
    pub indeterminate_action: IndeterminateActionV2,
}

pub fn reinforcement_attribution_policy_v2() -> ReinforcementAttributionPolicyV2 {
    ReinforcementAttributionPolicyV2 {
        minimum_accuracy_gain: 0.02,
        minimum_mean_nll_gain: 0.01,
        minimum_target_margin_gain: 0.02,
        maximum_protected_accuracy_regression: 0.01,
        maximum_protected_nll_regression: 0.01,
        require_full_support_panel: true,
        indeterminate_action: IndeterminateActionV2::Revert,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateExecutionStatusV1 {
    NotRun,
    Passed,
    Failed,
}

fn gate_execution_not_run_v1() -> GateExecutionStatusV1 {
    GateExecutionStatusV1::NotRun
}

fn gate_execution_is_not_run_v1(status: &GateExecutionStatusV1) -> bool {
    *status == GateExecutionStatusV1::NotRun
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct M3MicroEvidenceLifecycleV2 {
    pub initialization_oracle: GateExecutionStatusV1,
    pub attribution: GateExecutionStatusV1,
    pub final_gradient: GateExecutionStatusV1,
    pub metric_semantics: GateExecutionStatusV1,
    #[serde(
        default = "gate_execution_not_run_v1",
        skip_serializing_if = "gate_execution_is_not_run_v1"
    )]
    pub structural_loss: GateExecutionStatusV1,
    pub event_alignment: GateExecutionStatusV1,
    pub comparator_purity: GateExecutionStatusV1,
    pub suffix_integrity: GateExecutionStatusV1,
    pub protected_interaction: GateExecutionStatusV1,
    pub frozen_v3: GateExecutionStatusV1,
    pub report_integrity: GateExecutionStatusV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum M3MicroGradientGroupV1 {
    InputProjection,
    RetentionDecay,
    PreviousInputGate,
    CurrentInputGate,
    StateInjectionScale,
    StateReadout,
    SkipPath,
    OutputProjection,
    PredictionHead,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroGradientGroupEvidenceV1 {
    pub group: M3MicroGradientGroupV1,
    pub parameter_count: usize,
    pub nonzero_gradient_count: usize,
    pub gradient_l1_norm: f32,
    pub all_finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroGradientAuditV1 {
    pub loss: f32,
    pub training_cache_bytes: usize,
    pub earliest_input_gradient_l1: f32,
    pub groups: Vec<M3MicroGradientGroupEvidenceV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum M3MicroAblationV1 {
    FixedRetention,
    NoPreviousInputContribution,
    NoCurrentInputGate,
    NoRecurrentState,
    NoSelectiveForgetting,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroStepCapabilityProbeV1 {
    pub mean_retention: f32,
    pub mean_previous_gate: f32,
    pub mean_current_gate: f32,
    pub previous_input_contribution_l1: f32,
    pub current_input_contribution_l1: f32,
    pub updated_state_digest: String,
    pub raw_output: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RoleFormulaSelectionV1 {
    pub active: Vec<FormulaId>,
    pub rejected: Vec<FormulaId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RolePolicyDescriptorV1 {
    pub agent_id: AgentId,
    pub output_dim: usize,
    pub target_policy: TargetPolicy,
    pub loss_policy: LossPolicy,
    pub confidence_policy: ConfidencePolicy,
    pub abstention_policy: AbstentionPolicy,
}

pub(crate) trait M3MicroRolePolicyV1: Sync {
    fn descriptor(&self) -> RolePolicyDescriptorV1;
    fn formulas(&self) -> RoleFormulaSelectionV1;
    fn validate_target(&self, target: &M3MicroTarget) -> bool;
    fn prediction_from_raw(&self, raw: &[f32]) -> Result<M3MicroPrediction, M3MicroError>;
    fn loss_and_output_gradient(
        &self,
        raw: &[f32],
        target: &M3MicroTarget,
    ) -> Result<(f32, Vec<f32>), M3MicroError>;
    fn constant_baseline_raw(
        &self,
        development: &[&M3MicroDevelopmentExample],
    ) -> Result<Vec<f32>, M3MicroError>;
    fn mathematical_baseline_raw(
        &self,
        normalized_sequence: &[Vec<f32>],
    ) -> Result<Vec<f32>, M3MicroError>;
}

pub(crate) fn role_policy_v1(agent_id: AgentId) -> &'static dyn M3MicroRolePolicyV1 {
    match agent_id {
        AgentId::TrendContinuation => &TREND_ROLE_POLICY_V1,
        AgentId::VolatilityRegime => &VOLATILITY_ROLE_POLICY_V1,
        AgentId::ReversalDistortion => &REVERSAL_ROLE_POLICY_V1,
    }
}

pub fn role_policy_descriptor_v1(agent_id: AgentId) -> (AgentId, usize) {
    let descriptor = role_policy_v1(agent_id).descriptor();
    (descriptor.agent_id, descriptor.output_dim)
}
