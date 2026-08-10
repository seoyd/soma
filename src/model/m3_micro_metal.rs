//! Actual Apple Metal execution for the production M3-Micro forward pass.
//!
//! This module is a child of `m3_micro`, so it shares the CPU model's exact
//! parameter and recurrent-state semantics without exposing mutable internals.

use super::*;
use metal::{
    CommandQueue, CompileOptions, ComputePipelineState, Device, MTLCommandBufferStatus,
    MTLResourceOptions, MTLSize,
};
use serde::{Deserialize, Serialize};

const METAL_LAYOUT_VERSION_V1: u32 = 1;
const METAL_FUNCTION_IDENTITY_V1: &str = "m3_micro_forward_v1";
const METAL_PIPELINE_ARTIFACT_SCHEMA_VERSION_V1: u32 = 1;
const METAL_GPU_WITNESS_MAGIC_V1: u32 = 0x4d33_4d31;
const METAL_POISON_BITS_V1: u32 = 0x7f7f_ffff;
const METAL_WITNESS_WORDS_V1: usize = 6;
const METADATA_PREFIX_WORDS_V1: usize = 15;
const METADATA_BLOCK_WORDS_V1: usize = 15;
const MAX_METAL_D_MODEL_V1: usize = 64;
const MAX_METAL_INNER_DIM_V1: usize = 128;

const METAL_LIBRARY_BUILD_POLICY_V1: &str =
    "m3-micro-metal-library-build-v1:runtime-source:fast-math=false";
const METAL_FUNCTION_LOOKUP_POLICY_V1: &str =
    "m3-micro-metal-function-lookup-v1:exact-name:no-function-constants";
const METAL_FUNCTION_CONSTANT_IDENTITY_V1: &str = "m3-micro-metal-function-constants-v1:none";
const METAL_KERNEL_ABI_IDENTITY_V1: &str =
    "m3-micro-metal-kernel-abi-v1:eight-buffers:grid-position-u32";
const METAL_PARAMETER_BUFFER_LAYOUT_IDENTITY_V1: &str = "m3-micro-metal-buffer-layout-v1:input0:parameters1:initial-state2:output3:final-state4:metadata5:scalar6:witness7";

pub const M3_MICRO_METAL_SHADER_V1: &str = r#"
#include <metal_stdlib>
using namespace metal;

constant uint META_LAYOUT_VERSION = 0;
constant uint META_INPUT_DIM = 1;
constant uint META_D_MODEL = 2;
constant uint META_D_STATE = 3;
constant uint META_BLOCK_COUNT = 4;
constant uint META_INNER_DIM = 5;
constant uint META_OUTPUT_DIM = 6;
constant uint META_SEQUENCE_LENGTH = 7;
constant uint META_PARAMETER_COUNT = 8;
constant uint META_STATE_WIDTH = 9;
constant uint META_INITIAL_STEP = 10;
constant uint META_W_EMBED = 11;
constant uint META_B_EMBED = 12;
constant uint META_W_HEAD = 13;
constant uint META_B_HEAD = 14;
constant uint META_BLOCK_PREFIX = 15;
constant uint META_BLOCK_WORDS = 15;

constant uint BLOCK_W_IN = 0;
constant uint BLOCK_B_IN = 1;
constant uint BLOCK_W_DECAY = 2;
constant uint BLOCK_B_DECAY = 3;
constant uint BLOCK_W_PREV_GATE = 4;
constant uint BLOCK_B_PREV_GATE = 5;
constant uint BLOCK_W_CURR_GATE = 6;
constant uint BLOCK_B_CURR_GATE = 7;
constant uint BLOCK_DECAY_STATE_BIAS = 8;
constant uint BLOCK_PREV_SCALE = 9;
constant uint BLOCK_CURR_SCALE = 10;
constant uint BLOCK_READOUT_SCALE = 11;
constant uint BLOCK_SKIP = 12;
constant uint BLOCK_W_OUT = 13;
constant uint BLOCK_B_OUT = 14;

inline float m3_sigmoid_v1(float value) {
    if (value >= 0.0f) {
        return 1.0f / (1.0f + exp(-value));
    }
    float exponential = exp(value);
    return exponential / (1.0f + exponential);
}

kernel void m3_micro_forward_v1(
    device const float* input [[buffer(0)]],
    device const float* parameters [[buffer(1)]],
    device const float* initial_state [[buffer(2)]],
    device float* output [[buffer(3)]],
    device float* final_state [[buffer(4)]],
    constant uint* metadata [[buffer(5)]],
    constant float* scalar_config [[buffer(6)]],
    device uint* witness [[buffer(7)]],
    uint gid [[thread_position_in_grid]]) {
    if (gid != 0) {
        return;
    }

    const uint input_dim = metadata[META_INPUT_DIM];
    const uint d_model = metadata[META_D_MODEL];
    const uint d_state = metadata[META_D_STATE];
    const uint block_count = metadata[META_BLOCK_COUNT];
    const uint inner = metadata[META_INNER_DIM];
    const uint output_dim = metadata[META_OUTPUT_DIM];
    const uint sequence_length = metadata[META_SEQUENCE_LENGTH];
    const uint state_width = metadata[META_STATE_WIDTH];
    const float decay_min = scalar_config[0];
    const float decay_max = scalar_config[1];

    for (uint index = 0; index < state_width; ++index) {
        final_state[index] = initial_state[index];
    }

    thread float hidden[64];
    thread float next_hidden[64];
    thread float u[128];
    thread float decay_channel[128];
    thread float previous_gate[128];
    thread float current_gate[128];
    thread float z[128];

    for (uint step = 0; step < sequence_length; ++step) {
        const uint input_base = step * input_dim;
        for (uint row = 0; row < d_model; ++row) {
            float value = parameters[metadata[META_B_EMBED] + row];
            const uint weight_base = metadata[META_W_EMBED] + row * input_dim;
            for (uint column = 0; column < input_dim; ++column) {
                value += parameters[weight_base + column] * input[input_base + column];
            }
            hidden[row] = tanh(value);
        }

        for (uint block = 0; block < block_count; ++block) {
            const uint offset_base = META_BLOCK_PREFIX + block * META_BLOCK_WORDS;
            const uint state_base = block * (inner * d_state + inner);
            const uint previous_u_base = state_base + inner * d_state;

            for (uint row = 0; row < inner; ++row) {
                float value = parameters[metadata[offset_base + BLOCK_B_IN] + row];
                const uint weight_base = metadata[offset_base + BLOCK_W_IN] + row * d_model;
                for (uint column = 0; column < d_model; ++column) {
                    value += parameters[weight_base + column] * hidden[column];
                }
                u[row] = tanh(value);
            }

            for (uint row = 0; row < inner; ++row) {
                float decay_value = parameters[metadata[offset_base + BLOCK_B_DECAY] + row];
                float previous_value = parameters[metadata[offset_base + BLOCK_B_PREV_GATE] + row];
                float current_value = parameters[metadata[offset_base + BLOCK_B_CURR_GATE] + row];
                const uint decay_base = metadata[offset_base + BLOCK_W_DECAY] + row * inner;
                const uint previous_base = metadata[offset_base + BLOCK_W_PREV_GATE] + row * inner;
                const uint current_base = metadata[offset_base + BLOCK_W_CURR_GATE] + row * inner;
                for (uint column = 0; column < inner; ++column) {
                    decay_value += parameters[decay_base + column] * u[column];
                    previous_value += parameters[previous_base + column] * u[column];
                    current_value += parameters[current_base + column] * u[column];
                }
                decay_channel[row] = decay_value;
                previous_gate[row] = m3_sigmoid_v1(previous_value);
                current_gate[row] = m3_sigmoid_v1(current_value);
            }

            for (uint channel = 0; channel < inner; ++channel) {
                float readout = 0.0f;
                for (uint state_index = 0; state_index < d_state; ++state_index) {
                    const uint local_index = channel * d_state + state_index;
                    const uint state_index_flat = state_base + local_index;
                    const float bounded = m3_sigmoid_v1(
                        parameters[metadata[offset_base + BLOCK_DECAY_STATE_BIAS] + local_index]
                            + decay_channel[channel]);
                    const float decay = decay_min + (decay_max - decay_min) * bounded;
                    const float previous_scale = tanh(
                        parameters[metadata[offset_base + BLOCK_PREV_SCALE] + local_index]);
                    const float current_scale = tanh(
                        parameters[metadata[offset_base + BLOCK_CURR_SCALE] + local_index]);
                    const float next = decay * final_state[state_index_flat]
                        + previous_gate[channel] * final_state[previous_u_base + channel]
                            * previous_scale
                        + current_gate[channel] * u[channel] * current_scale;
                    final_state[state_index_flat] = next;
                    readout += next
                        * tanh(parameters[metadata[offset_base + BLOCK_READOUT_SCALE] + local_index])
                        / float(d_state);
                }
                z[channel] = tanh(readout
                    + m3_sigmoid_v1(parameters[metadata[offset_base + BLOCK_SKIP] + channel])
                        * u[channel]);
            }

            for (uint channel = 0; channel < inner; ++channel) {
                final_state[previous_u_base + channel] = u[channel];
            }

            for (uint row = 0; row < d_model; ++row) {
                float value = parameters[metadata[offset_base + BLOCK_B_OUT] + row];
                const uint weight_base = metadata[offset_base + BLOCK_W_OUT] + row * inner;
                for (uint column = 0; column < inner; ++column) {
                    value += parameters[weight_base + column] * z[column];
                }
                next_hidden[row] = tanh(value);
            }
            for (uint row = 0; row < d_model; ++row) {
                hidden[row] = next_hidden[row];
            }
        }
    }

    for (uint row = 0; row < output_dim; ++row) {
        float value = parameters[metadata[META_B_HEAD] + row];
        const uint weight_base = metadata[META_W_HEAD] + row * d_model;
        for (uint column = 0; column < d_model; ++column) {
            value += parameters[weight_base + column] * hidden[column];
        }
        output[row] = value;
    }

    witness[0] = 0x4d334d31u;
    witness[1] = output_dim;
    witness[2] = state_width;
    witness[3] = sequence_length;
    witness[4] = metadata[META_INITIAL_STEP] + sequence_length;
    witness[5] = 0u;
}
"#;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct M3MicroMetalParameterOffsetsV1 {
    pub w_in: Range<usize>,
    pub b_in: Range<usize>,
    pub w_decay: Range<usize>,
    pub b_decay: Range<usize>,
    pub w_prev_gate: Range<usize>,
    pub b_prev_gate: Range<usize>,
    pub w_curr_gate: Range<usize>,
    pub b_curr_gate: Range<usize>,
    pub decay_state_bias: Range<usize>,
    pub prev_scale: Range<usize>,
    pub curr_scale: Range<usize>,
    pub readout_scale: Range<usize>,
    pub skip: Range<usize>,
    pub w_out: Range<usize>,
    pub b_out: Range<usize>,
}

impl From<&BlockLayout> for M3MicroMetalParameterOffsetsV1 {
    fn from(value: &BlockLayout) -> Self {
        Self {
            w_in: value.w_in.clone(),
            b_in: value.b_in.clone(),
            w_decay: value.w_decay.clone(),
            b_decay: value.b_decay.clone(),
            w_prev_gate: value.w_prev_gate.clone(),
            b_prev_gate: value.b_prev_gate.clone(),
            w_curr_gate: value.w_curr_gate.clone(),
            b_curr_gate: value.b_curr_gate.clone(),
            decay_state_bias: value.decay_state_bias.clone(),
            prev_scale: value.prev_scale.clone(),
            curr_scale: value.curr_scale.clone(),
            readout_scale: value.readout_scale.clone(),
            skip: value.skip.clone(),
            w_out: value.w_out.clone(),
            b_out: value.b_out.clone(),
        }
    }
}

impl M3MicroMetalParameterOffsetsV1 {
    fn ranges(&self) -> [&Range<usize>; 15] {
        [
            &self.w_in,
            &self.b_in,
            &self.w_decay,
            &self.b_decay,
            &self.w_prev_gate,
            &self.b_prev_gate,
            &self.w_curr_gate,
            &self.b_curr_gate,
            &self.decay_state_bias,
            &self.prev_scale,
            &self.curr_scale,
            &self.readout_scale,
            &self.skip,
            &self.w_out,
            &self.b_out,
        ]
    }

    fn starts(&self) -> [u32; 15] {
        [
            self.w_in.start,
            self.b_in.start,
            self.w_decay.start,
            self.b_decay.start,
            self.w_prev_gate.start,
            self.b_prev_gate.start,
            self.w_curr_gate.start,
            self.b_curr_gate.start,
            self.decay_state_bias.start,
            self.prev_scale.start,
            self.curr_scale.start,
            self.readout_scale.start,
            self.skip.start,
            self.w_out.start,
            self.b_out.start,
        ]
        .map(|value| value as u32)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct M3MicroMetalLayoutV1 {
    pub layout_version: u32,
    pub input_width: usize,
    pub state_width: usize,
    pub output_width: usize,
    pub block_count: usize,
    pub total_parameter_count: usize,
    pub w_embed: Range<usize>,
    pub b_embed: Range<usize>,
    pub parameter_offsets: Vec<M3MicroMetalParameterOffsetsV1>,
    pub w_head: Range<usize>,
    pub b_head: Range<usize>,
    pub layout_identity: String,
}

impl M3MicroMetalLayoutV1 {
    pub fn from_config(config: &M3MicroConfig) -> Result<Self, M3MicroMetalErrorV1> {
        config
            .validate()
            .map_err(|_| M3MicroMetalErrorV1::MetalShapeMismatch)?;
        if config.d_model > MAX_METAL_D_MODEL_V1
            || config.inner_dim() > MAX_METAL_INNER_DIM_V1
            || config.block_count != 2
        {
            return Err(M3MicroMetalErrorV1::MetalShapeMismatch);
        }
        let cpu =
            M3MicroLayout::new(config).map_err(|_| M3MicroMetalErrorV1::MetalShapeMismatch)?;
        let state_width =
            config.block_count * (config.inner_dim() * config.d_state + config.inner_dim());
        let parameter_offsets = cpu
            .blocks
            .iter()
            .map(M3MicroMetalParameterOffsetsV1::from)
            .collect::<Vec<_>>();
        let layout_identity = m3_micro_metal_layout_semantic_identity_v1(config)
            .map_err(|_| M3MicroMetalErrorV1::MetalShapeMismatch)?;
        let mut layout = Self {
            layout_version: METAL_LAYOUT_VERSION_V1,
            input_width: config.input_dim,
            state_width,
            output_width: config.output_dim,
            block_count: config.block_count,
            total_parameter_count: cpu.parameter_count,
            w_embed: cpu.w_embed,
            b_embed: cpu.b_embed,
            parameter_offsets,
            w_head: cpu.w_head,
            b_head: cpu.b_head,
            layout_identity,
        };
        layout.validate(config)?;
        Ok(layout)
    }

    pub fn validate(&mut self, config: &M3MicroConfig) -> Result<(), M3MicroMetalErrorV1> {
        let cpu =
            M3MicroLayout::new(config).map_err(|_| M3MicroMetalErrorV1::MetalShapeMismatch)?;
        let mut coverage = vec![0u8; self.total_parameter_count];
        let mut mark = |range: &Range<usize>| -> Result<(), M3MicroMetalErrorV1> {
            if range.start > range.end || range.end > coverage.len() {
                return Err(M3MicroMetalErrorV1::MetalShapeMismatch);
            }
            for value in &mut coverage[range.clone()] {
                *value = value
                    .checked_add(1)
                    .ok_or(M3MicroMetalErrorV1::MetalShapeMismatch)?;
            }
            Ok(())
        };
        mark(&self.w_embed)?;
        mark(&self.b_embed)?;
        for block in &self.parameter_offsets {
            for range in block.ranges() {
                mark(range)?;
            }
        }
        mark(&self.w_head)?;
        mark(&self.b_head)?;
        if self.layout_version != METAL_LAYOUT_VERSION_V1
            || self.input_width != config.input_dim
            || self.state_width
                != config.block_count * (config.inner_dim() * config.d_state + config.inner_dim())
            || self.output_width != config.output_dim
            || self.block_count != config.block_count
            || self.parameter_offsets.len() != config.block_count
            || self.total_parameter_count != cpu.parameter_count
            || coverage.iter().any(|count| *count != 1)
        {
            return Err(M3MicroMetalErrorV1::MetalShapeMismatch);
        }
        Ok(())
    }

    fn metadata(
        &self,
        config: &M3MicroConfig,
        sequence_length: usize,
        step: usize,
    ) -> Result<Vec<u32>, M3MicroMetalErrorV1> {
        let mut metadata =
            vec![0u32; METADATA_PREFIX_WORDS_V1 + self.block_count * METADATA_BLOCK_WORDS_V1];
        let convert = |value: usize| {
            u32::try_from(value).map_err(|_| M3MicroMetalErrorV1::MetalShapeMismatch)
        };
        metadata[0] = self.layout_version;
        metadata[1] = convert(config.input_dim)?;
        metadata[2] = convert(config.d_model)?;
        metadata[3] = convert(config.d_state)?;
        metadata[4] = convert(config.block_count)?;
        metadata[5] = convert(config.inner_dim())?;
        metadata[6] = convert(config.output_dim)?;
        metadata[7] = convert(sequence_length)?;
        metadata[8] = convert(self.total_parameter_count)?;
        metadata[9] = convert(self.state_width)?;
        metadata[10] = convert(step)?;
        metadata[11] = convert(self.w_embed.start)?;
        metadata[12] = convert(self.b_embed.start)?;
        metadata[13] = convert(self.w_head.start)?;
        metadata[14] = convert(self.b_head.start)?;
        for (block_index, block) in self.parameter_offsets.iter().enumerate() {
            let start = METADATA_PREFIX_WORDS_V1 + block_index * METADATA_BLOCK_WORDS_V1;
            metadata[start..start + METADATA_BLOCK_WORDS_V1].copy_from_slice(&block.starts());
        }
        Ok(metadata)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum M3MicroMetalErrorV1 {
    MetalDeviceUnavailable,
    MetalQueueCreationFailed,
    MetalLibraryCreationFailed,
    MetalFunctionNotFound,
    MetalPipelineCreationFailed,
    MetalBufferAllocationFailed,
    MetalEncodingFailed,
    MetalCommandFailed,
    MetalCommandFailedMissingDetails,
    MetalCommandCompletionContradiction,
    MetalCommandInvalidTerminalState,
    MetalOutputNotWritten,
    MetalStateNotWritten,
    MetalNonFiniteInput,
    MetalNonFiniteOutput,
    MetalShapeMismatch,
    MetalParityFailure,
    MetalCpuFallbackForbidden,
}

impl std::fmt::Display for M3MicroMetalErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for M3MicroMetalErrorV1 {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalCommandTerminalStatusV1 {
    Completed,
    Error,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalCommandCompletionObservationV1 {
    pub terminal_status: MetalCommandTerminalStatusV1,
    pub error_present: bool,
    pub error_domain: Option<String>,
    pub error_code: Option<i64>,
    pub command_committed: bool,
    pub command_completed_or_failed: bool,
}

pub(super) fn classify_command_completion_v1(
    observation: &MetalCommandCompletionObservationV1,
) -> Result<(), M3MicroMetalErrorV1> {
    if !observation.command_committed || !observation.command_completed_or_failed {
        return Err(M3MicroMetalErrorV1::MetalCommandInvalidTerminalState);
    }
    match (observation.terminal_status, observation.error_present) {
        (MetalCommandTerminalStatusV1::Completed, false) => Ok(()),
        (MetalCommandTerminalStatusV1::Error, true) => Err(M3MicroMetalErrorV1::MetalCommandFailed),
        (MetalCommandTerminalStatusV1::Error, false) => {
            Err(M3MicroMetalErrorV1::MetalCommandFailedMissingDetails)
        }
        (MetalCommandTerminalStatusV1::Completed, true) => {
            Err(M3MicroMetalErrorV1::MetalCommandCompletionContradiction)
        }
        (MetalCommandTerminalStatusV1::Other, _) => {
            Err(M3MicroMetalErrorV1::MetalCommandInvalidTerminalState)
        }
    }
}

pub(super) fn handle_command_completion_without_cpu_fallback_v1(
    observation: &MetalCommandCompletionObservationV1,
    cpu_fallback_executions: &mut usize,
) -> Result<(), M3MicroMetalErrorV1> {
    let before = *cpu_fallback_executions;
    let result = classify_command_completion_v1(observation);
    debug_assert_eq!(*cpu_fallback_executions, before);
    result
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MetalExecutionControlV1 {
    Normal,
    #[cfg(test)]
    SkipDispatch,
    #[cfg(test)]
    SuppressOutputWrite,
    #[cfg(test)]
    SuppressStateWrite,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalFaultIdentityV2 {
    MissingKernelFunction,
    SkipDispatch,
    SuppressOutputWrite,
    SuppressStateWrite,
    AttemptCpuFallback,
    CommandBufferFailure,
}

#[cfg(test)]
impl MetalFaultIdentityV2 {
    pub(super) const ALL: [Self; 6] = [
        Self::MissingKernelFunction,
        Self::SkipDispatch,
        Self::SuppressOutputWrite,
        Self::SuppressStateWrite,
        Self::AttemptCpuFallback,
        Self::CommandBufferFailure,
    ];

    fn expected_error(self) -> M3MicroMetalErrorV1 {
        match self {
            Self::MissingKernelFunction => M3MicroMetalErrorV1::MetalFunctionNotFound,
            Self::SkipDispatch | Self::SuppressOutputWrite => {
                M3MicroMetalErrorV1::MetalOutputNotWritten
            }
            Self::SuppressStateWrite => M3MicroMetalErrorV1::MetalStateNotWritten,
            Self::AttemptCpuFallback => M3MicroMetalErrorV1::MetalCpuFallbackForbidden,
            Self::CommandBufferFailure => M3MicroMetalErrorV1::MetalCommandFailed,
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalRequiredFaultInjectionV2 {
    MissingKernelFunction,
    SkipDispatch,
    SuppressOutputWrite,
    SuppressStateWrite,
    AttemptCpuFallback,
}

#[cfg(test)]
impl MetalRequiredFaultInjectionV2 {
    pub(super) const ALL: [Self; 5] = [
        Self::MissingKernelFunction,
        Self::SkipDispatch,
        Self::SuppressOutputWrite,
        Self::SuppressStateWrite,
        Self::AttemptCpuFallback,
    ];

    pub(super) fn identity(self) -> MetalFaultIdentityV2 {
        match self {
            Self::MissingKernelFunction => MetalFaultIdentityV2::MissingKernelFunction,
            Self::SkipDispatch => MetalFaultIdentityV2::SkipDispatch,
            Self::SuppressOutputWrite => MetalFaultIdentityV2::SuppressOutputWrite,
            Self::SuppressStateWrite => MetalFaultIdentityV2::SuppressStateWrite,
            Self::AttemptCpuFallback => MetalFaultIdentityV2::AttemptCpuFallback,
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalFaultRequirementV2 {
    RequiredGenuineHardware,
    OptionalPlatformDependent,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalFaultPolicyEntryV2 {
    pub fault: MetalFaultIdentityV2,
    pub requirement: MetalFaultRequirementV2,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalPolicyRevisionReasonV2 {
    NoSafeDeterministicPlatformInjection,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalNegativePolicyRevisionV2 {
    pub previous_policy_identity: String,
    pub current_policy_identity: String,
    pub changed_fault: MetalFaultIdentityV2,
    pub previous_requirement: MetalFaultRequirementV2,
    pub current_requirement: MetalFaultRequirementV2,
    pub reason: MetalPolicyRevisionReasonV2,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum OptionalMetalFaultStatusV2 {
    ObservedGenuineFailure,
    SafeInjectionUnavailable,
    FailedObservationContract,
    NotRun,
}

#[cfg(test)]
impl OptionalMetalFaultStatusV2 {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::ObservedGenuineFailure => "OBSERVED_GENUINE_FAILURE",
            Self::SafeInjectionUnavailable => "SAFE_INJECTION_UNAVAILABLE",
            Self::FailedObservationContract => "FAILED_OBSERVATION_CONTRACT",
            Self::NotRun => "NOT_RUN",
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum RequiredMetalNegativeStatusV3 {
    PassedAllRequiredGenuineFaults,
    FailedRequiredGenuineFault,
    NotRun,
}

#[cfg(test)]
impl RequiredMetalNegativeStatusV3 {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::PassedAllRequiredGenuineFaults => "PASSED_ALL_REQUIRED_GENUINE_FAULTS",
            Self::FailedRequiredGenuineFault => "FAILED_REQUIRED_GENUINE_FAULT",
            Self::NotRun => "NOT_RUN",
        }
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalNegativeEvidencePolicyV2 {
    pub entries: Vec<MetalFaultPolicyEntryV2>,
    pub require_actual_executor_path: bool,
    pub require_observed_error_match: bool,
    pub require_no_cpu_fallback_execution: bool,
    pub allow_synthetic_enum_only: bool,
}

#[cfg(test)]
impl Default for MetalNegativeEvidencePolicyV2 {
    fn default() -> Self {
        Self {
            entries: vec![
                MetalFaultPolicyEntryV2 {
                    fault: MetalFaultIdentityV2::MissingKernelFunction,
                    requirement: MetalFaultRequirementV2::RequiredGenuineHardware,
                },
                MetalFaultPolicyEntryV2 {
                    fault: MetalFaultIdentityV2::SkipDispatch,
                    requirement: MetalFaultRequirementV2::RequiredGenuineHardware,
                },
                MetalFaultPolicyEntryV2 {
                    fault: MetalFaultIdentityV2::SuppressOutputWrite,
                    requirement: MetalFaultRequirementV2::RequiredGenuineHardware,
                },
                MetalFaultPolicyEntryV2 {
                    fault: MetalFaultIdentityV2::SuppressStateWrite,
                    requirement: MetalFaultRequirementV2::RequiredGenuineHardware,
                },
                MetalFaultPolicyEntryV2 {
                    fault: MetalFaultIdentityV2::AttemptCpuFallback,
                    requirement: MetalFaultRequirementV2::RequiredGenuineHardware,
                },
                MetalFaultPolicyEntryV2 {
                    fault: MetalFaultIdentityV2::CommandBufferFailure,
                    requirement: MetalFaultRequirementV2::OptionalPlatformDependent,
                },
            ],
            require_actual_executor_path: true,
            require_observed_error_match: true,
            require_no_cpu_fallback_execution: true,
            allow_synthetic_enum_only: false,
        }
    }
}

#[cfg(test)]
impl MetalNegativeEvidencePolicyV2 {
    pub(super) fn required_faults(&self) -> Vec<MetalFaultIdentityV2> {
        self.entries
            .iter()
            .filter(|entry| entry.requirement == MetalFaultRequirementV2::RequiredGenuineHardware)
            .map(|entry| entry.fault)
            .collect()
    }

    pub(super) fn optional_faults(&self) -> Vec<MetalFaultIdentityV2> {
        self.entries
            .iter()
            .filter(|entry| entry.requirement == MetalFaultRequirementV2::OptionalPlatformDependent)
            .map(|entry| entry.fault)
            .collect()
    }

    pub(super) fn identity(&self) -> String {
        stable_hash_string(&format!("m3-micro-safe-metal-fault-policy-v2:{self:?}"))
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalFaultEvidenceV1 {
    pub fault: MetalFaultIdentityV2,
    pub device_acquired: bool,
    pub queue_created: bool,
    pub library_created: bool,
    pub function_lookup_attempted: bool,
    pub function_lookup_succeeded: bool,
    pub pipeline_created: bool,
    pub buffers_created: bool,
    pub encoder_created: bool,
    pub dispatch_attempted: bool,
    pub dispatch_performed: bool,
    pub command_buffer_committed: bool,
    pub command_buffer_completed: bool,
    pub command_buffer_failed: bool,
    pub command_status: String,
    pub command_error_present: bool,
    pub output_poison_remaining: usize,
    pub state_poison_remaining: usize,
    pub fallback_decision_seam_reached: bool,
    pub cpu_fallback_attempts: usize,
    pub cpu_fallback_executions: usize,
    pub trigger_error: Option<M3MicroMetalErrorV1>,
    pub observed_error: Option<M3MicroMetalErrorV1>,
    pub expected_error: M3MicroMetalErrorV1,
    pub genuine_fault_path_reached: bool,
    pub synthetic_enum_only: bool,
    pub safe_injection_supported: bool,
    pub unsupported_reason: Option<String>,
    pub passed: bool,
}

#[cfg(test)]
impl MetalFaultEvidenceV1 {
    fn new(fault: MetalFaultIdentityV2) -> Self {
        Self {
            fault,
            device_acquired: false,
            queue_created: false,
            library_created: false,
            function_lookup_attempted: false,
            function_lookup_succeeded: false,
            pipeline_created: false,
            buffers_created: false,
            encoder_created: false,
            dispatch_attempted: false,
            dispatch_performed: false,
            command_buffer_committed: false,
            command_buffer_completed: false,
            command_buffer_failed: false,
            command_status: "NOT_RUN".to_string(),
            command_error_present: false,
            output_poison_remaining: 0,
            state_poison_remaining: 0,
            fallback_decision_seam_reached: false,
            cpu_fallback_attempts: 0,
            cpu_fallback_executions: 0,
            trigger_error: None,
            observed_error: None,
            expected_error: fault.expected_error(),
            genuine_fault_path_reached: false,
            synthetic_enum_only: false,
            safe_injection_supported: true,
            unsupported_reason: None,
            passed: false,
        }
    }

    fn finalize(&mut self, observed_error: Option<M3MicroMetalErrorV1>) {
        self.observed_error = observed_error;
        self.genuine_fault_path_reached = match self.fault {
            MetalFaultIdentityV2::MissingKernelFunction => {
                self.device_acquired
                    && self.queue_created
                    && self.library_created
                    && self.function_lookup_attempted
                    && !self.function_lookup_succeeded
            }
            MetalFaultIdentityV2::SkipDispatch => {
                self.device_acquired
                    && self.queue_created
                    && self.pipeline_created
                    && self.buffers_created
                    && self.encoder_created
                    && self.dispatch_attempted
                    && !self.dispatch_performed
                    && self.command_buffer_committed
                    && self.command_buffer_completed
                    && !self.command_buffer_failed
                    && self.output_poison_remaining > 0
                    && self.state_poison_remaining > 0
            }
            MetalFaultIdentityV2::SuppressOutputWrite => {
                self.dispatch_performed
                    && self.command_buffer_completed
                    && !self.command_buffer_failed
                    && self.output_poison_remaining > 0
                    && self.state_poison_remaining == 0
            }
            MetalFaultIdentityV2::SuppressStateWrite => {
                self.dispatch_performed
                    && self.command_buffer_completed
                    && !self.command_buffer_failed
                    && self.output_poison_remaining == 0
                    && self.state_poison_remaining > 0
            }
            MetalFaultIdentityV2::AttemptCpuFallback => {
                self.trigger_error.is_some()
                    && self.fallback_decision_seam_reached
                    && self.cpu_fallback_attempts == 1
                    && self.cpu_fallback_executions == 0
            }
            MetalFaultIdentityV2::CommandBufferFailure => false,
        };
        self.passed = self.genuine_fault_path_reached
            && !self.synthetic_enum_only
            && self.observed_error == Some(self.expected_error)
            && self.cpu_fallback_executions == 0;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetalExecutionWitnessV1 {
    pub device_acquired: bool,
    pub queue_created: bool,
    pub pipeline_created: bool,
    pub command_buffer_count: usize,
    pub command_buffers_committed: usize,
    pub compute_encoder_count: usize,
    pub dispatches_attempted: usize,
    pub dispatch_count: usize,
    pub command_buffers_completed: usize,
    pub command_buffer_failures: usize,
    pub command_error_none: bool,
    pub output_readback_count: usize,
    pub state_readback_count: usize,
    pub output_poison_remaining: usize,
    pub state_poison_remaining: usize,
    pub gpu_witness_valid: bool,
    pub cpu_fallback_attempts: usize,
    pub cpu_fallback_count: usize,
    pub output_digest: String,
    pub final_state_digest: String,
}

impl MetalExecutionWitnessV1 {
    pub fn successful(&self) -> bool {
        self.device_acquired
            && self.queue_created
            && self.pipeline_created
            && self.command_buffer_count > 0
            && self.command_buffers_committed == self.command_buffer_count
            && self.compute_encoder_count == self.command_buffer_count
            && self.dispatches_attempted == self.command_buffer_count
            && self.dispatch_count == self.command_buffer_count
            && self.command_buffers_completed == self.command_buffer_count
            && self.command_buffer_failures == 0
            && self.command_error_none
            && self.output_readback_count == self.command_buffer_count
            && self.state_readback_count == self.command_buffer_count
            && self.output_poison_remaining == 0
            && self.state_poison_remaining == 0
            && self.gpu_witness_valid
            && self.cpu_fallback_attempts == 0
            && self.cpu_fallback_count == 0
            && !self.output_digest.is_empty()
            && !self.final_state_digest.is_empty()
    }

    pub fn merge(&mut self, value: &Self) {
        if self.command_buffer_count == 0 {
            self.device_acquired = value.device_acquired;
            self.queue_created = value.queue_created;
            self.pipeline_created = value.pipeline_created;
            self.command_error_none = value.command_error_none;
            self.gpu_witness_valid = value.gpu_witness_valid;
            self.output_digest = value.output_digest.clone();
            self.final_state_digest = value.final_state_digest.clone();
        } else {
            self.device_acquired &= value.device_acquired;
            self.queue_created &= value.queue_created;
            self.pipeline_created &= value.pipeline_created;
            self.command_error_none &= value.command_error_none;
            self.gpu_witness_valid &= value.gpu_witness_valid;
            self.output_digest =
                stable_hash_string(&format!("{}:{}", self.output_digest, value.output_digest));
            self.final_state_digest = stable_hash_string(&format!(
                "{}:{}",
                self.final_state_digest, value.final_state_digest
            ));
        }
        self.command_buffer_count += value.command_buffer_count;
        self.command_buffers_committed += value.command_buffers_committed;
        self.compute_encoder_count += value.compute_encoder_count;
        self.dispatches_attempted += value.dispatches_attempted;
        self.dispatch_count += value.dispatch_count;
        self.command_buffers_completed += value.command_buffers_completed;
        self.command_buffer_failures += value.command_buffer_failures;
        self.output_readback_count += value.output_readback_count;
        self.state_readback_count += value.state_readback_count;
        self.output_poison_remaining += value.output_poison_remaining;
        self.state_poison_remaining += value.state_poison_remaining;
        self.cpu_fallback_attempts += value.cpu_fallback_attempts;
        self.cpu_fallback_count += value.cpu_fallback_count;
    }
}

#[cfg(test)]
pub(super) const METAL_SEMANTIC_TRACE_SCHEMA_VERSION_V4: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MetalDispatchInvocationValidationErrorV1 {
    ZeroDimension,
    ConversionFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MetalDispatchInvocationV1 {
    pub grid: MTLSize,
    pub threads_per_threadgroup: MTLSize,
    pub sequence_step_ordinal: usize,
    pub chunk_ordinal: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ValidatedMetalDispatchArgumentsV1 {
    grid_threads: [usize; 3],
    threads_per_threadgroup: [usize; 3],
    sequence_step_ordinal: usize,
    chunk_ordinal: usize,
}

impl MetalDispatchInvocationV1 {
    pub(super) fn new(
        grid: MTLSize,
        threads_per_threadgroup: MTLSize,
        sequence_step_ordinal: usize,
        chunk_ordinal: usize,
    ) -> Self {
        Self {
            grid,
            threads_per_threadgroup,
            sequence_step_ordinal,
            chunk_ordinal,
        }
    }

    fn validated_arguments(
        &self,
    ) -> Result<ValidatedMetalDispatchArgumentsV1, MetalDispatchInvocationValidationErrorV1> {
        let convert = |value| {
            usize::try_from(value)
                .map_err(|_| MetalDispatchInvocationValidationErrorV1::ConversionFailure)
        };
        let grid_threads = [
            convert(self.grid.width)?,
            convert(self.grid.height)?,
            convert(self.grid.depth)?,
        ];
        let threads_per_threadgroup = [
            convert(self.threads_per_threadgroup.width)?,
            convert(self.threads_per_threadgroup.height)?,
            convert(self.threads_per_threadgroup.depth)?,
        ];
        if grid_threads.contains(&0) || threads_per_threadgroup.contains(&0) {
            return Err(MetalDispatchInvocationValidationErrorV1::ZeroDimension);
        }
        Ok(ValidatedMetalDispatchArgumentsV1 {
            grid_threads,
            threads_per_threadgroup,
            sequence_step_ordinal: self.sequence_step_ordinal,
            chunk_ordinal: self.chunk_ordinal,
        })
    }

    #[cfg(test)]
    pub(super) fn validated_geometry_for_test(
        &self,
    ) -> Result<([usize; 3], [usize; 3]), MetalDispatchInvocationValidationErrorV1> {
        let arguments = self.validated_arguments()?;
        Ok((arguments.grid_threads, arguments.threads_per_threadgroup))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MetalPipelineArtifactMetadataV1 {
    pub artifact_schema_version: u32,
    pub function_name: String,
    pub shader_source_digest: String,
    pub library_build_policy_identity: String,
    pub function_lookup_policy_identity: String,
    pub function_constant_identity: String,
    pub kernel_abi_identity: String,
    pub parameter_buffer_layout_identity: String,
    pub semantic_pipeline_identity: String,
    pub thread_execution_width: usize,
    pub max_total_threads_per_threadgroup: usize,
}

struct M3MicroMetalPipelineArtifactV1 {
    pipeline_state: ComputePipelineState,
    metadata: MetalPipelineArtifactMetadataV1,
}

struct MetalFunctionLookupInvocationV1<'a> {
    library: &'a metal::LibraryRef,
    function_name: &'a str,
}

struct MetalPipelineCreationInvocationV1<'a> {
    function: &'a metal::FunctionRef,
    function_name: &'a str,
    shader_source_digest: &'a str,
    library_build_policy_identity: &'a str,
    function_lookup_policy_identity: &'a str,
    function_constant_identity: &'a str,
    kernel_abi_identity: &'a str,
    parameter_buffer_layout_identity: &'a str,
}

pub(super) struct MetalPipelineBindingInvocationV1<'a> {
    artifact: &'a M3MicroMetalPipelineArtifactV1,
    pub encoder_ordinal: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MetalPipelineArtifactValidationErrorV1 {
    IncompleteArtifact,
    SemanticIdentityMismatch,
    InvalidObservation,
    ObservationMismatch,
}

impl<'a> MetalPipelineBindingInvocationV1<'a> {
    fn new(artifact: &'a M3MicroMetalPipelineArtifactV1, encoder_ordinal: usize) -> Self {
        Self {
            artifact,
            encoder_ordinal,
        }
    }
}

pub(super) fn metal_library_build_policy_identity_v1() -> String {
    stable_hash_string(METAL_LIBRARY_BUILD_POLICY_V1)
}

pub(super) fn metal_function_lookup_policy_identity_v1() -> String {
    stable_hash_string(METAL_FUNCTION_LOOKUP_POLICY_V1)
}

pub(super) fn metal_function_constant_identity_v1() -> String {
    stable_hash_string(METAL_FUNCTION_CONSTANT_IDENTITY_V1)
}

pub(super) fn metal_kernel_abi_identity_v1() -> String {
    stable_hash_string(METAL_KERNEL_ABI_IDENTITY_V1)
}

pub(super) fn metal_parameter_buffer_layout_identity_v1() -> String {
    stable_hash_string(METAL_PARAMETER_BUFFER_LAYOUT_IDENTITY_V1)
}

pub(super) fn metal_pipeline_semantic_identity_from_metadata_v1(
    metadata: &MetalPipelineArtifactMetadataV1,
) -> String {
    stable_hash_string(&format!(
        "m3-micro-metal-pipeline-artifact-v1:{}:{}:{}:{}:{}:{}:{}:{}",
        metadata.artifact_schema_version,
        metadata.shader_source_digest,
        metadata.function_name,
        metadata.library_build_policy_identity,
        metadata.function_lookup_policy_identity,
        metadata.function_constant_identity,
        metadata.kernel_abi_identity,
        metadata.parameter_buffer_layout_identity,
    ))
}

fn validate_pipeline_artifact_metadata_v1(
    metadata: &MetalPipelineArtifactMetadataV1,
) -> Result<(), MetalPipelineArtifactValidationErrorV1> {
    if metadata.artifact_schema_version == 0
        || metadata.function_name.is_empty()
        || metadata.shader_source_digest.is_empty()
        || metadata.library_build_policy_identity.is_empty()
        || metadata.function_lookup_policy_identity.is_empty()
        || metadata.function_constant_identity.is_empty()
        || metadata.kernel_abi_identity.is_empty()
        || metadata.parameter_buffer_layout_identity.is_empty()
        || metadata.semantic_pipeline_identity.is_empty()
    {
        return Err(MetalPipelineArtifactValidationErrorV1::IncompleteArtifact);
    }
    if metadata.semantic_pipeline_identity
        != metal_pipeline_semantic_identity_from_metadata_v1(metadata)
    {
        return Err(MetalPipelineArtifactValidationErrorV1::SemanticIdentityMismatch);
    }
    if metadata.thread_execution_width == 0 || metadata.max_total_threads_per_threadgroup == 0 {
        return Err(MetalPipelineArtifactValidationErrorV1::InvalidObservation);
    }
    Ok(())
}

fn validate_pipeline_artifact_v1(
    artifact: &M3MicroMetalPipelineArtifactV1,
) -> Result<(), MetalPipelineArtifactValidationErrorV1> {
    validate_pipeline_artifact_metadata_v1(&artifact.metadata)?;
    let thread_execution_width = usize::try_from(artifact.pipeline_state.thread_execution_width())
        .map_err(|_| MetalPipelineArtifactValidationErrorV1::InvalidObservation)?;
    let max_total_threads_per_threadgroup =
        usize::try_from(artifact.pipeline_state.max_total_threads_per_threadgroup())
            .map_err(|_| MetalPipelineArtifactValidationErrorV1::InvalidObservation)?;
    if artifact.metadata.thread_execution_width != thread_execution_width
        || artifact.metadata.max_total_threads_per_threadgroup != max_total_threads_per_threadgroup
    {
        return Err(MetalPipelineArtifactValidationErrorV1::ObservationMismatch);
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn validate_pipeline_artifact_metadata_for_test_v1(
    metadata: &MetalPipelineArtifactMetadataV1,
) -> Result<(), String> {
    validate_pipeline_artifact_metadata_v1(metadata).map_err(|error| format!("{error:?}"))
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalExecutionModeV1 {
    Streaming,
    FullSequence,
    Chunked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalBufferRoleV2 {
    Input,
    Parameters,
    InitialState,
    Metadata,
    ScalarConfiguration,
    Output,
    FinalState,
    ExecutionWitness,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalBufferAccessV2 {
    ReadOnly,
    WriteOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MetalBufferBindingValidationErrorV1 {
    BindingIndexConversion,
    ByteOffsetConversion,
    ResourceLengthConversion,
    ZeroRequiredSpan,
    InvalidOffset,
    InsufficientSpan,
}

pub(super) struct MetalBufferBindingInvocationV1<'a> {
    pub buffer: &'a metal::BufferRef,
    pub binding_index: usize,
    pub byte_offset: usize,
    pub semantic_role: MetalBufferRoleV2,
    pub semantic_access: MetalBufferAccessV2,
    pub required_span_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ValidatedMetalBufferBindingArgumentsV1 {
    api_binding_index: u64,
    api_byte_offset: u64,
    binding_index: usize,
    byte_offset: usize,
    semantic_role: MetalBufferRoleV2,
    semantic_access: MetalBufferAccessV2,
    actual_resource_length_bytes: usize,
    required_span_bytes: usize,
    available_span_bytes: usize,
}

impl<'a> MetalBufferBindingInvocationV1<'a> {
    pub(super) fn new(
        buffer: &'a metal::BufferRef,
        binding_index: usize,
        byte_offset: usize,
        semantic_role: MetalBufferRoleV2,
        semantic_access: MetalBufferAccessV2,
        required_span_bytes: usize,
    ) -> Self {
        Self {
            buffer,
            binding_index,
            byte_offset,
            semantic_role,
            semantic_access,
            required_span_bytes,
        }
    }

    fn validated_arguments(
        &self,
    ) -> Result<ValidatedMetalBufferBindingArgumentsV1, MetalBufferBindingValidationErrorV1> {
        let api_binding_index = u64::try_from(self.binding_index)
            .map_err(|_| MetalBufferBindingValidationErrorV1::BindingIndexConversion)?;
        let api_byte_offset = u64::try_from(self.byte_offset)
            .map_err(|_| MetalBufferBindingValidationErrorV1::ByteOffsetConversion)?;
        let actual_resource_length_bytes = usize::try_from(self.buffer.length())
            .map_err(|_| MetalBufferBindingValidationErrorV1::ResourceLengthConversion)?;
        if self.required_span_bytes == 0 {
            return Err(MetalBufferBindingValidationErrorV1::ZeroRequiredSpan);
        }
        let available_span_bytes = actual_resource_length_bytes
            .checked_sub(self.byte_offset)
            .ok_or(MetalBufferBindingValidationErrorV1::InvalidOffset)?;
        if self.required_span_bytes > available_span_bytes {
            return Err(MetalBufferBindingValidationErrorV1::InsufficientSpan);
        }
        Ok(ValidatedMetalBufferBindingArgumentsV1 {
            api_binding_index,
            api_byte_offset,
            binding_index: self.binding_index,
            byte_offset: self.byte_offset,
            semantic_role: self.semantic_role,
            semantic_access: self.semantic_access,
            actual_resource_length_bytes,
            required_span_bytes: self.required_span_bytes,
            available_span_bytes,
        })
    }

    #[cfg(test)]
    pub(super) fn validated_range_for_test(
        &self,
    ) -> Result<(usize, usize, usize), MetalBufferBindingValidationErrorV1> {
        let arguments = self.validated_arguments()?;
        Ok((
            arguments.actual_resource_length_bytes,
            arguments.required_span_bytes,
            arguments.available_span_bytes,
        ))
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalTraceCallSiteV1 {
    CommandBufferCreated,
    EncoderCreated,
    PipelineBound,
    BufferBound,
    DispatchIssued,
    EncoderEnded,
    CommandCommitted,
    CommandCompleted,
    OutputReadback,
    StateReadback,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalDispatchCallSiteV1 {
    M3MicroForward,
    MutationOnly,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalSemanticTraceEventV1 {
    pub event_ordinal: usize,
    pub command_ordinal: usize,
    pub encoder_ordinal: Option<usize>,
    pub call_site: MetalTraceCallSiteV1,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalActualBufferBindingTraceV1 {
    pub binding_ordinal: usize,
    pub binding_index: usize,
    pub semantic_role: MetalBufferRoleV2,
    pub semantic_access: MetalBufferAccessV2,
    pub byte_offset: usize,
    pub actual_resource_length_bytes: usize,
    pub required_span_bytes: usize,
    pub available_span_bytes: usize,
    pub binding_performed: bool,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalActualPipelineBindingTraceV1 {
    pub pipeline_binding_ordinal: usize,
    pub encoder_ordinal: usize,
    pub artifact_schema_version: u32,
    pub function_name: String,
    pub shader_source_digest: String,
    pub library_build_policy_identity: String,
    pub function_lookup_policy_identity: String,
    pub function_constant_identity: String,
    pub kernel_abi_identity: String,
    pub parameter_buffer_layout_identity: String,
    pub semantic_pipeline_identity: String,
    pub thread_execution_width: usize,
    pub max_total_threads_per_threadgroup: usize,
    pub binding_performed: bool,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalSemanticDispatchTraceV1 {
    pub dispatch_ordinal: usize,
    pub call_site: MetalDispatchCallSiteV1,
    pub grid_threads: [usize; 3],
    pub threads_per_threadgroup: [usize; 3],
    pub sequence_step_ordinal: usize,
    pub chunk_ordinal: usize,
    pub chunk_start: usize,
    pub chunk_length: usize,
    pub dispatch_performed: bool,
}

#[cfg(test)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalDispatchArgumentRunEvidenceV1 {
    pub actual_argument_capture_count: usize,
    pub trace_record_count: usize,
    pub argument_trace_mismatch_count: usize,
    pub invalid_dimension_count: usize,
    pub conversion_failure_count: usize,
    pub geometry_digest: String,
}

#[cfg(test)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalBufferBindingArgumentRunEvidenceV1 {
    pub actual_buffer_capture_count: usize,
    pub trace_record_count: usize,
    pub argument_trace_mismatch_count: usize,
    pub invalid_offset_count: usize,
    pub insufficient_span_count: usize,
    pub duplicate_binding_index_count: usize,
    pub missing_required_role_count: usize,
    pub unexpected_role_count: usize,
    pub binding_sequence_digest: String,
}

#[cfg(test)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalPipelineBindingArgumentRunEvidenceV1 {
    pub pipeline_artifact_capture_count: usize,
    pub trace_record_count: usize,
    pub artifact_trace_mismatch_count: usize,
    pub incomplete_artifact_count: usize,
    pub invalid_pipeline_observation_count: usize,
    pub semantic_pipeline_digest: String,
    pub pipeline_observation_digest: String,
    pub pipeline_binding_sequence_digest: String,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalSemanticEncoderTraceV1 {
    pub encoder_ordinal: usize,
    pub pipeline_bindings: Vec<MetalActualPipelineBindingTraceV1>,
    pub buffer_bindings: Vec<MetalActualBufferBindingTraceV1>,
    pub dispatches: Vec<MetalSemanticDispatchTraceV1>,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalSemanticCommandTraceV1 {
    pub command_ordinal: usize,
    pub encoder_count: usize,
    pub encoders: Vec<MetalSemanticEncoderTraceV1>,
    pub committed: bool,
    pub terminal_status: MetalCommandTerminalStatusV1,
    pub error_present: bool,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalSemanticSegmentTraceV1 {
    pub chunk_ordinal: usize,
    pub chunk_start: usize,
    pub chunk_length: usize,
    pub command_ordinal: usize,
    pub chunk_boundary_state_digest: String,
}

#[cfg(test)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalPerRunWitnessDeltaV1 {
    pub command_buffers_created: usize,
    pub command_buffers_committed: usize,
    pub command_buffers_completed: usize,
    pub command_buffer_failures: usize,
    pub compute_encoders_created: usize,
    pub dispatches_attempted: usize,
    pub dispatches_performed: usize,
    pub output_readbacks: usize,
    pub state_readbacks: usize,
    pub output_poison_remaining: usize,
    pub state_poison_remaining: usize,
    pub cpu_fallback_attempts: usize,
    pub cpu_fallback_executions: usize,
}

#[cfg(test)]
impl MetalPerRunWitnessDeltaV1 {
    pub(super) fn between(
        before: &MetalExecutionWitnessV1,
        after: &MetalExecutionWitnessV1,
    ) -> Result<Self, String> {
        let delta = |name: &str, before: usize, after: usize| {
            after
                .checked_sub(before)
                .ok_or_else(|| format!("Metal witness counter regressed: {name}"))
        };
        Ok(Self {
            command_buffers_created: delta(
                "command_buffers_created",
                before.command_buffer_count,
                after.command_buffer_count,
            )?,
            command_buffers_committed: delta(
                "command_buffers_committed",
                before.command_buffers_committed,
                after.command_buffers_committed,
            )?,
            command_buffers_completed: delta(
                "command_buffers_completed",
                before.command_buffers_completed,
                after.command_buffers_completed,
            )?,
            command_buffer_failures: delta(
                "command_buffer_failures",
                before.command_buffer_failures,
                after.command_buffer_failures,
            )?,
            compute_encoders_created: delta(
                "compute_encoders_created",
                before.compute_encoder_count,
                after.compute_encoder_count,
            )?,
            dispatches_attempted: delta(
                "dispatches_attempted",
                before.dispatches_attempted,
                after.dispatches_attempted,
            )?,
            dispatches_performed: delta(
                "dispatches_performed",
                before.dispatch_count,
                after.dispatch_count,
            )?,
            output_readbacks: delta(
                "output_readbacks",
                before.output_readback_count,
                after.output_readback_count,
            )?,
            state_readbacks: delta(
                "state_readbacks",
                before.state_readback_count,
                after.state_readback_count,
            )?,
            output_poison_remaining: delta(
                "output_poison_remaining",
                before.output_poison_remaining,
                after.output_poison_remaining,
            )?,
            state_poison_remaining: delta(
                "state_poison_remaining",
                before.state_poison_remaining,
                after.state_poison_remaining,
            )?,
            cpu_fallback_attempts: delta(
                "cpu_fallback_attempts",
                before.cpu_fallback_attempts,
                after.cpu_fallback_attempts,
            )?,
            cpu_fallback_executions: delta(
                "cpu_fallback_executions",
                before.cpu_fallback_count,
                after.cpu_fallback_count,
            )?,
        })
    }

    pub(super) fn successful(&self) -> bool {
        self.command_buffers_created > 0
            && self.command_buffers_created == self.command_buffers_committed
            && self.command_buffers_committed == self.command_buffers_completed
            && self.command_buffer_failures == 0
            && self.compute_encoders_created > 0
            && self.dispatches_attempted > 0
            && self.dispatches_performed == self.dispatches_attempted
            && self.output_readbacks > 0
            && self.state_readbacks > 0
            && self.output_poison_remaining == 0
            && self.state_poison_remaining == 0
            && self.cpu_fallback_attempts == 0
            && self.cpu_fallback_executions == 0
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalSemanticRunTraceV1 {
    pub trace_schema_version: u32,
    pub execution_mode: MetalExecutionModeV1,
    pub agent_role: String,
    pub sequence_length: usize,
    pub chunk_policy_identity: String,
    pub commands: Vec<MetalSemanticCommandTraceV1>,
    pub segments: Vec<MetalSemanticSegmentTraceV1>,
    pub call_site_events: Vec<MetalSemanticTraceEventV1>,
    pub dispatch_argument_provenance: MetalDispatchArgumentRunEvidenceV1,
    pub buffer_binding_argument_provenance: MetalBufferBindingArgumentRunEvidenceV1,
    pub pipeline_binding_argument_provenance: MetalPipelineBindingArgumentRunEvidenceV1,
    pub per_run_witness_delta: MetalPerRunWitnessDeltaV1,
    pub output_digest: String,
    pub final_state_digest: String,
    pub semantic_trace_digest: String,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalTopologyMismatchV1 {
    pub scenario_identity: String,
    pub run_pair: String,
    pub command_ordinal: Option<usize>,
    pub encoder_ordinal: Option<usize>,
    pub dispatch_ordinal: Option<usize>,
    pub binding_index: Option<usize>,
    pub field: String,
    pub left: String,
    pub right: String,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalTopologyDeterminismStatusV1 {
    #[serde(rename = "PASSED_SEMANTIC_COMMAND_TOPOLOGY")]
    PassedSemanticTopology,
    #[serde(rename = "FAILED_SEMANTIC_COMMAND_TOPOLOGY")]
    FailedSemanticTopology,
    #[serde(rename = "INCOMPLETE")]
    Incomplete,
    #[serde(rename = "NOT_RUN")]
    NotRun,
}

#[cfg(test)]
impl MetalTopologyDeterminismStatusV1 {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::PassedSemanticTopology => "PASSED_SEMANTIC_COMMAND_TOPOLOGY",
            Self::FailedSemanticTopology => "FAILED_SEMANTIC_COMMAND_TOPOLOGY",
            Self::Incomplete => "INCOMPLETE",
            Self::NotRun => "NOT_RUN",
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingSemanticSegmentV1 {
    chunk_ordinal: usize,
    chunk_start: usize,
    chunk_length: usize,
}

#[cfg(test)]
struct MetalSemanticTraceCollectorV4 {
    trace: MetalSemanticRunTraceV1,
    witness_before: MetalExecutionWitnessV1,
    pending_segment: Option<PendingSemanticSegmentV1>,
}

#[cfg(test)]
impl MetalSemanticTraceCollectorV4 {
    fn new(
        execution_mode: MetalExecutionModeV1,
        agent_role: String,
        sequence_length: usize,
        chunk_policy_identity: String,
        witness_before: MetalExecutionWitnessV1,
    ) -> Self {
        Self {
            trace: MetalSemanticRunTraceV1 {
                trace_schema_version: METAL_SEMANTIC_TRACE_SCHEMA_VERSION_V4,
                execution_mode,
                agent_role,
                sequence_length,
                chunk_policy_identity,
                commands: Vec::new(),
                segments: Vec::new(),
                call_site_events: Vec::new(),
                dispatch_argument_provenance: MetalDispatchArgumentRunEvidenceV1::default(),
                buffer_binding_argument_provenance:
                    MetalBufferBindingArgumentRunEvidenceV1::default(),
                pipeline_binding_argument_provenance:
                    MetalPipelineBindingArgumentRunEvidenceV1::default(),
                per_run_witness_delta: MetalPerRunWitnessDeltaV1::default(),
                output_digest: String::new(),
                final_state_digest: String::new(),
                semantic_trace_digest: String::new(),
            },
            witness_before,
            pending_segment: None,
        }
    }

    fn set_segment(&mut self, chunk_ordinal: usize, chunk_start: usize, chunk_length: usize) {
        self.pending_segment = Some(PendingSemanticSegmentV1 {
            chunk_ordinal,
            chunk_start,
            chunk_length,
        });
    }

    fn record_event(
        &mut self,
        command_ordinal: usize,
        encoder_ordinal: Option<usize>,
        call_site: MetalTraceCallSiteV1,
    ) {
        self.trace.call_site_events.push(MetalSemanticTraceEventV1 {
            event_ordinal: self.trace.call_site_events.len(),
            command_ordinal,
            encoder_ordinal,
            call_site,
        });
    }

    fn command_created(&mut self, sequence_length: usize) -> Result<usize, String> {
        let segment = self
            .pending_segment
            .ok_or_else(|| "semantic trace segment was not registered".to_string())?;
        if segment.chunk_length != sequence_length
            || segment.chunk_start + segment.chunk_length > self.trace.sequence_length
        {
            return Err("semantic trace segment bounds mismatch".to_string());
        }
        let command_ordinal = self.trace.commands.len();
        self.trace.commands.push(MetalSemanticCommandTraceV1 {
            command_ordinal,
            encoder_count: 0,
            encoders: Vec::new(),
            committed: false,
            terminal_status: MetalCommandTerminalStatusV1::Other,
            error_present: false,
        });
        self.record_event(
            command_ordinal,
            None,
            MetalTraceCallSiteV1::CommandBufferCreated,
        );
        Ok(command_ordinal)
    }

    fn encoder_created(&mut self, command_ordinal: usize) -> usize {
        let command = &mut self.trace.commands[command_ordinal];
        let encoder_ordinal = command.encoders.len();
        command.encoders.push(MetalSemanticEncoderTraceV1 {
            encoder_ordinal,
            pipeline_bindings: Vec::new(),
            buffer_bindings: Vec::new(),
            dispatches: Vec::new(),
        });
        command.encoder_count = command.encoders.len();
        self.record_event(
            command_ordinal,
            Some(encoder_ordinal),
            MetalTraceCallSiteV1::EncoderCreated,
        );
        encoder_ordinal
    }

    fn record_actual_pipeline_binding_v2(
        &mut self,
        command_ordinal: usize,
        invocation: &MetalPipelineBindingInvocationV1<'_>,
    ) -> Result<usize, String> {
        let encoder_ordinal = invocation.encoder_ordinal;
        let encoder = self
            .trace
            .commands
            .get_mut(command_ordinal)
            .and_then(|command| command.encoders.get_mut(encoder_ordinal))
            .ok_or_else(|| "Metal pipeline binding encoder is missing".to_string())?;
        if !encoder.pipeline_bindings.is_empty() {
            self.trace
                .pipeline_binding_argument_provenance
                .incomplete_artifact_count += 1;
            return Err("duplicate normal Metal pipeline binding".to_string());
        }
        let pipeline_binding_ordinal = encoder.pipeline_bindings.len();
        self.trace
            .pipeline_binding_argument_provenance
            .pipeline_artifact_capture_count += 1;
        let trace =
            metal_pipeline_binding_trace_from_invocation_v1(pipeline_binding_ordinal, invocation)?;
        self.trace
            .pipeline_binding_argument_provenance
            .semantic_pipeline_digest = next_metal_pipeline_semantic_digest_v1(
            &self
                .trace
                .pipeline_binding_argument_provenance
                .semantic_pipeline_digest,
            &trace,
        );
        self.trace
            .pipeline_binding_argument_provenance
            .pipeline_observation_digest = next_metal_pipeline_observation_digest_v1(
            &self
                .trace
                .pipeline_binding_argument_provenance
                .pipeline_observation_digest,
            &trace,
        );
        self.trace
            .pipeline_binding_argument_provenance
            .pipeline_binding_sequence_digest = next_metal_pipeline_binding_sequence_digest_v1(
            &self
                .trace
                .pipeline_binding_argument_provenance
                .pipeline_binding_sequence_digest,
            command_ordinal,
            &trace,
        );
        encoder.pipeline_bindings.push(trace);
        self.trace
            .pipeline_binding_argument_provenance
            .trace_record_count += 1;
        if validate_pipeline_trace_matches_invocation_v1(
            invocation,
            &encoder.pipeline_bindings[pipeline_binding_ordinal],
        )
        .is_err()
        {
            self.trace
                .pipeline_binding_argument_provenance
                .artifact_trace_mismatch_count += 1;
            return Err("Metal pipeline artifact/trace mismatch".to_string());
        }
        self.record_event(
            command_ordinal,
            Some(encoder_ordinal),
            MetalTraceCallSiteV1::PipelineBound,
        );
        Ok(pipeline_binding_ordinal)
    }

    fn pipeline_binding_validation_failed(
        &mut self,
        error: MetalPipelineArtifactValidationErrorV1,
    ) {
        match error {
            MetalPipelineArtifactValidationErrorV1::InvalidObservation
            | MetalPipelineArtifactValidationErrorV1::ObservationMismatch => {
                self.trace
                    .pipeline_binding_argument_provenance
                    .invalid_pipeline_observation_count += 1;
            }
            MetalPipelineArtifactValidationErrorV1::IncompleteArtifact
            | MetalPipelineArtifactValidationErrorV1::SemanticIdentityMismatch => {
                self.trace
                    .pipeline_binding_argument_provenance
                    .incomplete_artifact_count += 1;
            }
        }
    }

    fn pipeline_binding_performed(
        &mut self,
        command_ordinal: usize,
        encoder_ordinal: usize,
        pipeline_binding_ordinal: usize,
    ) {
        self.trace.commands[command_ordinal].encoders[encoder_ordinal].pipeline_bindings
            [pipeline_binding_ordinal]
            .binding_performed = true;
    }

    fn record_actual_buffer_binding_v1(
        &mut self,
        command_ordinal: usize,
        encoder_ordinal: usize,
        invocation: &MetalBufferBindingInvocationV1<'_>,
        arguments: &ValidatedMetalBufferBindingArgumentsV1,
    ) -> Result<usize, String> {
        let expected = normal_metal_buffer_binding_contract_v2(arguments.binding_index)
            .ok_or_else(|| {
                self.trace
                    .buffer_binding_argument_provenance
                    .unexpected_role_count += 1;
                "unexpected normal Metal buffer binding index".to_string()
            })?;
        if expected != (arguments.semantic_role, arguments.semantic_access) {
            self.trace
                .buffer_binding_argument_provenance
                .unexpected_role_count += 1;
            return Err("unexpected normal Metal buffer role/access contract".to_string());
        }
        let encoder = &mut self.trace.commands[command_ordinal].encoders[encoder_ordinal];
        if encoder
            .buffer_bindings
            .iter()
            .any(|binding| binding.binding_index == arguments.binding_index)
        {
            self.trace
                .buffer_binding_argument_provenance
                .duplicate_binding_index_count += 1;
            return Err("duplicate normal Metal buffer binding index".to_string());
        }
        let binding_ordinal = encoder.buffer_bindings.len();
        self.trace
            .buffer_binding_argument_provenance
            .actual_buffer_capture_count += 1;
        self.trace
            .buffer_binding_argument_provenance
            .binding_sequence_digest = next_metal_buffer_binding_sequence_digest_v1(
            &self
                .trace
                .buffer_binding_argument_provenance
                .binding_sequence_digest,
            command_ordinal,
            encoder_ordinal,
            binding_ordinal,
            arguments,
        );
        encoder
            .buffer_bindings
            .push(MetalActualBufferBindingTraceV1 {
                binding_ordinal,
                binding_index: arguments.binding_index,
                semantic_role: arguments.semantic_role,
                semantic_access: arguments.semantic_access,
                byte_offset: arguments.byte_offset,
                actual_resource_length_bytes: arguments.actual_resource_length_bytes,
                required_span_bytes: arguments.required_span_bytes,
                available_span_bytes: arguments.available_span_bytes,
                binding_performed: false,
            });
        self.trace
            .buffer_binding_argument_provenance
            .trace_record_count += 1;
        if validate_buffer_binding_trace_matches_invocation_v1(
            invocation,
            &encoder.buffer_bindings[binding_ordinal],
        )
        .is_err()
        {
            self.trace
                .buffer_binding_argument_provenance
                .argument_trace_mismatch_count += 1;
            return Err("Metal buffer binding argument/trace mismatch".to_string());
        }
        self.record_event(
            command_ordinal,
            Some(encoder_ordinal),
            MetalTraceCallSiteV1::BufferBound,
        );
        Ok(binding_ordinal)
    }

    fn buffer_binding_validation_failed(&mut self, error: MetalBufferBindingValidationErrorV1) {
        match error {
            MetalBufferBindingValidationErrorV1::InvalidOffset => {
                self.trace
                    .buffer_binding_argument_provenance
                    .invalid_offset_count += 1;
            }
            MetalBufferBindingValidationErrorV1::InsufficientSpan
            | MetalBufferBindingValidationErrorV1::ZeroRequiredSpan => {
                self.trace
                    .buffer_binding_argument_provenance
                    .insufficient_span_count += 1;
            }
            MetalBufferBindingValidationErrorV1::BindingIndexConversion
            | MetalBufferBindingValidationErrorV1::ByteOffsetConversion
            | MetalBufferBindingValidationErrorV1::ResourceLengthConversion => {}
        }
    }

    fn buffer_binding_performed(
        &mut self,
        command_ordinal: usize,
        encoder_ordinal: usize,
        binding_ordinal: usize,
    ) {
        self.trace.commands[command_ordinal].encoders[encoder_ordinal].buffer_bindings
            [binding_ordinal]
            .binding_performed = true;
    }

    fn record_actual_dispatch_v2(
        &mut self,
        command_ordinal: usize,
        encoder_ordinal: usize,
        invocation: &MetalDispatchInvocationV1,
    ) -> Result<usize, String> {
        let segment = self
            .pending_segment
            .ok_or_else(|| "semantic trace dispatch segment is missing".to_string())?;
        let arguments = match invocation.validated_arguments() {
            Ok(arguments) => arguments,
            Err(MetalDispatchInvocationValidationErrorV1::ZeroDimension) => {
                self.trace
                    .dispatch_argument_provenance
                    .invalid_dimension_count += 1;
                return Err("Metal dispatch invocation has a zero dimension".to_string());
            }
            Err(MetalDispatchInvocationValidationErrorV1::ConversionFailure) => {
                self.trace
                    .dispatch_argument_provenance
                    .conversion_failure_count += 1;
                return Err("Metal dispatch invocation dimension conversion failed".to_string());
            }
        };
        if arguments.sequence_step_ordinal != segment.chunk_start
            || arguments.chunk_ordinal != segment.chunk_ordinal
        {
            return Err("Metal dispatch invocation segment identity mismatch".to_string());
        }
        let encoder = &mut self.trace.commands[command_ordinal].encoders[encoder_ordinal];
        let dispatch_ordinal = encoder.dispatches.len();
        let actual_dispatch_ordinal = self
            .trace
            .dispatch_argument_provenance
            .actual_argument_capture_count;
        self.trace
            .dispatch_argument_provenance
            .actual_argument_capture_count += 1;
        self.trace.dispatch_argument_provenance.geometry_digest =
            next_metal_dispatch_geometry_digest_v1(
                &self.trace.dispatch_argument_provenance.geometry_digest,
                actual_dispatch_ordinal,
                &arguments,
            );
        encoder
            .dispatches
            .push(metal_semantic_dispatch_trace_from_invocation_v1(
                dispatch_ordinal,
                segment.chunk_start,
                segment.chunk_length,
                invocation,
            )?);
        self.trace.dispatch_argument_provenance.trace_record_count += 1;
        let recorded = &encoder.dispatches[dispatch_ordinal];
        if validate_dispatch_trace_matches_invocation_v1(invocation, recorded).is_err() {
            self.trace
                .dispatch_argument_provenance
                .argument_trace_mismatch_count += 1;
            return Err("Metal dispatch argument/trace mismatch".to_string());
        }
        self.record_event(
            command_ordinal,
            Some(encoder_ordinal),
            MetalTraceCallSiteV1::DispatchIssued,
        );
        Ok(dispatch_ordinal)
    }

    fn dispatch_performed(
        &mut self,
        command_ordinal: usize,
        encoder_ordinal: usize,
        dispatch_ordinal: usize,
    ) {
        self.trace.commands[command_ordinal].encoders[encoder_ordinal].dispatches
            [dispatch_ordinal]
            .dispatch_performed = true;
    }

    fn encoder_ended(&mut self, command_ordinal: usize, encoder_ordinal: usize) {
        self.record_event(
            command_ordinal,
            Some(encoder_ordinal),
            MetalTraceCallSiteV1::EncoderEnded,
        );
    }

    fn command_committed(&mut self, command_ordinal: usize) {
        self.trace.commands[command_ordinal].committed = true;
        self.record_event(
            command_ordinal,
            None,
            MetalTraceCallSiteV1::CommandCommitted,
        );
    }

    fn command_completed(
        &mut self,
        command_ordinal: usize,
        observation: &MetalCommandCompletionObservationV1,
    ) {
        let command = &mut self.trace.commands[command_ordinal];
        command.terminal_status = observation.terminal_status;
        command.error_present = observation.error_present;
        self.record_event(
            command_ordinal,
            None,
            MetalTraceCallSiteV1::CommandCompleted,
        );
    }

    fn output_readback(&mut self, command_ordinal: usize) {
        self.record_event(command_ordinal, None, MetalTraceCallSiteV1::OutputReadback);
    }

    fn state_readback(&mut self, command_ordinal: usize) {
        self.record_event(command_ordinal, None, MetalTraceCallSiteV1::StateReadback);
    }

    fn segment_completed(&mut self, command_ordinal: usize, state_digest: String) {
        let segment = self
            .pending_segment
            .take()
            .expect("segment checked at command creation");
        self.trace.segments.push(MetalSemanticSegmentTraceV1 {
            chunk_ordinal: segment.chunk_ordinal,
            chunk_start: segment.chunk_start,
            chunk_length: segment.chunk_length,
            command_ordinal,
            chunk_boundary_state_digest: state_digest,
        });
    }

    fn finalize_buffer_binding_provenance(&mut self) {
        let required_contracts = (0..)
            .map_while(normal_metal_buffer_binding_contract_v2)
            .collect::<Vec<_>>();
        for encoder in self
            .trace
            .commands
            .iter()
            .flat_map(|command| &command.encoders)
        {
            for (binding_index, (role, access)) in required_contracts.iter().copied().enumerate() {
                if !encoder.buffer_bindings.iter().any(|binding| {
                    binding.binding_index == binding_index
                        && binding.semantic_role == role
                        && binding.semantic_access == access
                }) {
                    self.trace
                        .buffer_binding_argument_provenance
                        .missing_required_role_count += 1;
                }
            }
        }
    }

    fn finalize_pipeline_binding_provenance(&mut self) {
        for encoder in self
            .trace
            .commands
            .iter()
            .flat_map(|command| &command.encoders)
        {
            if encoder.pipeline_bindings.len() != 1
                || !encoder.pipeline_bindings[0].binding_performed
            {
                self.trace
                    .pipeline_binding_argument_provenance
                    .incomplete_artifact_count += 1;
            }
        }
    }
}

#[cfg(test)]
fn next_metal_dispatch_geometry_digest_v1(
    previous: &str,
    dispatch_ordinal: usize,
    arguments: &ValidatedMetalDispatchArgumentsV1,
) -> String {
    stable_hash_string(&format!(
        "m3-micro-metal-actual-dispatch-v1:{previous}:{dispatch_ordinal}:{:?}:{:?}:{}:{}",
        arguments.grid_threads,
        arguments.threads_per_threadgroup,
        arguments.sequence_step_ordinal,
        arguments.chunk_ordinal,
    ))
}

#[cfg(test)]
pub(super) fn validate_dispatch_trace_matches_invocation_v1(
    invocation: &MetalDispatchInvocationV1,
    trace: &MetalSemanticDispatchTraceV1,
) -> Result<(), String> {
    let arguments = invocation
        .validated_arguments()
        .map_err(|error| format!("invalid Metal dispatch invocation: {error:?}"))?;
    if trace.call_site != MetalDispatchCallSiteV1::M3MicroForward
        || trace.grid_threads != arguments.grid_threads
        || trace.threads_per_threadgroup != arguments.threads_per_threadgroup
        || trace.sequence_step_ordinal != arguments.sequence_step_ordinal
        || trace.chunk_ordinal != arguments.chunk_ordinal
    {
        return Err("Metal dispatch invocation does not match its trace record".to_string());
    }
    Ok(())
}

#[cfg(test)]
fn normal_metal_buffer_binding_contract_v2(
    binding_index: usize,
) -> Option<(MetalBufferRoleV2, MetalBufferAccessV2)> {
    match binding_index {
        0 => Some((MetalBufferRoleV2::Input, MetalBufferAccessV2::ReadOnly)),
        1 => Some((MetalBufferRoleV2::Parameters, MetalBufferAccessV2::ReadOnly)),
        2 => Some((
            MetalBufferRoleV2::InitialState,
            MetalBufferAccessV2::ReadOnly,
        )),
        3 => Some((MetalBufferRoleV2::Output, MetalBufferAccessV2::WriteOnly)),
        4 => Some((
            MetalBufferRoleV2::FinalState,
            MetalBufferAccessV2::WriteOnly,
        )),
        5 => Some((MetalBufferRoleV2::Metadata, MetalBufferAccessV2::ReadOnly)),
        6 => Some((
            MetalBufferRoleV2::ScalarConfiguration,
            MetalBufferAccessV2::ReadOnly,
        )),
        7 => Some((
            MetalBufferRoleV2::ExecutionWitness,
            MetalBufferAccessV2::WriteOnly,
        )),
        _ => None,
    }
}

#[cfg(test)]
fn next_metal_buffer_binding_sequence_digest_v1(
    previous: &str,
    command_ordinal: usize,
    encoder_ordinal: usize,
    binding_ordinal: usize,
    arguments: &ValidatedMetalBufferBindingArgumentsV1,
) -> String {
    stable_hash_string(&format!(
        "m3-micro-metal-actual-buffer-binding-v1:{previous}:{command_ordinal}:{encoder_ordinal}:{binding_ordinal}:{}:{:?}:{:?}:{}:{}:{}:{}",
        arguments.binding_index,
        arguments.semantic_role,
        arguments.semantic_access,
        arguments.byte_offset,
        arguments.actual_resource_length_bytes,
        arguments.required_span_bytes,
        arguments.available_span_bytes,
    ))
}

#[cfg(test)]
pub(super) fn validate_buffer_binding_trace_matches_invocation_v1(
    invocation: &MetalBufferBindingInvocationV1<'_>,
    trace: &MetalActualBufferBindingTraceV1,
) -> Result<(), String> {
    let arguments = invocation
        .validated_arguments()
        .map_err(|error| format!("invalid Metal buffer binding invocation: {error:?}"))?;
    if trace.binding_index != arguments.binding_index
        || trace.semantic_role != arguments.semantic_role
        || trace.semantic_access != arguments.semantic_access
        || trace.byte_offset != arguments.byte_offset
        || trace.actual_resource_length_bytes != arguments.actual_resource_length_bytes
        || trace.required_span_bytes != arguments.required_span_bytes
        || trace.available_span_bytes != arguments.available_span_bytes
    {
        return Err("Metal buffer binding invocation does not match its trace record".to_string());
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn metal_buffer_binding_trace_from_invocation_v1(
    binding_ordinal: usize,
    invocation: &MetalBufferBindingInvocationV1<'_>,
) -> Result<MetalActualBufferBindingTraceV1, String> {
    let arguments = invocation
        .validated_arguments()
        .map_err(|error| format!("invalid Metal buffer binding invocation: {error:?}"))?;
    Ok(MetalActualBufferBindingTraceV1 {
        binding_ordinal,
        binding_index: arguments.binding_index,
        semantic_role: arguments.semantic_role,
        semantic_access: arguments.semantic_access,
        byte_offset: arguments.byte_offset,
        actual_resource_length_bytes: arguments.actual_resource_length_bytes,
        required_span_bytes: arguments.required_span_bytes,
        available_span_bytes: arguments.available_span_bytes,
        binding_performed: false,
    })
}

#[cfg(test)]
pub(super) fn metal_buffer_binding_sequence_digest_v1(trace: &MetalSemanticRunTraceV1) -> String {
    let mut digest = String::new();
    for (command_ordinal, command) in trace.commands.iter().enumerate() {
        for (encoder_ordinal, encoder) in command.encoders.iter().enumerate() {
            for (binding_ordinal, binding) in encoder.buffer_bindings.iter().enumerate() {
                let arguments = ValidatedMetalBufferBindingArgumentsV1 {
                    api_binding_index: binding.binding_index as u64,
                    api_byte_offset: binding.byte_offset as u64,
                    binding_index: binding.binding_index,
                    byte_offset: binding.byte_offset,
                    semantic_role: binding.semantic_role,
                    semantic_access: binding.semantic_access,
                    actual_resource_length_bytes: binding.actual_resource_length_bytes,
                    required_span_bytes: binding.required_span_bytes,
                    available_span_bytes: binding.available_span_bytes,
                };
                digest = next_metal_buffer_binding_sequence_digest_v1(
                    &digest,
                    command_ordinal,
                    encoder_ordinal,
                    binding_ordinal,
                    &arguments,
                );
            }
        }
    }
    digest
}

#[cfg(test)]
pub(super) fn compare_metal_buffer_binding_argument_sequences_v1(
    left: &MetalSemanticRunTraceV1,
    right: &MetalSemanticRunTraceV1,
) -> usize {
    let bindings = |trace: &MetalSemanticRunTraceV1| {
        trace
            .commands
            .iter()
            .flat_map(|command| &command.encoders)
            .flat_map(|encoder| &encoder.buffer_bindings)
            .map(|binding| {
                (
                    binding.binding_ordinal,
                    binding.binding_index,
                    binding.semantic_role,
                    binding.semantic_access,
                    binding.byte_offset,
                    binding.actual_resource_length_bytes,
                    binding.required_span_bytes,
                    binding.available_span_bytes,
                )
            })
            .collect::<Vec<_>>()
    };
    let left = bindings(left);
    let right = bindings(right);
    left.iter()
        .zip(&right)
        .filter(|(left, right)| left != right)
        .count()
        + left.len().abs_diff(right.len())
}

#[cfg(test)]
fn metal_pipeline_binding_trace_from_metadata_v1(
    pipeline_binding_ordinal: usize,
    encoder_ordinal: usize,
    metadata: &MetalPipelineArtifactMetadataV1,
) -> MetalActualPipelineBindingTraceV1 {
    MetalActualPipelineBindingTraceV1 {
        pipeline_binding_ordinal,
        encoder_ordinal,
        artifact_schema_version: metadata.artifact_schema_version,
        function_name: metadata.function_name.clone(),
        shader_source_digest: metadata.shader_source_digest.clone(),
        library_build_policy_identity: metadata.library_build_policy_identity.clone(),
        function_lookup_policy_identity: metadata.function_lookup_policy_identity.clone(),
        function_constant_identity: metadata.function_constant_identity.clone(),
        kernel_abi_identity: metadata.kernel_abi_identity.clone(),
        parameter_buffer_layout_identity: metadata.parameter_buffer_layout_identity.clone(),
        semantic_pipeline_identity: metadata.semantic_pipeline_identity.clone(),
        thread_execution_width: metadata.thread_execution_width,
        max_total_threads_per_threadgroup: metadata.max_total_threads_per_threadgroup,
        binding_performed: false,
    }
}

#[cfg(test)]
fn metal_pipeline_metadata_from_trace_v1(
    trace: &MetalActualPipelineBindingTraceV1,
) -> MetalPipelineArtifactMetadataV1 {
    MetalPipelineArtifactMetadataV1 {
        artifact_schema_version: trace.artifact_schema_version,
        function_name: trace.function_name.clone(),
        shader_source_digest: trace.shader_source_digest.clone(),
        library_build_policy_identity: trace.library_build_policy_identity.clone(),
        function_lookup_policy_identity: trace.function_lookup_policy_identity.clone(),
        function_constant_identity: trace.function_constant_identity.clone(),
        kernel_abi_identity: trace.kernel_abi_identity.clone(),
        parameter_buffer_layout_identity: trace.parameter_buffer_layout_identity.clone(),
        semantic_pipeline_identity: trace.semantic_pipeline_identity.clone(),
        thread_execution_width: trace.thread_execution_width,
        max_total_threads_per_threadgroup: trace.max_total_threads_per_threadgroup,
    }
}

#[cfg(test)]
fn metal_pipeline_binding_trace_from_invocation_v1(
    pipeline_binding_ordinal: usize,
    invocation: &MetalPipelineBindingInvocationV1<'_>,
) -> Result<MetalActualPipelineBindingTraceV1, String> {
    validate_pipeline_artifact_v1(invocation.artifact)
        .map_err(|error| format!("invalid Metal pipeline artifact: {error:?}"))?;
    Ok(metal_pipeline_binding_trace_from_metadata_v1(
        pipeline_binding_ordinal,
        invocation.encoder_ordinal,
        &invocation.artifact.metadata,
    ))
}

#[cfg(test)]
fn validate_pipeline_trace_matches_metadata_v1(
    metadata: &MetalPipelineArtifactMetadataV1,
    encoder_ordinal: usize,
    trace: &MetalActualPipelineBindingTraceV1,
) -> Result<(), String> {
    validate_pipeline_artifact_metadata_v1(metadata)
        .map_err(|error| format!("invalid Metal pipeline metadata: {error:?}"))?;
    if trace.encoder_ordinal != encoder_ordinal
        || trace.artifact_schema_version != metadata.artifact_schema_version
        || trace.function_name != metadata.function_name
        || trace.shader_source_digest != metadata.shader_source_digest
        || trace.library_build_policy_identity != metadata.library_build_policy_identity
        || trace.function_lookup_policy_identity != metadata.function_lookup_policy_identity
        || trace.function_constant_identity != metadata.function_constant_identity
        || trace.kernel_abi_identity != metadata.kernel_abi_identity
        || trace.parameter_buffer_layout_identity != metadata.parameter_buffer_layout_identity
        || trace.semantic_pipeline_identity != metadata.semantic_pipeline_identity
        || trace.thread_execution_width != metadata.thread_execution_width
        || trace.max_total_threads_per_threadgroup != metadata.max_total_threads_per_threadgroup
    {
        return Err("Metal pipeline artifact does not match its trace record".to_string());
    }
    Ok(())
}

#[cfg(test)]
fn validate_pipeline_trace_matches_invocation_v1(
    invocation: &MetalPipelineBindingInvocationV1<'_>,
    trace: &MetalActualPipelineBindingTraceV1,
) -> Result<(), String> {
    validate_pipeline_artifact_v1(invocation.artifact)
        .map_err(|error| format!("invalid Metal pipeline artifact: {error:?}"))?;
    validate_pipeline_trace_matches_metadata_v1(
        &invocation.artifact.metadata,
        invocation.encoder_ordinal,
        trace,
    )
}

#[cfg(test)]
pub(super) fn metal_pipeline_binding_trace_from_metadata_for_test_v1(
    metadata: &MetalPipelineArtifactMetadataV1,
) -> MetalActualPipelineBindingTraceV1 {
    metal_pipeline_binding_trace_from_metadata_v1(0, 0, metadata)
}

#[cfg(test)]
pub(super) fn validate_pipeline_trace_matches_metadata_for_test_v1(
    metadata: &MetalPipelineArtifactMetadataV1,
    trace: &MetalActualPipelineBindingTraceV1,
) -> Result<(), String> {
    validate_pipeline_trace_matches_metadata_v1(metadata, trace.encoder_ordinal, trace)
}

#[cfg(test)]
fn next_metal_pipeline_semantic_digest_v1(
    previous: &str,
    trace: &MetalActualPipelineBindingTraceV1,
) -> String {
    stable_hash_string(&format!(
        "m3-micro-metal-pipeline-semantic-v1:{previous}:{}:{}:{}:{}:{}:{}:{}:{}",
        trace.artifact_schema_version,
        trace.function_name,
        trace.shader_source_digest,
        trace.library_build_policy_identity,
        trace.function_lookup_policy_identity,
        trace.function_constant_identity,
        trace.kernel_abi_identity,
        trace.parameter_buffer_layout_identity,
    ))
}

#[cfg(test)]
fn next_metal_pipeline_observation_digest_v1(
    previous: &str,
    trace: &MetalActualPipelineBindingTraceV1,
) -> String {
    stable_hash_string(&format!(
        "m3-micro-metal-pipeline-observation-v1:{previous}:{}:{}",
        trace.thread_execution_width, trace.max_total_threads_per_threadgroup,
    ))
}

#[cfg(test)]
fn next_metal_pipeline_binding_sequence_digest_v1(
    previous: &str,
    command_ordinal: usize,
    trace: &MetalActualPipelineBindingTraceV1,
) -> String {
    stable_hash_string(&format!(
        "m3-micro-metal-pipeline-binding-v1:{previous}:{command_ordinal}:{}:{}:{}:{}:{}",
        trace.encoder_ordinal,
        trace.pipeline_binding_ordinal,
        trace.semantic_pipeline_identity,
        trace.thread_execution_width,
        trace.max_total_threads_per_threadgroup,
    ))
}

#[cfg(test)]
pub(super) fn metal_pipeline_binding_sequence_digest_v1(trace: &MetalSemanticRunTraceV1) -> String {
    let mut digest = String::new();
    for (command_ordinal, command) in trace.commands.iter().enumerate() {
        for encoder in &command.encoders {
            for pipeline in &encoder.pipeline_bindings {
                digest = next_metal_pipeline_binding_sequence_digest_v1(
                    &digest,
                    command_ordinal,
                    pipeline,
                );
            }
        }
    }
    digest
}

#[cfg(test)]
pub(super) fn metal_pipeline_semantic_digest_v1(trace: &MetalSemanticRunTraceV1) -> String {
    let mut digest = String::new();
    for pipeline in trace
        .commands
        .iter()
        .flat_map(|command| &command.encoders)
        .flat_map(|encoder| &encoder.pipeline_bindings)
    {
        digest = next_metal_pipeline_semantic_digest_v1(&digest, pipeline);
    }
    digest
}

#[cfg(test)]
pub(super) fn metal_pipeline_observation_digest_v1(trace: &MetalSemanticRunTraceV1) -> String {
    let mut digest = String::new();
    for pipeline in trace
        .commands
        .iter()
        .flat_map(|command| &command.encoders)
        .flat_map(|encoder| &encoder.pipeline_bindings)
    {
        digest = next_metal_pipeline_observation_digest_v1(&digest, pipeline);
    }
    digest
}

#[cfg(test)]
pub(super) fn compare_metal_pipeline_binding_argument_sequences_v1(
    left: &MetalSemanticRunTraceV1,
    right: &MetalSemanticRunTraceV1,
) -> usize {
    let pipelines = |trace: &MetalSemanticRunTraceV1| {
        trace
            .commands
            .iter()
            .flat_map(|command| &command.encoders)
            .flat_map(|encoder| &encoder.pipeline_bindings)
            .cloned()
            .collect::<Vec<_>>()
    };
    let left = pipelines(left);
    let right = pipelines(right);
    left.iter()
        .zip(&right)
        .filter(|(left, right)| left != right)
        .count()
        + left.len().abs_diff(right.len())
}

#[cfg(test)]
pub(super) fn metal_semantic_dispatch_trace_from_invocation_v1(
    dispatch_ordinal: usize,
    chunk_start: usize,
    chunk_length: usize,
    invocation: &MetalDispatchInvocationV1,
) -> Result<MetalSemanticDispatchTraceV1, String> {
    if chunk_length == 0 {
        return Err("Metal dispatch trace chunk is empty".to_string());
    }
    let arguments = invocation
        .validated_arguments()
        .map_err(|error| format!("invalid Metal dispatch invocation: {error:?}"))?;
    Ok(MetalSemanticDispatchTraceV1 {
        dispatch_ordinal,
        call_site: MetalDispatchCallSiteV1::M3MicroForward,
        grid_threads: arguments.grid_threads,
        threads_per_threadgroup: arguments.threads_per_threadgroup,
        sequence_step_ordinal: arguments.sequence_step_ordinal,
        chunk_ordinal: arguments.chunk_ordinal,
        chunk_start,
        chunk_length,
        dispatch_performed: false,
    })
}

#[cfg(test)]
pub(super) fn metal_dispatch_geometry_digest_v1(trace: &MetalSemanticRunTraceV1) -> String {
    let mut digest = String::new();
    for (dispatch_ordinal, dispatch) in trace
        .commands
        .iter()
        .flat_map(|command| &command.encoders)
        .flat_map(|encoder| &encoder.dispatches)
        .enumerate()
    {
        let arguments = ValidatedMetalDispatchArgumentsV1 {
            grid_threads: dispatch.grid_threads,
            threads_per_threadgroup: dispatch.threads_per_threadgroup,
            sequence_step_ordinal: dispatch.sequence_step_ordinal,
            chunk_ordinal: dispatch.chunk_ordinal,
        };
        digest = next_metal_dispatch_geometry_digest_v1(&digest, dispatch_ordinal, &arguments);
    }
    digest
}

#[cfg(test)]
pub(super) fn compare_metal_dispatch_argument_sequences_v1(
    left: &MetalSemanticRunTraceV1,
    right: &MetalSemanticRunTraceV1,
) -> usize {
    let dispatches = |trace: &MetalSemanticRunTraceV1| {
        trace
            .commands
            .iter()
            .flat_map(|command| &command.encoders)
            .flat_map(|encoder| &encoder.dispatches)
            .map(|dispatch| {
                (
                    dispatch.call_site,
                    dispatch.grid_threads,
                    dispatch.threads_per_threadgroup,
                    dispatch.sequence_step_ordinal,
                    dispatch.chunk_ordinal,
                )
            })
            .collect::<Vec<_>>()
    };
    let left = dispatches(left);
    let right = dispatches(right);
    left.iter()
        .zip(&right)
        .filter(|(left, right)| left != right)
        .count()
        + left.len().abs_diff(right.len())
}

#[cfg(test)]
pub(super) fn metal_semantic_trace_schema_identity_v2() -> String {
    stable_hash_string(
        "m3-micro-metal-semantic-trace-v2:actual-dispatch-invocation:grid-width-height-depth:threadgroup-width-height-depth:sequence-step:chunk:runtime-equality:no-provenance",
    )
}

#[cfg(test)]
pub(super) fn metal_semantic_trace_schema_identity_v3() -> String {
    stable_hash_string(
        "m3-micro-metal-semantic-trace-v3:actual-dispatch-invocation:actual-buffer-reference-index-offset:observed-resource-length:checked-required-span:role-access-contract:runtime-equality:no-object-identity",
    )
}

#[cfg(test)]
pub(super) fn metal_semantic_trace_schema_identity_v4() -> String {
    stable_hash_string(
        "m3-micro-metal-semantic-trace-v4:actual-pipeline-artifact:actual-dispatch-invocation:actual-buffer-reference-index-offset:runtime-pipeline-observation:single-source-binding:abc-exact:no-object-identity",
    )
}

#[cfg(test)]
pub(super) fn metal_pipeline_binding_argument_provenance_policy_identity_v1() -> String {
    stable_hash_string(
        "m3-micro-metal-pipeline-binding-provenance-v1:compile-source-digest:function-lookup-invocation:pipeline-creation-artifact:validate-record-call-mark:all-normal-bindings:abc-exact:modes-independent:no-object-identity",
    )
}

#[cfg(test)]
pub(super) fn metal_buffer_binding_argument_provenance_policy_identity_v1() -> String {
    stable_hash_string(
        "m3-micro-metal-buffer-binding-provenance-v1:single-invocation:validate-observe-record-call-mark:all-normal-bindings:abc-exact:modes-independent:no-object-identity",
    )
}

#[cfg(test)]
pub(super) fn metal_dispatch_argument_provenance_policy_identity_v1() -> String {
    stable_hash_string(
        "m3-micro-metal-dispatch-provenance-v1:single-invocation:validate-record-call-mark:all-normal-dispatches:abc-exact:modes-independent",
    )
}

#[cfg(test)]
pub(super) fn metal_topology_determinism_policy_identity_v1() -> String {
    stable_hash_string(
        "m3-micro-metal-topology-policy-v1:abc-independent-owned:exact-command-encoder-pipeline-binding-dispatch-terminal:witness-delta:exact-result-digests",
    )
}

#[cfg(test)]
pub(super) fn metal_semantic_trace_digest_v1(
    trace: &MetalSemanticRunTraceV1,
) -> Result<String, String> {
    let mut semantic = trace.clone();
    semantic.per_run_witness_delta = MetalPerRunWitnessDeltaV1::default();
    semantic.output_digest.clear();
    semantic.final_state_digest.clear();
    semantic.semantic_trace_digest.clear();
    for segment in &mut semantic.segments {
        segment.chunk_boundary_state_digest.clear();
    }
    serde_json::to_string(&semantic)
        .map(|value| stable_hash_string(&value))
        .map_err(|error| error.to_string())
}

#[cfg(test)]
pub(super) fn canonical_metal_semantic_trace_bytes_v1(
    trace: &MetalSemanticRunTraceV1,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(trace).map_err(|error| error.to_string())
}

#[cfg(test)]
pub(super) fn validate_metal_semantic_run_trace_v1(
    trace: &MetalSemanticRunTraceV1,
) -> Result<(), String> {
    let required_binding_contracts = (0..)
        .map_while(normal_metal_buffer_binding_contract_v2)
        .collect::<Vec<_>>();
    let dispatch_count = trace
        .commands
        .iter()
        .flat_map(|command| &command.encoders)
        .map(|encoder| encoder.dispatches.len())
        .sum::<usize>();
    let dispatch_provenance = &trace.dispatch_argument_provenance;
    let buffer_provenance = &trace.buffer_binding_argument_provenance;
    let pipeline_provenance = &trace.pipeline_binding_argument_provenance;
    let binding_count = trace
        .commands
        .iter()
        .flat_map(|command| &command.encoders)
        .map(|encoder| encoder.buffer_bindings.len())
        .sum::<usize>();
    let pipeline_binding_count = trace
        .commands
        .iter()
        .flat_map(|command| &command.encoders)
        .map(|encoder| encoder.pipeline_bindings.len())
        .sum::<usize>();
    if trace.trace_schema_version != METAL_SEMANTIC_TRACE_SCHEMA_VERSION_V4
        || trace.agent_role.is_empty()
        || trace.sequence_length == 0
        || trace.chunk_policy_identity.is_empty()
        || trace.commands.is_empty()
        || trace.commands.len() != trace.segments.len()
        || !trace.per_run_witness_delta.successful()
        || trace.output_digest.is_empty()
        || trace.final_state_digest.is_empty()
        || trace.semantic_trace_digest != metal_semantic_trace_digest_v1(trace)?
        || dispatch_count == 0
        || dispatch_provenance.actual_argument_capture_count != dispatch_count
        || dispatch_provenance.trace_record_count != dispatch_count
        || dispatch_provenance.argument_trace_mismatch_count != 0
        || dispatch_provenance.invalid_dimension_count != 0
        || dispatch_provenance.conversion_failure_count != 0
        || dispatch_provenance.geometry_digest != metal_dispatch_geometry_digest_v1(trace)
        || binding_count == 0
        || buffer_provenance.actual_buffer_capture_count != binding_count
        || buffer_provenance.trace_record_count != binding_count
        || buffer_provenance.argument_trace_mismatch_count != 0
        || buffer_provenance.invalid_offset_count != 0
        || buffer_provenance.insufficient_span_count != 0
        || buffer_provenance.duplicate_binding_index_count != 0
        || buffer_provenance.missing_required_role_count != 0
        || buffer_provenance.unexpected_role_count != 0
        || buffer_provenance.binding_sequence_digest
            != metal_buffer_binding_sequence_digest_v1(trace)
        || pipeline_binding_count == 0
        || pipeline_provenance.pipeline_artifact_capture_count != pipeline_binding_count
        || pipeline_provenance.trace_record_count != pipeline_binding_count
        || pipeline_provenance.artifact_trace_mismatch_count != 0
        || pipeline_provenance.incomplete_artifact_count != 0
        || pipeline_provenance.invalid_pipeline_observation_count != 0
        || pipeline_provenance.semantic_pipeline_digest != metal_pipeline_semantic_digest_v1(trace)
        || pipeline_provenance.pipeline_observation_digest
            != metal_pipeline_observation_digest_v1(trace)
        || pipeline_provenance.pipeline_binding_sequence_digest
            != metal_pipeline_binding_sequence_digest_v1(trace)
    {
        return Err("invalid Metal semantic run trace envelope".to_string());
    }
    for (command_ordinal, command) in trace.commands.iter().enumerate() {
        if command.command_ordinal != command_ordinal
            || command.encoder_count != command.encoders.len()
            || command.encoders.is_empty()
            || !command.committed
            || command.terminal_status != MetalCommandTerminalStatusV1::Completed
            || command.error_present
        {
            return Err(format!("invalid Metal command trace: {command_ordinal}"));
        }
        for (encoder_ordinal, encoder) in command.encoders.iter().enumerate() {
            if encoder.encoder_ordinal != encoder_ordinal
                || encoder.pipeline_bindings.len() != 1
                || encoder.buffer_bindings.len() != required_binding_contracts.len()
                || encoder.dispatches.is_empty()
            {
                return Err(format!(
                    "invalid Metal encoder trace: {command_ordinal}/{encoder_ordinal}"
                ));
            }
            let pipeline = &encoder.pipeline_bindings[0];
            let pipeline_metadata = metal_pipeline_metadata_from_trace_v1(pipeline);
            if pipeline.pipeline_binding_ordinal != 0
                || pipeline.encoder_ordinal != encoder_ordinal
                || validate_pipeline_artifact_metadata_v1(&pipeline_metadata).is_err()
                || !pipeline.binding_performed
            {
                return Err(format!(
                    "invalid Metal pipeline binding: {command_ordinal}/{encoder_ordinal}"
                ));
            }
            let mut indices = std::collections::BTreeSet::new();
            let mut roles = std::collections::BTreeSet::new();
            for (index, binding) in encoder.buffer_bindings.iter().enumerate() {
                let (expected_role, expected_access) = required_binding_contracts[index];
                if binding.binding_ordinal != index
                    || binding.binding_index != index
                    || binding.semantic_role != expected_role
                    || binding.semantic_access != expected_access
                    || binding.required_span_bytes == 0
                    || binding.byte_offset > binding.actual_resource_length_bytes
                    || binding.available_span_bytes
                        != binding.actual_resource_length_bytes - binding.byte_offset
                    || binding.required_span_bytes > binding.available_span_bytes
                    || !binding.binding_performed
                    || !indices.insert(binding.binding_index)
                    || !roles.insert(format!("{:?}", binding.semantic_role))
                {
                    return Err(format!(
                        "invalid Metal buffer binding: {command_ordinal}/{encoder_ordinal}/{index}"
                    ));
                }
            }
            for (dispatch_ordinal, dispatch) in encoder.dispatches.iter().enumerate() {
                if dispatch.dispatch_ordinal != dispatch_ordinal
                    || dispatch.call_site != MetalDispatchCallSiteV1::M3MicroForward
                    || dispatch.grid_threads.contains(&0)
                    || dispatch.threads_per_threadgroup.contains(&0)
                    || dispatch.sequence_step_ordinal != dispatch.chunk_start
                    || dispatch.chunk_ordinal != command_ordinal
                    || dispatch.chunk_start + dispatch.chunk_length > trace.sequence_length
                    || dispatch.chunk_length == 0
                    || !dispatch.dispatch_performed
                {
                    return Err(format!(
                        "invalid Metal dispatch trace: {command_ordinal}/{encoder_ordinal}/{dispatch_ordinal}"
                    ));
                }
            }
        }
        let actual_events = trace
            .call_site_events
            .iter()
            .filter(|event| event.command_ordinal == command_ordinal)
            .map(|event| event.call_site)
            .collect::<Vec<_>>();
        let mut expected_events = vec![
            MetalTraceCallSiteV1::CommandBufferCreated,
            MetalTraceCallSiteV1::EncoderCreated,
            MetalTraceCallSiteV1::PipelineBound,
        ];
        expected_events.extend(
            std::iter::repeat(MetalTraceCallSiteV1::BufferBound)
                .take(required_binding_contracts.len()),
        );
        expected_events.extend(
            std::iter::repeat(MetalTraceCallSiteV1::DispatchIssued).take(
                command
                    .encoders
                    .iter()
                    .map(|encoder| encoder.dispatches.len())
                    .sum(),
            ),
        );
        expected_events.extend([
            MetalTraceCallSiteV1::EncoderEnded,
            MetalTraceCallSiteV1::CommandCommitted,
            MetalTraceCallSiteV1::CommandCompleted,
            MetalTraceCallSiteV1::OutputReadback,
            MetalTraceCallSiteV1::StateReadback,
        ]);
        if actual_events != expected_events {
            return Err(format!(
                "Metal call-site ordering mismatch: {command_ordinal}"
            ));
        }
    }
    if trace
        .call_site_events
        .iter()
        .enumerate()
        .any(|(ordinal, event)| event.event_ordinal != ordinal)
        || trace.segments.iter().enumerate().any(|(ordinal, segment)| {
            segment.command_ordinal != ordinal
                || segment.chunk_boundary_state_digest.is_empty()
                || segment.chunk_start + segment.chunk_length > trace.sequence_length
        })
    {
        return Err("Metal trace ordinal or segment mismatch".to_string());
    }
    let encoded = canonical_metal_semantic_trace_bytes_v1(trace)?;
    let decoded: MetalSemanticRunTraceV1 =
        serde_json::from_slice(&encoded).map_err(|error| error.to_string())?;
    if decoded != *trace || canonical_metal_semantic_trace_bytes_v1(&decoded)? != encoded {
        return Err("non-canonical Metal semantic trace roundtrip".to_string());
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn compare_metal_semantic_run_traces_v1(
    scenario_identity: &str,
    run_pair: &str,
    left: &MetalSemanticRunTraceV1,
    right: &MetalSemanticRunTraceV1,
) -> Vec<MetalTopologyMismatchV1> {
    let mut mismatches = Vec::new();
    let mut push = |field: String,
                    command_ordinal,
                    encoder_ordinal,
                    dispatch_ordinal,
                    binding_index,
                    left: String,
                    right: String| {
        mismatches.push(MetalTopologyMismatchV1 {
            scenario_identity: scenario_identity.to_string(),
            run_pair: run_pair.to_string(),
            command_ordinal,
            encoder_ordinal,
            dispatch_ordinal,
            binding_index,
            field,
            left,
            right,
        });
    };
    macro_rules! compare_root {
        ($field:expr, $left:expr, $right:expr) => {
            if $left != $right {
                push(
                    $field.to_string(),
                    None,
                    None,
                    None,
                    None,
                    format!("{:?}", $left),
                    format!("{:?}", $right),
                );
            }
        };
    }
    compare_root!(
        "trace_schema_version",
        left.trace_schema_version,
        right.trace_schema_version
    );
    compare_root!("execution_mode", left.execution_mode, right.execution_mode);
    compare_root!("agent_role", left.agent_role, right.agent_role);
    compare_root!(
        "sequence_length",
        left.sequence_length,
        right.sequence_length
    );
    compare_root!(
        "chunk_policy_identity",
        left.chunk_policy_identity,
        right.chunk_policy_identity
    );
    compare_root!(
        "dispatch_argument_provenance.actual_argument_capture_count",
        left.dispatch_argument_provenance
            .actual_argument_capture_count,
        right
            .dispatch_argument_provenance
            .actual_argument_capture_count
    );
    compare_root!(
        "dispatch_argument_provenance.trace_record_count",
        left.dispatch_argument_provenance.trace_record_count,
        right.dispatch_argument_provenance.trace_record_count
    );
    compare_root!(
        "dispatch_argument_provenance.argument_trace_mismatch_count",
        left.dispatch_argument_provenance
            .argument_trace_mismatch_count,
        right
            .dispatch_argument_provenance
            .argument_trace_mismatch_count
    );
    compare_root!(
        "dispatch_argument_provenance.invalid_dimension_count",
        left.dispatch_argument_provenance.invalid_dimension_count,
        right.dispatch_argument_provenance.invalid_dimension_count
    );
    compare_root!(
        "dispatch_argument_provenance.conversion_failure_count",
        left.dispatch_argument_provenance.conversion_failure_count,
        right.dispatch_argument_provenance.conversion_failure_count
    );
    compare_root!(
        "dispatch_argument_provenance.geometry_digest",
        left.dispatch_argument_provenance.geometry_digest,
        right.dispatch_argument_provenance.geometry_digest
    );
    macro_rules! compare_buffer_provenance {
        ($field:ident) => {
            compare_root!(
                concat!("buffer_binding_argument_provenance.", stringify!($field)),
                left.buffer_binding_argument_provenance.$field,
                right.buffer_binding_argument_provenance.$field
            );
        };
    }
    compare_buffer_provenance!(actual_buffer_capture_count);
    compare_buffer_provenance!(trace_record_count);
    compare_buffer_provenance!(argument_trace_mismatch_count);
    compare_buffer_provenance!(invalid_offset_count);
    compare_buffer_provenance!(insufficient_span_count);
    compare_buffer_provenance!(duplicate_binding_index_count);
    compare_buffer_provenance!(missing_required_role_count);
    compare_buffer_provenance!(unexpected_role_count);
    compare_buffer_provenance!(binding_sequence_digest);
    macro_rules! compare_pipeline_provenance {
        ($field:ident) => {
            compare_root!(
                concat!("pipeline_binding_argument_provenance.", stringify!($field)),
                left.pipeline_binding_argument_provenance.$field,
                right.pipeline_binding_argument_provenance.$field
            );
        };
    }
    compare_pipeline_provenance!(pipeline_artifact_capture_count);
    compare_pipeline_provenance!(trace_record_count);
    compare_pipeline_provenance!(artifact_trace_mismatch_count);
    compare_pipeline_provenance!(incomplete_artifact_count);
    compare_pipeline_provenance!(invalid_pipeline_observation_count);
    compare_pipeline_provenance!(semantic_pipeline_digest);
    compare_pipeline_provenance!(pipeline_observation_digest);
    compare_pipeline_provenance!(pipeline_binding_sequence_digest);
    compare_root!("commands.len", left.commands.len(), right.commands.len());
    for (command_ordinal, (left_command, right_command)) in
        left.commands.iter().zip(&right.commands).enumerate()
    {
        macro_rules! compare_command {
            ($field:literal, $left:expr, $right:expr) => {
                if $left != $right {
                    push(
                        format!("commands[{command_ordinal}].{}", $field),
                        Some(command_ordinal),
                        None,
                        None,
                        None,
                        format!("{:?}", $left),
                        format!("{:?}", $right),
                    );
                }
            };
        }
        compare_command!(
            "command_ordinal",
            left_command.command_ordinal,
            right_command.command_ordinal
        );
        compare_command!(
            "encoder_count",
            left_command.encoder_count,
            right_command.encoder_count
        );
        compare_command!("committed", left_command.committed, right_command.committed);
        compare_command!(
            "terminal_status",
            left_command.terminal_status,
            right_command.terminal_status
        );
        compare_command!(
            "error_present",
            left_command.error_present,
            right_command.error_present
        );
        compare_command!(
            "encoders.len",
            left_command.encoders.len(),
            right_command.encoders.len()
        );
        for (encoder_ordinal, (left_encoder, right_encoder)) in left_command
            .encoders
            .iter()
            .zip(&right_command.encoders)
            .enumerate()
        {
            macro_rules! compare_encoder {
                ($field:literal, $left:expr, $right:expr) => {
                    if $left != $right {
                        push(
                            format!(
                                "commands[{command_ordinal}].encoders[{encoder_ordinal}].{}",
                                $field
                            ),
                            Some(command_ordinal),
                            Some(encoder_ordinal),
                            None,
                            None,
                            format!("{:?}", $left),
                            format!("{:?}", $right),
                        );
                    }
                };
            }
            compare_encoder!(
                "encoder_ordinal",
                left_encoder.encoder_ordinal,
                right_encoder.encoder_ordinal
            );
            compare_encoder!(
                "pipeline_bindings.len",
                left_encoder.pipeline_bindings.len(),
                right_encoder.pipeline_bindings.len()
            );
            for (pipeline_ordinal, (left_pipeline, right_pipeline)) in left_encoder
                .pipeline_bindings
                .iter()
                .zip(&right_encoder.pipeline_bindings)
                .enumerate()
            {
                macro_rules! compare_pipeline {
                    ($field:literal, $left:expr, $right:expr) => {
                        if $left != $right {
                            push(
                                format!("commands[{command_ordinal}].encoders[{encoder_ordinal}].pipeline_bindings[{pipeline_ordinal}].{}", $field),
                                Some(command_ordinal),
                                Some(encoder_ordinal),
                                None,
                                None,
                                format!("{:?}", $left),
                                format!("{:?}", $right),
                            );
                        }
                    };
                }
                compare_pipeline!(
                    "pipeline_binding_ordinal",
                    left_pipeline.pipeline_binding_ordinal,
                    right_pipeline.pipeline_binding_ordinal
                );
                compare_pipeline!(
                    "encoder_ordinal",
                    left_pipeline.encoder_ordinal,
                    right_pipeline.encoder_ordinal
                );
                compare_pipeline!(
                    "artifact_schema_version",
                    left_pipeline.artifact_schema_version,
                    right_pipeline.artifact_schema_version
                );
                compare_pipeline!(
                    "function_name",
                    left_pipeline.function_name,
                    right_pipeline.function_name
                );
                compare_pipeline!(
                    "shader_source_digest",
                    left_pipeline.shader_source_digest,
                    right_pipeline.shader_source_digest
                );
                compare_pipeline!(
                    "library_build_policy_identity",
                    left_pipeline.library_build_policy_identity,
                    right_pipeline.library_build_policy_identity
                );
                compare_pipeline!(
                    "function_lookup_policy_identity",
                    left_pipeline.function_lookup_policy_identity,
                    right_pipeline.function_lookup_policy_identity
                );
                compare_pipeline!(
                    "function_constant_identity",
                    left_pipeline.function_constant_identity,
                    right_pipeline.function_constant_identity
                );
                compare_pipeline!(
                    "kernel_abi_identity",
                    left_pipeline.kernel_abi_identity,
                    right_pipeline.kernel_abi_identity
                );
                compare_pipeline!(
                    "parameter_buffer_layout_identity",
                    left_pipeline.parameter_buffer_layout_identity,
                    right_pipeline.parameter_buffer_layout_identity
                );
                compare_pipeline!(
                    "semantic_pipeline_identity",
                    left_pipeline.semantic_pipeline_identity,
                    right_pipeline.semantic_pipeline_identity
                );
                compare_pipeline!(
                    "thread_execution_width",
                    left_pipeline.thread_execution_width,
                    right_pipeline.thread_execution_width
                );
                compare_pipeline!(
                    "max_total_threads_per_threadgroup",
                    left_pipeline.max_total_threads_per_threadgroup,
                    right_pipeline.max_total_threads_per_threadgroup
                );
                compare_pipeline!(
                    "binding_performed",
                    left_pipeline.binding_performed,
                    right_pipeline.binding_performed
                );
            }
            compare_encoder!(
                "buffer_bindings.len",
                left_encoder.buffer_bindings.len(),
                right_encoder.buffer_bindings.len()
            );
            for (binding_ordinal, (left_binding, right_binding)) in left_encoder
                .buffer_bindings
                .iter()
                .zip(&right_encoder.buffer_bindings)
                .enumerate()
            {
                macro_rules! compare_binding {
                    ($field:literal, $left:expr, $right:expr) => {
                        if $left != $right {
                            push(
                                format!("commands[{command_ordinal}].encoders[{encoder_ordinal}].buffer_bindings[{binding_ordinal}].{}", $field),
                                Some(command_ordinal),
                                Some(encoder_ordinal),
                                None,
                                Some(left_binding.binding_index),
                                format!("{:?}", $left),
                                format!("{:?}", $right),
                            );
                        }
                    };
                }
                compare_binding!(
                    "binding_ordinal",
                    left_binding.binding_ordinal,
                    right_binding.binding_ordinal
                );
                compare_binding!(
                    "binding_index",
                    left_binding.binding_index,
                    right_binding.binding_index
                );
                compare_binding!(
                    "semantic_role",
                    left_binding.semantic_role,
                    right_binding.semantic_role
                );
                compare_binding!(
                    "byte_offset",
                    left_binding.byte_offset,
                    right_binding.byte_offset
                );
                compare_binding!(
                    "semantic_access",
                    left_binding.semantic_access,
                    right_binding.semantic_access
                );
                compare_binding!(
                    "actual_resource_length_bytes",
                    left_binding.actual_resource_length_bytes,
                    right_binding.actual_resource_length_bytes
                );
                compare_binding!(
                    "required_span_bytes",
                    left_binding.required_span_bytes,
                    right_binding.required_span_bytes
                );
                compare_binding!(
                    "available_span_bytes",
                    left_binding.available_span_bytes,
                    right_binding.available_span_bytes
                );
                compare_binding!(
                    "binding_performed",
                    left_binding.binding_performed,
                    right_binding.binding_performed
                );
            }
            compare_encoder!(
                "dispatches.len",
                left_encoder.dispatches.len(),
                right_encoder.dispatches.len()
            );
            for (dispatch_ordinal, (left_dispatch, right_dispatch)) in left_encoder
                .dispatches
                .iter()
                .zip(&right_encoder.dispatches)
                .enumerate()
            {
                macro_rules! compare_dispatch {
                    ($field:literal, $left:expr, $right:expr) => {
                        if $left != $right {
                            push(
                                format!("commands[{command_ordinal}].encoders[{encoder_ordinal}].dispatches[{dispatch_ordinal}].{}", $field),
                                Some(command_ordinal),
                                Some(encoder_ordinal),
                                Some(dispatch_ordinal),
                                None,
                                format!("{:?}", $left),
                                format!("{:?}", $right),
                            );
                        }
                    };
                }
                compare_dispatch!(
                    "dispatch_ordinal",
                    left_dispatch.dispatch_ordinal,
                    right_dispatch.dispatch_ordinal
                );
                compare_dispatch!(
                    "call_site",
                    left_dispatch.call_site,
                    right_dispatch.call_site
                );
                compare_dispatch!(
                    "grid_threads",
                    left_dispatch.grid_threads,
                    right_dispatch.grid_threads
                );
                compare_dispatch!(
                    "threads_per_threadgroup",
                    left_dispatch.threads_per_threadgroup,
                    right_dispatch.threads_per_threadgroup
                );
                compare_dispatch!(
                    "sequence_step_ordinal",
                    left_dispatch.sequence_step_ordinal,
                    right_dispatch.sequence_step_ordinal
                );
                compare_dispatch!(
                    "chunk_ordinal",
                    left_dispatch.chunk_ordinal,
                    right_dispatch.chunk_ordinal
                );
                compare_dispatch!(
                    "chunk_start",
                    left_dispatch.chunk_start,
                    right_dispatch.chunk_start
                );
                compare_dispatch!(
                    "chunk_length",
                    left_dispatch.chunk_length,
                    right_dispatch.chunk_length
                );
                compare_dispatch!(
                    "dispatch_performed",
                    left_dispatch.dispatch_performed,
                    right_dispatch.dispatch_performed
                );
            }
        }
    }
    compare_root!("segments.len", left.segments.len(), right.segments.len());
    for (segment_ordinal, (left_segment, right_segment)) in
        left.segments.iter().zip(&right.segments).enumerate()
    {
        macro_rules! compare_segment {
            ($field:literal, $left:expr, $right:expr) => {
                if $left != $right {
                    push(
                        format!("segments[{segment_ordinal}].{}", $field),
                        Some(left_segment.command_ordinal),
                        None,
                        None,
                        None,
                        format!("{:?}", $left),
                        format!("{:?}", $right),
                    );
                }
            };
        }
        compare_segment!(
            "chunk_ordinal",
            left_segment.chunk_ordinal,
            right_segment.chunk_ordinal
        );
        compare_segment!(
            "chunk_start",
            left_segment.chunk_start,
            right_segment.chunk_start
        );
        compare_segment!(
            "chunk_length",
            left_segment.chunk_length,
            right_segment.chunk_length
        );
        compare_segment!(
            "command_ordinal",
            left_segment.command_ordinal,
            right_segment.command_ordinal
        );
        compare_segment!(
            "chunk_boundary_state_digest",
            left_segment.chunk_boundary_state_digest,
            right_segment.chunk_boundary_state_digest
        );
    }
    compare_root!(
        "call_site_events",
        left.call_site_events,
        right.call_site_events
    );
    macro_rules! compare_witness {
        ($field:ident) => {
            compare_root!(
                concat!("per_run_witness_delta.", stringify!($field)),
                left.per_run_witness_delta.$field,
                right.per_run_witness_delta.$field
            );
        };
    }
    compare_witness!(command_buffers_created);
    compare_witness!(command_buffers_committed);
    compare_witness!(command_buffers_completed);
    compare_witness!(command_buffer_failures);
    compare_witness!(compute_encoders_created);
    compare_witness!(dispatches_attempted);
    compare_witness!(dispatches_performed);
    compare_witness!(output_readbacks);
    compare_witness!(state_readbacks);
    compare_witness!(output_poison_remaining);
    compare_witness!(state_poison_remaining);
    compare_witness!(cpu_fallback_attempts);
    compare_witness!(cpu_fallback_executions);
    compare_root!("output_digest", left.output_digest, right.output_digest);
    compare_root!(
        "final_state_digest",
        left.final_state_digest,
        right.final_state_digest
    );
    compare_root!(
        "semantic_trace_digest",
        left.semantic_trace_digest,
        right.semantic_trace_digest
    );
    mismatches
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroMetalParityPolicyV1 {
    pub output_absolute_tolerance: f32,
    pub output_relative_tolerance: f32,
    pub state_absolute_tolerance: f32,
    pub state_relative_tolerance: f32,
    pub require_exact_shape: bool,
    pub require_finite: bool,
    pub require_no_cpu_fallback: bool,
}

impl Default for M3MicroMetalParityPolicyV1 {
    fn default() -> Self {
        Self {
            output_absolute_tolerance: 2.0e-5,
            output_relative_tolerance: 2.0e-4,
            state_absolute_tolerance: 3.0e-5,
            state_relative_tolerance: 3.0e-4,
            require_exact_shape: true,
            require_finite: true,
            require_no_cpu_fallback: true,
        }
    }
}

impl M3MicroMetalParityPolicyV1 {
    pub fn identity(&self) -> String {
        if self == &Self::default() {
            m3_micro_metal_parity_policy_identity_v1()
        } else {
            stable_hash_string(&format!("m3-micro-metal-parity-policy-custom:{self:?}"))
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct M3MicroMetalParityMetricsV1 {
    pub element_count: usize,
    pub max_absolute_error: f32,
    pub max_relative_error: f32,
    pub max_error_index: usize,
    pub max_error_path: String,
    pub mean_absolute_error: f32,
    pub mismatch_count: usize,
}

impl M3MicroMetalParityMetricsV1 {
    pub fn compare(
        cpu: &[f32],
        metal: &[f32],
        absolute_tolerance: f32,
        relative_tolerance: f32,
        path_prefix: &str,
    ) -> Result<Self, M3MicroMetalErrorV1> {
        if cpu.len() != metal.len() || cpu.is_empty() {
            return Err(M3MicroMetalErrorV1::MetalShapeMismatch);
        }
        if cpu.iter().chain(metal).any(|value| !value.is_finite()) {
            return Err(M3MicroMetalErrorV1::MetalNonFiniteOutput);
        }
        let mut metrics = Self {
            element_count: cpu.len(),
            ..Self::default()
        };
        let mut absolute_sum = 0.0f64;
        for (index, (cpu_value, metal_value)) in cpu.iter().zip(metal).enumerate() {
            let absolute = (*cpu_value - *metal_value).abs();
            let scale = cpu_value.abs().max(metal_value.abs());
            let relative = if scale == 0.0 { 0.0 } else { absolute / scale };
            absolute_sum += absolute as f64;
            if absolute > metrics.max_absolute_error {
                metrics.max_absolute_error = absolute;
                metrics.max_error_index = index;
                metrics.max_error_path = format!("{path_prefix}[{index}]");
            }
            metrics.max_relative_error = metrics.max_relative_error.max(relative);
            if absolute > absolute_tolerance + relative_tolerance * scale {
                metrics.mismatch_count += 1;
            }
        }
        metrics.mean_absolute_error = (absolute_sum / cpu.len() as f64) as f32;
        if metrics.max_error_path.is_empty() {
            metrics.max_error_path = format!("{path_prefix}[0]");
        }
        Ok(metrics)
    }

    pub fn merge(&mut self, value: &Self) {
        let combined_count = self.element_count + value.element_count;
        if self.element_count == 0 || value.max_absolute_error > self.max_absolute_error {
            self.max_absolute_error = value.max_absolute_error;
            self.max_error_index = value.max_error_index;
            self.max_error_path = value.max_error_path.clone();
        }
        self.max_relative_error = self.max_relative_error.max(value.max_relative_error);
        if combined_count > 0 {
            self.mean_absolute_error = ((self.mean_absolute_error as f64
                * self.element_count as f64)
                + (value.mean_absolute_error as f64 * value.element_count as f64))
                as f32
                / combined_count as f32;
        }
        self.element_count = combined_count;
        self.mismatch_count += value.mismatch_count;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct M3MicroMetalBufferSizesV1 {
    pub parameter_bytes: usize,
    pub input_bytes: usize,
    pub initial_state_bytes: usize,
    pub output_bytes: usize,
    pub final_state_bytes: usize,
    pub metadata_bytes: usize,
    pub witness_bytes: usize,
}

struct UploadedModelV1 {
    config: M3MicroConfig,
    layout: M3MicroMetalLayoutV1,
    parameters: metal::Buffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalPipelineLifecycleEventKindV1 {
    LibraryCompileAttempted,
    LibraryCompileSucceeded,
    LibraryCompileFailed,
    FunctionLookupAttempted,
    FunctionLookupSucceeded,
    FunctionLookupFailed,
    PipelineCreationAttempted,
    PipelineCreationSucceeded,
    PipelineCreationFailed,
    PipelineObservationCaptured,
    PipelineArtifactCreated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalPipelineLifecycleEventV1 {
    pub event_ordinal: usize,
    pub executor_construction_ordinal: usize,
    pub kind: MetalPipelineLifecycleEventKindV1,
    pub function_name: Option<String>,
    pub shader_source_digest: Option<String>,
    pub semantic_pipeline_identity: Option<String>,
    pub error_kind: Option<M3MicroMetalErrorV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum MetalPipelineConstructionStatusV1 {
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalPipelineConstructionEvidenceV1 {
    pub executor_construction_ordinal: usize,
    pub events: Vec<MetalPipelineLifecycleEventV1>,
    pub artifact_semantic_pipeline_identity: Option<String>,
    pub status: MetalPipelineConstructionStatusV1,
}

#[cfg(test)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalPipelineLifecycleSnapshotV2 {
    pub event_sequence: u64,
    pub library_compile_attempted: u64,
    pub library_compile_succeeded: u64,
    pub library_compile_failed: u64,
    pub function_lookup_attempted: u64,
    pub function_lookup_succeeded: u64,
    pub function_lookup_failed: u64,
    pub pipeline_creation_attempted: u64,
    pub pipeline_creation_succeeded: u64,
    pub pipeline_creation_failed: u64,
    pub pipeline_observations_captured: u64,
    pub pipeline_artifacts_created: u64,
    pub lifecycle_digest_state: String,
    pub ordering_digest_state: String,
    pub function_sequence_digest_state: String,
    pub shader_sequence_digest_state: String,
    pub pipeline_identity_sequence_digest_state: String,
}

#[cfg(test)]
impl MetalPipelineLifecycleSnapshotV2 {
    pub(super) fn categorized_event_count(&self) -> Result<u64, String> {
        [
            self.library_compile_attempted,
            self.library_compile_succeeded,
            self.library_compile_failed,
            self.function_lookup_attempted,
            self.function_lookup_succeeded,
            self.function_lookup_failed,
            self.pipeline_creation_attempted,
            self.pipeline_creation_succeeded,
            self.pipeline_creation_failed,
            self.pipeline_observations_captured,
            self.pipeline_artifacts_created,
        ]
        .into_iter()
        .try_fold(0u64, |total, value| {
            total
                .checked_add(value)
                .ok_or_else(|| "Metal lifecycle snapshot counter overflow".to_string())
        })
    }

    pub(super) fn validate_consistency(&self) -> Result<(), String> {
        if self.lifecycle_digest_state.is_empty()
            || self.ordering_digest_state.is_empty()
            || self.function_sequence_digest_state.is_empty()
            || self.shader_sequence_digest_state.is_empty()
            || self.pipeline_identity_sequence_digest_state.is_empty()
            || self.categorized_event_count()? != self.event_sequence
        {
            return Err("inconsistent Metal lifecycle snapshot".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetalPipelineLifecycleDeltaV2 {
    pub event_sequence_delta: u64,
    pub library_compile_attempted_delta: u64,
    pub library_compile_succeeded_delta: u64,
    pub library_compile_failed_delta: u64,
    pub function_lookup_attempted_delta: u64,
    pub function_lookup_succeeded_delta: u64,
    pub function_lookup_failed_delta: u64,
    pub pipeline_creation_attempted_delta: u64,
    pub pipeline_creation_succeeded_delta: u64,
    pub pipeline_creation_failed_delta: u64,
    pub pipeline_observations_captured_delta: u64,
    pub pipeline_artifacts_created_delta: u64,
    pub before_lifecycle_digest: String,
    pub after_lifecycle_digest: String,
}

#[cfg(test)]
impl MetalPipelineLifecycleDeltaV2 {
    pub(super) fn checked_between(
        before: &MetalPipelineLifecycleSnapshotV2,
        after: &MetalPipelineLifecycleSnapshotV2,
    ) -> Result<Self, String> {
        before.validate_consistency()?;
        after.validate_consistency()?;
        let subtract = |after: u64, before: u64| {
            after
                .checked_sub(before)
                .ok_or_else(|| "Metal lifecycle monitor counter rolled back".to_string())
        };
        let delta = Self {
            event_sequence_delta: subtract(after.event_sequence, before.event_sequence)?,
            library_compile_attempted_delta: subtract(
                after.library_compile_attempted,
                before.library_compile_attempted,
            )?,
            library_compile_succeeded_delta: subtract(
                after.library_compile_succeeded,
                before.library_compile_succeeded,
            )?,
            library_compile_failed_delta: subtract(
                after.library_compile_failed,
                before.library_compile_failed,
            )?,
            function_lookup_attempted_delta: subtract(
                after.function_lookup_attempted,
                before.function_lookup_attempted,
            )?,
            function_lookup_succeeded_delta: subtract(
                after.function_lookup_succeeded,
                before.function_lookup_succeeded,
            )?,
            function_lookup_failed_delta: subtract(
                after.function_lookup_failed,
                before.function_lookup_failed,
            )?,
            pipeline_creation_attempted_delta: subtract(
                after.pipeline_creation_attempted,
                before.pipeline_creation_attempted,
            )?,
            pipeline_creation_succeeded_delta: subtract(
                after.pipeline_creation_succeeded,
                before.pipeline_creation_succeeded,
            )?,
            pipeline_creation_failed_delta: subtract(
                after.pipeline_creation_failed,
                before.pipeline_creation_failed,
            )?,
            pipeline_observations_captured_delta: subtract(
                after.pipeline_observations_captured,
                before.pipeline_observations_captured,
            )?,
            pipeline_artifacts_created_delta: subtract(
                after.pipeline_artifacts_created,
                before.pipeline_artifacts_created,
            )?,
            before_lifecycle_digest: before.lifecycle_digest_state.clone(),
            after_lifecycle_digest: after.lifecycle_digest_state.clone(),
        };
        let categorized_delta = [
            delta.library_compile_attempted_delta,
            delta.library_compile_succeeded_delta,
            delta.library_compile_failed_delta,
            delta.function_lookup_attempted_delta,
            delta.function_lookup_succeeded_delta,
            delta.function_lookup_failed_delta,
            delta.pipeline_creation_attempted_delta,
            delta.pipeline_creation_succeeded_delta,
            delta.pipeline_creation_failed_delta,
            delta.pipeline_observations_captured_delta,
            delta.pipeline_artifacts_created_delta,
        ]
        .into_iter()
        .try_fold(0u64, |total, value| {
            total
                .checked_add(value)
                .ok_or_else(|| "Metal lifecycle delta counter overflow".to_string())
        })?;
        let digest_changed = delta.before_lifecycle_digest != delta.after_lifecycle_digest;
        if categorized_delta != delta.event_sequence_delta
            || (delta.event_sequence_delta == 0 && digest_changed)
            || (delta.event_sequence_delta > 0 && !digest_changed)
        {
            return Err("contradictory Metal lifecycle monitor delta".to_string());
        }
        Ok(delta)
    }

    pub(super) fn is_zero(&self) -> bool {
        self.event_sequence_delta == 0
            && self.library_compile_attempted_delta == 0
            && self.library_compile_succeeded_delta == 0
            && self.library_compile_failed_delta == 0
            && self.function_lookup_attempted_delta == 0
            && self.function_lookup_succeeded_delta == 0
            && self.function_lookup_failed_delta == 0
            && self.pipeline_creation_attempted_delta == 0
            && self.pipeline_creation_succeeded_delta == 0
            && self.pipeline_creation_failed_delta == 0
            && self.pipeline_observations_captured_delta == 0
            && self.pipeline_artifacts_created_delta == 0
            && !self.before_lifecycle_digest.is_empty()
            && self.before_lifecycle_digest == self.after_lifecycle_digest
    }
}

#[cfg(test)]
#[derive(Clone)]
struct MetalPipelineLifecycleMonitorV2 {
    state: std::sync::Arc<std::sync::Mutex<MetalPipelineLifecycleSnapshotV2>>,
}

#[cfg(test)]
impl MetalPipelineLifecycleMonitorV2 {
    fn enabled() -> Self {
        Self {
            state: std::sync::Arc::new(std::sync::Mutex::new(MetalPipelineLifecycleSnapshotV2 {
                lifecycle_digest_state: stable_hash_string(
                    "m3-micro-metal-live-lifecycle-v2:empty",
                ),
                ordering_digest_state: stable_hash_string(
                    "m3-micro-metal-live-lifecycle-ordering-v3:empty",
                ),
                function_sequence_digest_state: stable_hash_string(
                    "m3-micro-metal-live-lifecycle-function-sequence-v3:empty",
                ),
                shader_sequence_digest_state: stable_hash_string(
                    "m3-micro-metal-live-lifecycle-shader-sequence-v3:empty",
                ),
                pipeline_identity_sequence_digest_state: stable_hash_string(
                    "m3-micro-metal-live-lifecycle-pipeline-sequence-v3:empty",
                ),
                ..MetalPipelineLifecycleSnapshotV2::default()
            })),
        }
    }

    fn record_actual_event_v2(
        &self,
        event: &MetalPipelineLifecycleEventV1,
    ) -> Result<(), M3MicroMetalErrorV1> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| M3MicroMetalErrorV1::MetalEncodingFailed)?;
        let next_sequence = state
            .event_sequence
            .checked_add(1)
            .ok_or(M3MicroMetalErrorV1::MetalEncodingFailed)?;
        if u64::try_from(event.event_ordinal).ok() != Some(next_sequence) {
            return Err(M3MicroMetalErrorV1::MetalEncodingFailed);
        }
        let counter = match event.kind {
            MetalPipelineLifecycleEventKindV1::LibraryCompileAttempted => {
                &mut state.library_compile_attempted
            }
            MetalPipelineLifecycleEventKindV1::LibraryCompileSucceeded => {
                &mut state.library_compile_succeeded
            }
            MetalPipelineLifecycleEventKindV1::LibraryCompileFailed => {
                &mut state.library_compile_failed
            }
            MetalPipelineLifecycleEventKindV1::FunctionLookupAttempted => {
                &mut state.function_lookup_attempted
            }
            MetalPipelineLifecycleEventKindV1::FunctionLookupSucceeded => {
                &mut state.function_lookup_succeeded
            }
            MetalPipelineLifecycleEventKindV1::FunctionLookupFailed => {
                &mut state.function_lookup_failed
            }
            MetalPipelineLifecycleEventKindV1::PipelineCreationAttempted => {
                &mut state.pipeline_creation_attempted
            }
            MetalPipelineLifecycleEventKindV1::PipelineCreationSucceeded => {
                &mut state.pipeline_creation_succeeded
            }
            MetalPipelineLifecycleEventKindV1::PipelineCreationFailed => {
                &mut state.pipeline_creation_failed
            }
            MetalPipelineLifecycleEventKindV1::PipelineObservationCaptured => {
                &mut state.pipeline_observations_captured
            }
            MetalPipelineLifecycleEventKindV1::PipelineArtifactCreated => {
                &mut state.pipeline_artifacts_created
            }
        };
        *counter = counter
            .checked_add(1)
            .ok_or(M3MicroMetalErrorV1::MetalEncodingFailed)?;
        let event_semantic = serde_json::to_string(&(
            event.event_ordinal,
            event.executor_construction_ordinal,
            event.kind,
            &event.function_name,
            &event.shader_source_digest,
            &event.semantic_pipeline_identity,
            event.error_kind.as_ref().map(|error| format!("{error:?}")),
        ))
        .map_err(|_| M3MicroMetalErrorV1::MetalEncodingFailed)?;
        state.lifecycle_digest_state = stable_hash_string(&format!(
            "{}:{event_semantic}",
            state.lifecycle_digest_state
        ));
        state.ordering_digest_state =
            stable_hash_string(&format!("{}:{event_semantic}", state.ordering_digest_state));
        let function_semantic =
            serde_json::to_string(&(event.event_ordinal, event.kind, &event.function_name))
                .map_err(|_| M3MicroMetalErrorV1::MetalEncodingFailed)?;
        state.function_sequence_digest_state = stable_hash_string(&format!(
            "{}:{function_semantic}",
            state.function_sequence_digest_state
        ));
        let shader_semantic =
            serde_json::to_string(&(event.event_ordinal, event.kind, &event.shader_source_digest))
                .map_err(|_| M3MicroMetalErrorV1::MetalEncodingFailed)?;
        state.shader_sequence_digest_state = stable_hash_string(&format!(
            "{}:{shader_semantic}",
            state.shader_sequence_digest_state
        ));
        let pipeline_semantic = serde_json::to_string(&(
            event.event_ordinal,
            event.kind,
            &event.semantic_pipeline_identity,
        ))
        .map_err(|_| M3MicroMetalErrorV1::MetalEncodingFailed)?;
        state.pipeline_identity_sequence_digest_state = stable_hash_string(&format!(
            "{}:{pipeline_semantic}",
            state.pipeline_identity_sequence_digest_state
        ));
        state.event_sequence = next_sequence;
        Ok(())
    }

    fn snapshot(&self) -> Result<MetalPipelineLifecycleSnapshotV2, String> {
        let snapshot = self
            .state
            .lock()
            .map_err(|_| "Metal lifecycle monitor lock is poisoned".to_string())?
            .clone();
        snapshot.validate_consistency()?;
        Ok(snapshot)
    }
}

enum MetalPipelineLifecycleCollectionV1 {
    Disabled,
    #[cfg(test)]
    Collect {
        executor_construction_ordinal: usize,
        events: Vec<MetalPipelineLifecycleEventV1>,
        monitor: MetalPipelineLifecycleMonitorV2,
    },
}

impl MetalPipelineLifecycleCollectionV1 {
    #[cfg(test)]
    fn collect(executor_construction_ordinal: usize) -> Self {
        Self::Collect {
            executor_construction_ordinal,
            events: Vec::new(),
            monitor: MetalPipelineLifecycleMonitorV2::enabled(),
        }
    }

    #[cfg(test)]
    fn monitor(&self) -> Option<MetalPipelineLifecycleMonitorV2> {
        match self {
            Self::Disabled => None,
            Self::Collect { monitor, .. } => Some(monitor.clone()),
        }
    }

    #[cfg(test)]
    fn emit_actual_lifecycle_event_v2(
        &mut self,
        kind: LifecycleEventKindArgumentV1,
        function_name: Option<&str>,
        shader_source_digest: Option<&str>,
        semantic_pipeline_identity: Option<&str>,
        error_kind: Option<M3MicroMetalErrorV1>,
    ) -> Result<(), M3MicroMetalErrorV1> {
        if let Self::Collect {
            executor_construction_ordinal,
            events,
            monitor,
        } = self
        {
            let event = MetalPipelineLifecycleEventV1 {
                event_ordinal: events.len() + 1,
                executor_construction_ordinal: *executor_construction_ordinal,
                kind,
                function_name: function_name.map(str::to_string),
                shader_source_digest: shader_source_digest.map(str::to_string),
                semantic_pipeline_identity: semantic_pipeline_identity.map(str::to_string),
                error_kind,
            };
            monitor.record_actual_event_v2(&event)?;
            events.push(event);
        }
        Ok(())
    }

    #[cfg(test)]
    fn into_evidence(
        self,
        status: MetalPipelineConstructionStatusV1,
        artifact_semantic_pipeline_identity: Option<String>,
    ) -> Option<MetalPipelineConstructionEvidenceV1> {
        match self {
            Self::Disabled => None,
            Self::Collect {
                executor_construction_ordinal,
                events,
                ..
            } => Some(MetalPipelineConstructionEvidenceV1 {
                executor_construction_ordinal,
                events,
                artifact_semantic_pipeline_identity,
                status,
            }),
        }
    }
}

#[cfg(test)]
type LifecycleEventKindArgumentV1 = MetalPipelineLifecycleEventKindV1;

fn compile_library_with_lifecycle_events_v1(
    device: &Device,
    shader_source: &str,
    _shader_source_digest: &str,
    options: &CompileOptions,
    _lifecycle: &mut MetalPipelineLifecycleCollectionV1,
) -> Result<metal::Library, M3MicroMetalErrorV1> {
    #[cfg(test)]
    _lifecycle.emit_actual_lifecycle_event_v2(
        MetalPipelineLifecycleEventKindV1::LibraryCompileAttempted,
        None,
        Some(_shader_source_digest),
        None,
        None,
    )?;
    let result = device.new_library_with_source(shader_source, options);
    match result {
        Ok(library) => {
            #[cfg(test)]
            _lifecycle.emit_actual_lifecycle_event_v2(
                MetalPipelineLifecycleEventKindV1::LibraryCompileSucceeded,
                None,
                Some(_shader_source_digest),
                None,
                None,
            )?;
            Ok(library)
        }
        Err(_) => {
            #[cfg(test)]
            _lifecycle.emit_actual_lifecycle_event_v2(
                MetalPipelineLifecycleEventKindV1::LibraryCompileFailed,
                None,
                Some(_shader_source_digest),
                None,
                Some(M3MicroMetalErrorV1::MetalLibraryCreationFailed),
            )?;
            Err(M3MicroMetalErrorV1::MetalLibraryCreationFailed)
        }
    }
}

fn lookup_function_with_lifecycle_events_v1(
    library: &metal::LibraryRef,
    function_name: &str,
    _shader_source_digest: &str,
    _lifecycle: &mut MetalPipelineLifecycleCollectionV1,
) -> Result<metal::Function, M3MicroMetalErrorV1> {
    #[cfg(test)]
    _lifecycle.emit_actual_lifecycle_event_v2(
        MetalPipelineLifecycleEventKindV1::FunctionLookupAttempted,
        Some(function_name),
        Some(_shader_source_digest),
        None,
        None,
    )?;
    let result = library.get_function(function_name, None);
    match result {
        Ok(function) => {
            #[cfg(test)]
            _lifecycle.emit_actual_lifecycle_event_v2(
                MetalPipelineLifecycleEventKindV1::FunctionLookupSucceeded,
                Some(function_name),
                Some(_shader_source_digest),
                None,
                None,
            )?;
            Ok(function)
        }
        Err(_) => {
            #[cfg(test)]
            _lifecycle.emit_actual_lifecycle_event_v2(
                MetalPipelineLifecycleEventKindV1::FunctionLookupFailed,
                Some(function_name),
                Some(_shader_source_digest),
                None,
                Some(M3MicroMetalErrorV1::MetalFunctionNotFound),
            )?;
            Err(M3MicroMetalErrorV1::MetalFunctionNotFound)
        }
    }
}

fn create_pipeline_with_lifecycle_events_v1(
    device: &Device,
    function: &metal::FunctionRef,
    _function_name: &str,
    _shader_source_digest: &str,
    _lifecycle: &mut MetalPipelineLifecycleCollectionV1,
) -> Result<ComputePipelineState, M3MicroMetalErrorV1> {
    #[cfg(test)]
    _lifecycle.emit_actual_lifecycle_event_v2(
        MetalPipelineLifecycleEventKindV1::PipelineCreationAttempted,
        Some(_function_name),
        Some(_shader_source_digest),
        None,
        None,
    )?;
    let result = device.new_compute_pipeline_state_with_function(function);
    match result {
        Ok(pipeline) => {
            #[cfg(test)]
            _lifecycle.emit_actual_lifecycle_event_v2(
                MetalPipelineLifecycleEventKindV1::PipelineCreationSucceeded,
                Some(_function_name),
                Some(_shader_source_digest),
                None,
                None,
            )?;
            Ok(pipeline)
        }
        Err(_) => {
            #[cfg(test)]
            _lifecycle.emit_actual_lifecycle_event_v2(
                MetalPipelineLifecycleEventKindV1::PipelineCreationFailed,
                Some(_function_name),
                Some(_shader_source_digest),
                None,
                Some(M3MicroMetalErrorV1::MetalPipelineCreationFailed),
            )?;
            Err(M3MicroMetalErrorV1::MetalPipelineCreationFailed)
        }
    }
}

fn build_m3_micro_pipeline_artifact_v1(
    device: &Device,
    lifecycle: &mut MetalPipelineLifecycleCollectionV1,
) -> Result<M3MicroMetalPipelineArtifactV1, M3MicroMetalErrorV1> {
    let shader_source = M3_MICRO_METAL_SHADER_V1;
    let shader_source_digest = stable_hash_string(shader_source);
    let library_build_policy_identity = metal_library_build_policy_identity_v1();
    let function_lookup_policy_identity = metal_function_lookup_policy_identity_v1();
    let function_constant_identity = metal_function_constant_identity_v1();
    let kernel_abi_identity = metal_kernel_abi_identity_v1();
    let parameter_buffer_layout_identity = metal_parameter_buffer_layout_identity_v1();

    let options = CompileOptions::new();
    options.set_fast_math_enabled(false);
    let library = compile_library_with_lifecycle_events_v1(
        device,
        shader_source,
        &shader_source_digest,
        &options,
        lifecycle,
    )?;
    let lookup = MetalFunctionLookupInvocationV1 {
        library: &library,
        function_name: METAL_FUNCTION_IDENTITY_V1,
    };
    let function = lookup_function_with_lifecycle_events_v1(
        lookup.library,
        lookup.function_name,
        &shader_source_digest,
        lifecycle,
    )?;
    let creation = MetalPipelineCreationInvocationV1 {
        function: &function,
        function_name: lookup.function_name,
        shader_source_digest: &shader_source_digest,
        library_build_policy_identity: &library_build_policy_identity,
        function_lookup_policy_identity: &function_lookup_policy_identity,
        function_constant_identity: &function_constant_identity,
        kernel_abi_identity: &kernel_abi_identity,
        parameter_buffer_layout_identity: &parameter_buffer_layout_identity,
    };
    let pipeline_state = create_pipeline_with_lifecycle_events_v1(
        device,
        creation.function,
        creation.function_name,
        creation.shader_source_digest,
        lifecycle,
    )?;
    let thread_execution_width = usize::try_from(pipeline_state.thread_execution_width())
        .map_err(|_| M3MicroMetalErrorV1::MetalPipelineCreationFailed)?;
    let max_total_threads_per_threadgroup =
        usize::try_from(pipeline_state.max_total_threads_per_threadgroup())
            .map_err(|_| M3MicroMetalErrorV1::MetalPipelineCreationFailed)?;
    let mut metadata = MetalPipelineArtifactMetadataV1 {
        artifact_schema_version: METAL_PIPELINE_ARTIFACT_SCHEMA_VERSION_V1,
        function_name: creation.function_name.to_string(),
        shader_source_digest: creation.shader_source_digest.to_string(),
        library_build_policy_identity: creation.library_build_policy_identity.to_string(),
        function_lookup_policy_identity: creation.function_lookup_policy_identity.to_string(),
        function_constant_identity: creation.function_constant_identity.to_string(),
        kernel_abi_identity: creation.kernel_abi_identity.to_string(),
        parameter_buffer_layout_identity: creation.parameter_buffer_layout_identity.to_string(),
        semantic_pipeline_identity: String::new(),
        thread_execution_width,
        max_total_threads_per_threadgroup,
    };
    metadata.semantic_pipeline_identity =
        metal_pipeline_semantic_identity_from_metadata_v1(&metadata);
    #[cfg(test)]
    lifecycle.emit_actual_lifecycle_event_v2(
        MetalPipelineLifecycleEventKindV1::PipelineObservationCaptured,
        Some(&metadata.function_name),
        Some(&metadata.shader_source_digest),
        Some(&metadata.semantic_pipeline_identity),
        None,
    )?;
    let artifact = M3MicroMetalPipelineArtifactV1 {
        pipeline_state,
        metadata,
    };
    validate_pipeline_artifact_v1(&artifact)
        .map_err(|_| M3MicroMetalErrorV1::MetalPipelineCreationFailed)?;
    #[cfg(test)]
    lifecycle.emit_actual_lifecycle_event_v2(
        MetalPipelineLifecycleEventKindV1::PipelineArtifactCreated,
        Some(&artifact.metadata.function_name),
        Some(&artifact.metadata.shader_source_digest),
        Some(&artifact.metadata.semantic_pipeline_identity),
        None,
    )?;
    Ok(artifact)
}

pub struct M3MicroMetalExecutorV1 {
    device: Device,
    queue: CommandQueue,
    pipeline_artifact: M3MicroMetalPipelineArtifactV1,
    uploaded_model: Option<UploadedModelV1>,
    state_buffer: Option<metal::Buffer>,
    state_step_index: Option<usize>,
    last_output: Option<Vec<f32>>,
    last_state: Option<M3MicroState>,
    last_buffer_sizes: Option<M3MicroMetalBufferSizesV1>,
    witness: MetalExecutionWitnessV1,
    #[cfg(test)]
    fault_trace: Option<MetalFaultEvidenceV1>,
    #[cfg(test)]
    semantic_trace: Option<MetalSemanticTraceCollectorV4>,
    #[cfg(test)]
    pipeline_lifecycle_monitor: Option<MetalPipelineLifecycleMonitorV2>,
}

fn set_pipeline_with_semantic_trace_v1(
    encoder: &metal::ComputeCommandEncoderRef,
    invocation: &MetalPipelineBindingInvocationV1<'_>,
    #[cfg(test)] mut semantic_trace: Option<(&mut MetalSemanticTraceCollectorV4, usize)>,
) -> Result<(), M3MicroMetalErrorV1> {
    let _encoder_ordinal = invocation.encoder_ordinal;
    if let Err(_error) = validate_pipeline_artifact_v1(invocation.artifact) {
        #[cfg(test)]
        if let Some((collector, _)) = semantic_trace.as_mut() {
            collector.pipeline_binding_validation_failed(_error);
        }
        return Err(M3MicroMetalErrorV1::MetalEncodingFailed);
    }
    #[cfg(test)]
    let semantic_binding = match semantic_trace.as_mut() {
        Some((collector, command_ordinal)) => Some(
            collector
                .record_actual_pipeline_binding_v2(*command_ordinal, invocation)
                .map_err(|_| M3MicroMetalErrorV1::MetalEncodingFailed)?,
        ),
        None => None,
    };
    encoder.set_compute_pipeline_state(&invocation.artifact.pipeline_state);
    #[cfg(test)]
    if let (Some((collector, command_ordinal)), Some(pipeline_binding_ordinal)) =
        (semantic_trace, semantic_binding)
    {
        collector.pipeline_binding_performed(
            command_ordinal,
            invocation.encoder_ordinal,
            pipeline_binding_ordinal,
        );
    }
    Ok(())
}

fn set_buffer_with_semantic_trace_v1(
    encoder: &metal::ComputeCommandEncoderRef,
    invocation: &MetalBufferBindingInvocationV1<'_>,
    #[cfg(test)] mut semantic_trace: Option<(&mut MetalSemanticTraceCollectorV4, usize, usize)>,
) -> Result<(), M3MicroMetalErrorV1> {
    let arguments = match invocation.validated_arguments() {
        Ok(arguments) => arguments,
        Err(_error) => {
            #[cfg(test)]
            if let Some((collector, _, _)) = semantic_trace.as_mut() {
                collector.buffer_binding_validation_failed(_error);
            }
            return Err(M3MicroMetalErrorV1::MetalEncodingFailed);
        }
    };
    #[cfg(test)]
    let semantic_binding = match semantic_trace.as_mut() {
        Some((collector, command_ordinal, encoder_ordinal)) => Some(
            collector
                .record_actual_buffer_binding_v1(
                    *command_ordinal,
                    *encoder_ordinal,
                    invocation,
                    &arguments,
                )
                .map_err(|_| M3MicroMetalErrorV1::MetalEncodingFailed)?,
        ),
        None => None,
    };
    encoder.set_buffer(
        arguments.api_binding_index,
        Some(invocation.buffer),
        arguments.api_byte_offset,
    );
    #[cfg(test)]
    if let (Some((collector, command_ordinal, encoder_ordinal)), Some(binding_ordinal)) =
        (semantic_trace, semantic_binding)
    {
        collector.buffer_binding_performed(command_ordinal, encoder_ordinal, binding_ordinal);
    }
    Ok(())
}

fn checked_buffer_span_bytes_v1<T>(element_count: usize) -> Result<usize, M3MicroMetalErrorV1> {
    element_count
        .checked_mul(std::mem::size_of::<T>())
        .ok_or(M3MicroMetalErrorV1::MetalEncodingFailed)
}

fn dispatch_threads_with_semantic_trace_v1(
    encoder: &metal::ComputeCommandEncoderRef,
    invocation: &MetalDispatchInvocationV1,
    #[cfg(test)] mut semantic_trace: Option<(&mut MetalSemanticTraceCollectorV4, usize, usize)>,
) -> Result<(), M3MicroMetalErrorV1> {
    invocation
        .validated_arguments()
        .map_err(|_| M3MicroMetalErrorV1::MetalEncodingFailed)?;
    #[cfg(test)]
    let semantic_dispatch = match semantic_trace.as_mut() {
        Some((collector, command_ordinal, encoder_ordinal)) => Some(
            collector
                .record_actual_dispatch_v2(*command_ordinal, *encoder_ordinal, invocation)
                .map_err(|_| M3MicroMetalErrorV1::MetalEncodingFailed)?,
        ),
        None => None,
    };
    encoder.dispatch_threads(invocation.grid, invocation.threads_per_threadgroup);
    #[cfg(test)]
    if let (Some((collector, command_ordinal, encoder_ordinal)), Some(dispatch_ordinal)) =
        (semantic_trace, semantic_dispatch)
    {
        collector.dispatch_performed(command_ordinal, encoder_ordinal, dispatch_ordinal);
    }
    Ok(())
}

impl M3MicroMetalExecutorV1 {
    pub fn try_new() -> Result<Self, M3MicroMetalErrorV1> {
        let (executor, _) = Self::try_new_internal(MetalPipelineLifecycleCollectionV1::Disabled)?;
        Ok(executor)
    }

    fn try_new_internal(
        mut lifecycle: MetalPipelineLifecycleCollectionV1,
    ) -> Result<(Self, Option<MetalPipelineConstructionEvidenceV1>), M3MicroMetalErrorV1> {
        #[cfg(test)]
        let pipeline_lifecycle_monitor = lifecycle.monitor();
        let device = Device::system_default().ok_or(M3MicroMetalErrorV1::MetalDeviceUnavailable)?;
        let queue = device.new_command_queue();
        let pipeline_artifact = build_m3_micro_pipeline_artifact_v1(&device, &mut lifecycle)?;
        #[cfg(test)]
        let construction_evidence = lifecycle.into_evidence(
            MetalPipelineConstructionStatusV1::Succeeded,
            Some(
                pipeline_artifact
                    .metadata
                    .semantic_pipeline_identity
                    .clone(),
            ),
        );
        #[cfg(not(test))]
        let construction_evidence = None;
        Ok((
            Self {
                device,
                queue,
                pipeline_artifact,
                uploaded_model: None,
                state_buffer: None,
                state_step_index: None,
                last_output: None,
                last_state: None,
                last_buffer_sizes: None,
                witness: MetalExecutionWitnessV1 {
                    device_acquired: true,
                    queue_created: true,
                    pipeline_created: true,
                    command_error_none: true,
                    gpu_witness_valid: true,
                    ..MetalExecutionWitnessV1::default()
                },
                #[cfg(test)]
                fault_trace: None,
                #[cfg(test)]
                semantic_trace: None,
                #[cfg(test)]
                pipeline_lifecycle_monitor,
            },
            construction_evidence,
        ))
    }

    #[cfg(test)]
    pub(super) fn try_new_with_lifecycle_for_test(
        executor_construction_ordinal: usize,
    ) -> Result<(Self, MetalPipelineConstructionEvidenceV1), M3MicroMetalErrorV1> {
        let lifecycle = MetalPipelineLifecycleCollectionV1::collect(executor_construction_ordinal);
        let (executor, evidence) = Self::try_new_internal(lifecycle)?;
        Ok((
            executor,
            evidence.expect("collected pipeline construction must return lifecycle evidence"),
        ))
    }

    #[cfg(test)]
    pub(super) fn pipeline_lifecycle_monitor_enabled_for_test(&self) -> bool {
        self.pipeline_lifecycle_monitor.is_some()
    }

    #[cfg(test)]
    pub(super) fn pipeline_lifecycle_snapshot_for_test(
        &self,
    ) -> Result<MetalPipelineLifecycleSnapshotV2, String> {
        self.pipeline_lifecycle_monitor
            .as_ref()
            .ok_or_else(|| "Metal lifecycle monitor is disabled".to_string())?
            .snapshot()
    }

    #[cfg(test)]
    pub(super) fn record_pipeline_reconstruction_for_test(
        &self,
    ) -> Result<(), M3MicroMetalErrorV1> {
        let monitor = self
            .pipeline_lifecycle_monitor
            .as_ref()
            .ok_or(M3MicroMetalErrorV1::MetalEncodingFailed)?;
        let snapshot = monitor
            .snapshot()
            .map_err(|_| M3MicroMetalErrorV1::MetalEncodingFailed)?;
        let event_ordinal = usize::try_from(
            snapshot
                .event_sequence
                .checked_add(1)
                .ok_or(M3MicroMetalErrorV1::MetalEncodingFailed)?,
        )
        .map_err(|_| M3MicroMetalErrorV1::MetalEncodingFailed)?;
        monitor.record_actual_event_v2(&MetalPipelineLifecycleEventV1 {
            event_ordinal,
            executor_construction_ordinal: 0,
            kind: MetalPipelineLifecycleEventKindV1::PipelineCreationAttempted,
            function_name: Some(self.pipeline_artifact.metadata.function_name.clone()),
            shader_source_digest: Some(
                self.pipeline_artifact.metadata.shader_source_digest.clone(),
            ),
            semantic_pipeline_identity: Some(
                self.pipeline_artifact
                    .metadata
                    .semantic_pipeline_identity
                    .clone(),
            ),
            error_kind: None,
        })
    }

    pub fn validate(&self) -> Result<(), M3MicroMetalErrorV1> {
        if !self.witness.device_acquired
            || !self.witness.queue_created
            || !self.witness.pipeline_created
            || validate_pipeline_artifact_v1(&self.pipeline_artifact).is_err()
        {
            return Err(M3MicroMetalErrorV1::MetalPipelineCreationFailed);
        }
        if let Some(uploaded) = &self.uploaded_model {
            let mut layout = uploaded.layout.clone();
            layout.validate(&uploaded.config)?;
        }
        Ok(())
    }

    pub fn device_name(&self) -> String {
        self.device.name().to_string()
    }

    pub fn function_identity(&self) -> &str {
        &self.pipeline_artifact.metadata.function_name
    }

    pub fn upload_model(&mut self, model: &M3MicroModel) -> Result<(), M3MicroMetalErrorV1> {
        model
            .validate()
            .map_err(|_| M3MicroMetalErrorV1::MetalShapeMismatch)?;
        let layout = M3MicroMetalLayoutV1::from_config(&model.config)?;
        if model.parameters.values.len() != layout.total_parameter_count {
            return Err(M3MicroMetalErrorV1::MetalShapeMismatch);
        }
        let parameters = shared_buffer(&self.device, &model.parameters.values)?;
        self.uploaded_model = Some(UploadedModelV1 {
            config: model.config.clone(),
            layout,
            parameters,
        });
        self.state_buffer = None;
        self.state_step_index = None;
        self.last_output = None;
        self.last_state = None;
        Ok(())
    }

    pub fn upload_state(&mut self, state: &M3MicroState) -> Result<(), M3MicroMetalErrorV1> {
        let uploaded = self
            .uploaded_model
            .as_ref()
            .ok_or(M3MicroMetalErrorV1::MetalShapeMismatch)?;
        state
            .validate(&uploaded.config)
            .map_err(|_| M3MicroMetalErrorV1::MetalShapeMismatch)?;
        let flattened = flatten_state(state);
        if flattened.len() != uploaded.layout.state_width {
            return Err(M3MicroMetalErrorV1::MetalShapeMismatch);
        }
        self.state_buffer = Some(shared_buffer(&self.device, &flattened)?);
        self.state_step_index = Some(state.step_index);
        self.last_state = Some(state.clone());
        self.last_output = None;
        Ok(())
    }

    pub fn forward_step(&mut self, input: &[f32]) -> Result<Vec<f32>, M3MicroMetalErrorV1> {
        self.forward_sequence(&[input.to_vec()])
    }

    pub fn forward_sequence(
        &mut self,
        sequence: &[Vec<f32>],
    ) -> Result<Vec<f32>, M3MicroMetalErrorV1> {
        self.execute_internal(sequence, MetalExecutionControlV1::Normal)
    }

    fn execute_internal(
        &mut self,
        sequence: &[Vec<f32>],
        control: MetalExecutionControlV1,
    ) -> Result<Vec<f32>, M3MicroMetalErrorV1> {
        #[cfg(not(test))]
        let _ = control;
        let uploaded = self
            .uploaded_model
            .as_ref()
            .ok_or(M3MicroMetalErrorV1::MetalShapeMismatch)?;
        if sequence.is_empty()
            || sequence
                .iter()
                .any(|row| row.len() != uploaded.config.input_dim)
        {
            return Err(M3MicroMetalErrorV1::MetalShapeMismatch);
        }
        if sequence.iter().flatten().any(|value| !value.is_finite()) {
            return Err(M3MicroMetalErrorV1::MetalNonFiniteInput);
        }
        let step_index = self
            .state_step_index
            .ok_or(M3MicroMetalErrorV1::MetalShapeMismatch)?;
        let expected_step = step_index
            .checked_add(sequence.len())
            .ok_or(M3MicroMetalErrorV1::MetalShapeMismatch)?;
        let config = uploaded.config.clone();
        let layout = uploaded.layout.clone();
        let parameter_buffer = uploaded.parameters.clone();
        let initial_state = self
            .state_buffer
            .as_ref()
            .ok_or(M3MicroMetalErrorV1::MetalShapeMismatch)?
            .clone();
        let input_values = sequence.iter().flatten().copied().collect::<Vec<_>>();
        let input_buffer = shared_buffer(&self.device, &input_values)?;
        let metadata_values = layout.metadata(&config, sequence.len(), step_index)?;
        let metadata_buffer = shared_buffer(&self.device, &metadata_values)?;
        let scalar_buffer = shared_buffer(&self.device, &[config.decay_min, config.decay_max])?;
        let poison = f32::from_bits(METAL_POISON_BITS_V1);
        let output_buffer = shared_buffer(&self.device, &vec![poison; config.output_dim])?;
        let final_state_buffer = shared_buffer(&self.device, &vec![poison; layout.state_width])?;
        let witness_buffer = shared_buffer(
            &self.device,
            &vec![METAL_POISON_BITS_V1; METAL_WITNESS_WORDS_V1],
        )?;
        #[cfg(test)]
        let output_sink = (control == MetalExecutionControlV1::SuppressOutputWrite)
            .then(|| shared_buffer(&self.device, &vec![poison; config.output_dim]))
            .transpose()?;
        #[cfg(test)]
        let state_sink = (control == MetalExecutionControlV1::SuppressStateWrite)
            .then(|| shared_buffer(&self.device, &vec![poison; layout.state_width]))
            .transpose()?;
        #[cfg(test)]
        let output_write_buffer = output_sink.as_ref().unwrap_or(&output_buffer);
        #[cfg(not(test))]
        let output_write_buffer = &output_buffer;
        #[cfg(test)]
        let state_write_buffer = state_sink.as_ref().unwrap_or(&final_state_buffer);
        #[cfg(not(test))]
        let state_write_buffer = &final_state_buffer;

        #[cfg(test)]
        if let Some(trace) = &mut self.fault_trace {
            trace.buffers_created = true;
        }

        self.witness.command_buffer_count += 1;
        let command = self.queue.new_command_buffer();
        #[cfg(test)]
        let semantic_command_ordinal = self
            .semantic_trace
            .as_mut()
            .map(|collector| collector.command_created(sequence.len()))
            .transpose()
            .map_err(|_| M3MicroMetalErrorV1::MetalEncodingFailed)?;
        let encoder = command.new_compute_command_encoder();
        self.witness.compute_encoder_count += 1;
        #[cfg(test)]
        let semantic_encoder_ordinal = semantic_command_ordinal.map(|command_ordinal| {
            self.semantic_trace
                .as_mut()
                .expect("semantic trace collector remains active")
                .encoder_created(command_ordinal)
        });
        #[cfg(test)]
        if let Some(trace) = &mut self.fault_trace {
            trace.encoder_created = true;
        }
        #[cfg(test)]
        let pipeline_encoder_ordinal = semantic_encoder_ordinal.unwrap_or(0);
        #[cfg(not(test))]
        let pipeline_encoder_ordinal = 0;
        let pipeline_invocation = MetalPipelineBindingInvocationV1::new(
            &self.pipeline_artifact,
            pipeline_encoder_ordinal,
        );
        #[cfg(test)]
        let pipeline_semantic_trace = match (self.semantic_trace.as_mut(), semantic_command_ordinal)
        {
            (Some(collector), Some(command_ordinal)) => Some((collector, command_ordinal)),
            (None, None) => None,
            _ => return Err(M3MicroMetalErrorV1::MetalEncodingFailed),
        };
        set_pipeline_with_semantic_trace_v1(
            encoder,
            &pipeline_invocation,
            #[cfg(test)]
            pipeline_semantic_trace,
        )?;
        let binding_invocations = [
            MetalBufferBindingInvocationV1::new(
                &input_buffer,
                0,
                0,
                MetalBufferRoleV2::Input,
                MetalBufferAccessV2::ReadOnly,
                checked_buffer_span_bytes_v1::<f32>(input_values.len())?,
            ),
            MetalBufferBindingInvocationV1::new(
                &parameter_buffer,
                1,
                0,
                MetalBufferRoleV2::Parameters,
                MetalBufferAccessV2::ReadOnly,
                checked_buffer_span_bytes_v1::<f32>(layout.total_parameter_count)?,
            ),
            MetalBufferBindingInvocationV1::new(
                &initial_state,
                2,
                0,
                MetalBufferRoleV2::InitialState,
                MetalBufferAccessV2::ReadOnly,
                checked_buffer_span_bytes_v1::<f32>(layout.state_width)?,
            ),
            MetalBufferBindingInvocationV1::new(
                output_write_buffer,
                3,
                0,
                MetalBufferRoleV2::Output,
                MetalBufferAccessV2::WriteOnly,
                checked_buffer_span_bytes_v1::<f32>(config.output_dim)?,
            ),
            MetalBufferBindingInvocationV1::new(
                state_write_buffer,
                4,
                0,
                MetalBufferRoleV2::FinalState,
                MetalBufferAccessV2::WriteOnly,
                checked_buffer_span_bytes_v1::<f32>(layout.state_width)?,
            ),
            MetalBufferBindingInvocationV1::new(
                &metadata_buffer,
                5,
                0,
                MetalBufferRoleV2::Metadata,
                MetalBufferAccessV2::ReadOnly,
                checked_buffer_span_bytes_v1::<u32>(metadata_values.len())?,
            ),
            MetalBufferBindingInvocationV1::new(
                &scalar_buffer,
                6,
                0,
                MetalBufferRoleV2::ScalarConfiguration,
                MetalBufferAccessV2::ReadOnly,
                checked_buffer_span_bytes_v1::<f32>(2)?,
            ),
            MetalBufferBindingInvocationV1::new(
                &witness_buffer,
                7,
                0,
                MetalBufferRoleV2::ExecutionWitness,
                MetalBufferAccessV2::WriteOnly,
                checked_buffer_span_bytes_v1::<u32>(METAL_WITNESS_WORDS_V1)?,
            ),
        ];
        for invocation in &binding_invocations {
            #[cfg(test)]
            let semantic_trace = match (
                self.semantic_trace.as_mut(),
                semantic_command_ordinal.zip(semantic_encoder_ordinal),
            ) {
                (Some(collector), Some((command_ordinal, encoder_ordinal))) => {
                    Some((collector, command_ordinal, encoder_ordinal))
                }
                (None, None) => None,
                _ => return Err(M3MicroMetalErrorV1::MetalEncodingFailed),
            };
            set_buffer_with_semantic_trace_v1(
                encoder,
                invocation,
                #[cfg(test)]
                semantic_trace,
            )?;
        }
        self.witness.dispatches_attempted += 1;
        #[cfg(test)]
        if let Some(trace) = &mut self.fault_trace {
            trace.dispatch_attempted = true;
        }
        #[cfg(test)]
        let skip_dispatch = control == MetalExecutionControlV1::SkipDispatch;
        #[cfg(not(test))]
        let skip_dispatch = false;
        if !skip_dispatch {
            #[cfg(test)]
            let dispatch_context = self
                .semantic_trace
                .as_ref()
                .and_then(|collector| collector.pending_segment)
                .map(|segment| (segment.chunk_start, segment.chunk_ordinal))
                .unwrap_or((step_index, 0));
            #[cfg(not(test))]
            let dispatch_context = (step_index, 0);
            let invocation = MetalDispatchInvocationV1::new(
                MTLSize::new(1, 1, 1),
                MTLSize::new(1, 1, 1),
                dispatch_context.0,
                dispatch_context.1,
            );
            #[cfg(test)]
            let semantic_trace = match (
                self.semantic_trace.as_mut(),
                semantic_command_ordinal.zip(semantic_encoder_ordinal),
            ) {
                (Some(collector), Some((command_ordinal, encoder_ordinal))) => {
                    Some((collector, command_ordinal, encoder_ordinal))
                }
                (None, None) => None,
                _ => return Err(M3MicroMetalErrorV1::MetalEncodingFailed),
            };
            dispatch_threads_with_semantic_trace_v1(
                encoder,
                &invocation,
                #[cfg(test)]
                semantic_trace,
            )?;
            self.witness.dispatch_count += 1;
            #[cfg(test)]
            if let Some(trace) = &mut self.fault_trace {
                trace.dispatch_performed = true;
            }
        }
        encoder.end_encoding();
        #[cfg(test)]
        if let (Some(command_ordinal), Some(encoder_ordinal)) =
            (semantic_command_ordinal, semantic_encoder_ordinal)
        {
            self.semantic_trace
                .as_mut()
                .expect("semantic trace collector remains active")
                .encoder_ended(command_ordinal, encoder_ordinal);
        }
        command.commit();
        self.witness.command_buffers_committed += 1;
        #[cfg(test)]
        if let Some(command_ordinal) = semantic_command_ordinal {
            self.semantic_trace
                .as_mut()
                .expect("semantic trace collector remains active")
                .command_committed(command_ordinal);
        }
        #[cfg(test)]
        if let Some(trace) = &mut self.fault_trace {
            trace.command_buffer_committed = true;
        }
        command.wait_until_completed();

        let observation = observe_command_completion_v1(command, true);
        #[cfg(test)]
        if let Some(command_ordinal) = semantic_command_ordinal {
            self.semantic_trace
                .as_mut()
                .expect("semantic trace collector remains active")
                .command_completed(command_ordinal, &observation);
        }
        #[cfg(test)]
        if let Some(trace) = &mut self.fault_trace {
            trace.command_buffer_completed =
                observation.terminal_status == MetalCommandTerminalStatusV1::Completed;
            trace.command_buffer_failed =
                observation.terminal_status == MetalCommandTerminalStatusV1::Error;
            trace.command_status = format!("{:?}", observation.terminal_status).to_uppercase();
            trace.command_error_present = observation.error_present;
        }
        self.witness.command_error_none &= !observation.error_present;
        if let Err(error) = handle_command_completion_without_cpu_fallback_v1(
            &observation,
            &mut self.witness.cpu_fallback_count,
        ) {
            self.witness.command_buffer_failures += 1;
            return Err(error);
        }
        self.witness.command_buffers_completed += 1;

        let output = read_buffer::<f32>(&output_buffer, config.output_dim)?;
        self.witness.output_readback_count += 1;
        #[cfg(test)]
        if let Some(command_ordinal) = semantic_command_ordinal {
            self.semantic_trace
                .as_mut()
                .expect("semantic trace collector remains active")
                .output_readback(command_ordinal);
        }
        let state_values = read_buffer::<f32>(&final_state_buffer, layout.state_width)?;
        self.witness.state_readback_count += 1;
        #[cfg(test)]
        if let Some(command_ordinal) = semantic_command_ordinal {
            self.semantic_trace
                .as_mut()
                .expect("semantic trace collector remains active")
                .state_readback(command_ordinal);
        }
        let gpu_witness = read_buffer::<u32>(&witness_buffer, METAL_WITNESS_WORDS_V1)?;
        let output_poison = output
            .iter()
            .filter(|value| value.to_bits() == METAL_POISON_BITS_V1)
            .count();
        let state_poison = state_values
            .iter()
            .filter(|value| value.to_bits() == METAL_POISON_BITS_V1)
            .count();
        self.witness.output_poison_remaining += output_poison;
        self.witness.state_poison_remaining += state_poison;
        #[cfg(test)]
        if let Some(trace) = &mut self.fault_trace {
            trace.output_poison_remaining = output_poison;
            trace.state_poison_remaining = state_poison;
        }
        if output_poison != 0 {
            return Err(M3MicroMetalErrorV1::MetalOutputNotWritten);
        }
        if state_poison != 0 {
            return Err(M3MicroMetalErrorV1::MetalStateNotWritten);
        }
        let gpu_witness_valid = gpu_witness
            == [
                METAL_GPU_WITNESS_MAGIC_V1,
                config.output_dim as u32,
                layout.state_width as u32,
                sequence.len() as u32,
                expected_step as u32,
                0,
            ];
        self.witness.gpu_witness_valid &= gpu_witness_valid;
        if !gpu_witness_valid {
            return Err(M3MicroMetalErrorV1::MetalEncodingFailed);
        }
        if output
            .iter()
            .chain(&state_values)
            .any(|value| !value.is_finite())
        {
            return Err(M3MicroMetalErrorV1::MetalNonFiniteOutput);
        }
        let state = unflatten_state(&config, &state_values, expected_step)?;
        state
            .validate(&config)
            .map_err(|_| M3MicroMetalErrorV1::MetalNonFiniteOutput)?;
        #[cfg(test)]
        if let Some(command_ordinal) = semantic_command_ordinal {
            self.semantic_trace
                .as_mut()
                .expect("semantic trace collector remains active")
                .segment_completed(command_ordinal, state_bits_digest(&state));
        }
        self.witness.output_digest = float_bits_digest(&output);
        self.witness.final_state_digest = state_bits_digest(&state);
        self.last_buffer_sizes = Some(M3MicroMetalBufferSizesV1 {
            parameter_bytes: layout.total_parameter_count * std::mem::size_of::<f32>(),
            input_bytes: input_values.len() * std::mem::size_of::<f32>(),
            initial_state_bytes: layout.state_width * std::mem::size_of::<f32>(),
            output_bytes: config.output_dim * std::mem::size_of::<f32>(),
            final_state_bytes: layout.state_width * std::mem::size_of::<f32>(),
            metadata_bytes: metadata_values.len() * std::mem::size_of::<u32>(),
            witness_bytes: METAL_WITNESS_WORDS_V1 * std::mem::size_of::<u32>(),
        });
        self.state_buffer = Some(final_state_buffer);
        self.state_step_index = Some(expected_step);
        self.last_output = Some(output.clone());
        self.last_state = Some(state);
        Ok(output)
    }

    pub fn read_output(&self) -> Result<&[f32], M3MicroMetalErrorV1> {
        self.last_output
            .as_deref()
            .ok_or(M3MicroMetalErrorV1::MetalOutputNotWritten)
    }

    pub fn read_state(&self) -> Result<&M3MicroState, M3MicroMetalErrorV1> {
        self.last_state
            .as_ref()
            .ok_or(M3MicroMetalErrorV1::MetalStateNotWritten)
    }

    pub fn execution_witness(&self) -> &MetalExecutionWitnessV1 {
        &self.witness
    }

    #[cfg(test)]
    pub(super) fn semantic_trace_collection_enabled_for_test(&self) -> bool {
        self.semantic_trace.is_some()
    }

    #[cfg(test)]
    pub(super) fn begin_semantic_trace_for_test(
        &mut self,
        execution_mode: MetalExecutionModeV1,
        agent_role: impl Into<String>,
        sequence_length: usize,
        chunk_policy_identity: impl Into<String>,
    ) -> Result<(), String> {
        if self.semantic_trace.is_some() || sequence_length == 0 {
            return Err(
                "semantic trace collection boundary is already active or empty".to_string(),
            );
        }
        self.semantic_trace = Some(MetalSemanticTraceCollectorV4::new(
            execution_mode,
            agent_role.into(),
            sequence_length,
            chunk_policy_identity.into(),
            self.witness.clone(),
        ));
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn set_semantic_trace_segment_for_test(
        &mut self,
        chunk_ordinal: usize,
        chunk_start: usize,
        chunk_length: usize,
    ) -> Result<(), String> {
        let collector = self
            .semantic_trace
            .as_mut()
            .ok_or_else(|| "semantic trace collection is disabled".to_string())?;
        if collector.pending_segment.is_some() || chunk_length == 0 {
            return Err("semantic trace segment is already pending or empty".to_string());
        }
        collector.set_segment(chunk_ordinal, chunk_start, chunk_length);
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn finish_semantic_trace_for_test(
        &mut self,
    ) -> Result<MetalSemanticRunTraceV1, String> {
        let mut collector = self
            .semantic_trace
            .take()
            .ok_or_else(|| "semantic trace collection is disabled".to_string())?;
        if collector.pending_segment.is_some() {
            return Err("semantic trace segment did not complete".to_string());
        }
        let output = self
            .last_output
            .as_ref()
            .ok_or_else(|| "semantic trace output is missing".to_string())?;
        let state = self
            .last_state
            .as_ref()
            .ok_or_else(|| "semantic trace final state is missing".to_string())?;
        collector.trace.per_run_witness_delta =
            MetalPerRunWitnessDeltaV1::between(&collector.witness_before, &self.witness)?;
        collector.trace.output_digest = float_bits_digest(output);
        collector.trace.final_state_digest = state_bits_digest(state);
        collector.finalize_pipeline_binding_provenance();
        collector.finalize_buffer_binding_provenance();
        collector.trace.semantic_trace_digest = metal_semantic_trace_digest_v1(&collector.trace)?;
        validate_metal_semantic_run_trace_v1(&collector.trace)?;
        Ok(collector.trace)
    }

    #[cfg(test)]
    pub(super) fn pipeline_artifact_metadata_for_test(&self) -> &MetalPipelineArtifactMetadataV1 {
        &self.pipeline_artifact.metadata
    }

    #[cfg(test)]
    pub(super) fn validate_pipeline_artifact_for_test(&self) -> Result<(), String> {
        validate_pipeline_artifact_v1(&self.pipeline_artifact).map_err(|error| format!("{error:?}"))
    }

    pub fn buffer_sizes(&self) -> Result<&M3MicroMetalBufferSizesV1, M3MicroMetalErrorV1> {
        self.last_buffer_sizes
            .as_ref()
            .ok_or(M3MicroMetalErrorV1::MetalOutputNotWritten)
    }

    pub fn layout(&self) -> Result<&M3MicroMetalLayoutV1, M3MicroMetalErrorV1> {
        self.uploaded_model
            .as_ref()
            .map(|uploaded| &uploaded.layout)
            .ok_or(M3MicroMetalErrorV1::MetalShapeMismatch)
    }

    #[cfg(test)]
    pub(super) fn execute_required_fault_for_test(
        &mut self,
        sequence: &[Vec<f32>],
        injection: MetalRequiredFaultInjectionV2,
    ) -> MetalFaultEvidenceV1 {
        let fault = injection.identity();
        if injection == MetalRequiredFaultInjectionV2::MissingKernelFunction {
            return Self::missing_kernel_function_fault_for_test();
        }

        let mut trace = MetalFaultEvidenceV1::new(fault);
        trace.device_acquired = true;
        trace.queue_created = true;
        trace.library_created = true;
        trace.function_lookup_attempted = true;
        trace.function_lookup_succeeded = true;
        trace.pipeline_created = true;
        self.fault_trace = Some(trace);
        let control = match injection {
            MetalRequiredFaultInjectionV2::SkipDispatch
            | MetalRequiredFaultInjectionV2::AttemptCpuFallback => {
                MetalExecutionControlV1::SkipDispatch
            }
            MetalRequiredFaultInjectionV2::SuppressOutputWrite => {
                MetalExecutionControlV1::SuppressOutputWrite
            }
            MetalRequiredFaultInjectionV2::SuppressStateWrite => {
                MetalExecutionControlV1::SuppressStateWrite
            }
            MetalRequiredFaultInjectionV2::MissingKernelFunction => unreachable!(),
        };
        let result = self.execute_internal(sequence, control);
        let mut trace = self
            .fault_trace
            .take()
            .expect("test fault execution must retain its trace");
        let mut observed_error = result.err();
        if injection == MetalRequiredFaultInjectionV2::AttemptCpuFallback {
            trace.trigger_error = observed_error;
            observed_error = Some(refuse_cpu_fallback_after_metal_error_for_test(
                trace
                    .trigger_error
                    .expect("fallback seam requires a preceding Metal failure"),
                &mut trace,
            ));
        }
        trace.finalize(observed_error);
        trace
    }

    #[cfg(test)]
    fn missing_kernel_function_fault_for_test() -> MetalFaultEvidenceV1 {
        let mut trace = MetalFaultEvidenceV1::new(MetalFaultIdentityV2::MissingKernelFunction);
        let Some(device) = Device::system_default() else {
            trace.finalize(Some(M3MicroMetalErrorV1::MetalDeviceUnavailable));
            return trace;
        };
        trace.device_acquired = true;
        let _queue = device.new_command_queue();
        trace.queue_created = true;
        let options = CompileOptions::new();
        options.set_fast_math_enabled(false);
        let shader_source_digest = stable_hash_string(M3_MICRO_METAL_SHADER_V1);
        let mut lifecycle = MetalPipelineLifecycleCollectionV1::Disabled;
        let library = match compile_library_with_lifecycle_events_v1(
            &device,
            M3_MICRO_METAL_SHADER_V1,
            &shader_source_digest,
            &options,
            &mut lifecycle,
        ) {
            Ok(library) => library,
            Err(_) => {
                trace.finalize(Some(M3MicroMetalErrorV1::MetalLibraryCreationFailed));
                return trace;
            }
        };
        trace.library_created = true;
        trace.function_lookup_attempted = true;
        let observed_error = match lookup_function_with_lifecycle_events_v1(
            &library,
            "m3_micro_missing_function",
            &shader_source_digest,
            &mut lifecycle,
        ) {
            Ok(_) => {
                trace.function_lookup_succeeded = true;
                None
            }
            Err(_) => Some(M3MicroMetalErrorV1::MetalFunctionNotFound),
        };
        trace.finalize(observed_error);
        trace
    }

    #[cfg(test)]
    pub(super) fn missing_function_lifecycle_for_test(
        executor_construction_ordinal: usize,
    ) -> Result<MetalPipelineConstructionEvidenceV1, M3MicroMetalErrorV1> {
        let device = Device::system_default().ok_or(M3MicroMetalErrorV1::MetalDeviceUnavailable)?;
        let options = CompileOptions::new();
        options.set_fast_math_enabled(false);
        let shader_source_digest = stable_hash_string(M3_MICRO_METAL_SHADER_V1);
        let mut lifecycle =
            MetalPipelineLifecycleCollectionV1::collect(executor_construction_ordinal);
        let library = compile_library_with_lifecycle_events_v1(
            &device,
            M3_MICRO_METAL_SHADER_V1,
            &shader_source_digest,
            &options,
            &mut lifecycle,
        )?;
        let error = lookup_function_with_lifecycle_events_v1(
            &library,
            "m3_micro_missing_function",
            &shader_source_digest,
            &mut lifecycle,
        )
        .expect_err("missing test function must produce an actual Metal lookup error");
        Ok(lifecycle
            .into_evidence(MetalPipelineConstructionStatusV1::Failed, None)
            .expect("collected failure must return lifecycle evidence with actual error")
            .with_error_consistency(error))
    }
}

#[cfg(test)]
impl MetalPipelineConstructionEvidenceV1 {
    fn with_error_consistency(self, error: M3MicroMetalErrorV1) -> Self {
        debug_assert_eq!(
            self.events.last().and_then(|event| event.error_kind),
            Some(error)
        );
        self
    }
}

#[cfg(test)]
fn refuse_cpu_fallback_after_metal_error_for_test(
    trigger: M3MicroMetalErrorV1,
    trace: &mut MetalFaultEvidenceV1,
) -> M3MicroMetalErrorV1 {
    debug_assert_ne!(trigger, M3MicroMetalErrorV1::MetalCpuFallbackForbidden);
    trace.fallback_decision_seam_reached = true;
    trace.cpu_fallback_attempts += 1;
    trace.cpu_fallback_executions = 0;
    M3MicroMetalErrorV1::MetalCpuFallbackForbidden
}

pub fn m3_micro_metal_shader_digest_v1() -> String {
    stable_hash_string(M3_MICRO_METAL_SHADER_V1)
}

pub fn m3_micro_metal_executor_source_digest_v1() -> String {
    stable_hash_string(include_str!("m3_micro_metal.rs"))
}

pub fn m3_micro_metal_function_identity_v1() -> &'static str {
    METAL_FUNCTION_IDENTITY_V1
}

pub fn m3_micro_metal_state_values_v1(state: &M3MicroState) -> Vec<f32> {
    flatten_state(state)
}

pub fn m3_micro_metal_state_bits_digest_v1(state: &M3MicroState) -> String {
    state_bits_digest(state)
}

pub fn m3_micro_metal_output_bits_digest_v1(output: &[f32]) -> String {
    float_bits_digest(output)
}

fn flatten_state(state: &M3MicroState) -> Vec<f32> {
    state
        .blocks
        .iter()
        .flat_map(|block| block.values.iter().chain(&block.previous_u))
        .copied()
        .collect()
}

fn unflatten_state(
    config: &M3MicroConfig,
    values: &[f32],
    step_index: usize,
) -> Result<M3MicroState, M3MicroMetalErrorV1> {
    let inner = config.inner_dim();
    let block_width = inner * config.d_state + inner;
    if values.len() != block_width * config.block_count {
        return Err(M3MicroMetalErrorV1::MetalShapeMismatch);
    }
    let blocks = values
        .chunks_exact(block_width)
        .map(|block| M3MicroBlockState {
            values: block[..inner * config.d_state].to_vec(),
            previous_u: block[inner * config.d_state..].to_vec(),
        })
        .collect();
    Ok(M3MicroState { blocks, step_index })
}

fn float_bits_digest(values: &[f32]) -> String {
    stable_hash_string(&format!(
        "{:?}",
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    ))
}

fn state_bits_digest(state: &M3MicroState) -> String {
    stable_hash_string(&format!(
        "{}:{:?}",
        state.step_index,
        flatten_state(state)
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    ))
}

fn shared_buffer<T: Copy>(
    device: &Device,
    values: &[T],
) -> Result<metal::Buffer, M3MicroMetalErrorV1> {
    if values.is_empty() {
        return Err(M3MicroMetalErrorV1::MetalBufferAllocationFailed);
    }
    let byte_len = values
        .len()
        .checked_mul(std::mem::size_of::<T>())
        .and_then(|value| u64::try_from(value).ok())
        .ok_or(M3MicroMetalErrorV1::MetalBufferAllocationFailed)?;
    let buffer = device.new_buffer(byte_len, MTLResourceOptions::StorageModeShared);
    if buffer.contents().is_null() {
        return Err(M3MicroMetalErrorV1::MetalBufferAllocationFailed);
    }
    // StorageModeShared is CPU-visible and the allocation exactly matches `values`.
    unsafe {
        std::ptr::copy_nonoverlapping(values.as_ptr(), buffer.contents().cast::<T>(), values.len());
    }
    Ok(buffer)
}

fn read_buffer<T: Copy>(buffer: &metal::Buffer, len: usize) -> Result<Vec<T>, M3MicroMetalErrorV1> {
    if len == 0 || buffer.contents().is_null() {
        return Err(M3MicroMetalErrorV1::MetalBufferAllocationFailed);
    }
    // The command buffer is complete and the shared allocation contains `len` T values.
    Ok(unsafe { std::slice::from_raw_parts(buffer.contents().cast::<T>(), len) }.to_vec())
}

fn observe_command_completion_v1(
    command: &metal::CommandBufferRef,
    command_committed: bool,
) -> MetalCommandCompletionObservationV1 {
    let status = command.status();
    let terminal_status = if status == MTLCommandBufferStatus::Completed {
        MetalCommandTerminalStatusV1::Completed
    } else if status == MTLCommandBufferStatus::Error {
        MetalCommandTerminalStatusV1::Error
    } else {
        MetalCommandTerminalStatusV1::Other
    };
    let (error_present, error_domain, error_code) = command_buffer_error_details_v1(command);
    MetalCommandCompletionObservationV1 {
        terminal_status,
        error_present,
        error_domain,
        error_code,
        command_committed,
        command_completed_or_failed: matches!(
            terminal_status,
            MetalCommandTerminalStatusV1::Completed | MetalCommandTerminalStatusV1::Error
        ),
    }
}

fn command_buffer_error_details_v1(
    command: &metal::CommandBufferRef,
) -> (bool, Option<String>, Option<i64>) {
    use metal::objc::{
        Message,
        runtime::{Object, Sel},
    };
    use std::ffi::CStr;
    use std::os::raw::c_char;

    // MTLCommandBuffer.error is nil on success. metal-rs 0.33 exposes status but not this property.
    let error: Result<*mut Object, _> = unsafe { command.send_message(Sel::register("error"), ()) };
    let Ok(error) = error else {
        return (true, None, None);
    };
    if error.is_null() {
        return (false, None, None);
    }
    let error = unsafe { &*error };
    let domain = unsafe {
        let domain: Result<*mut Object, _> = error.send_message(Sel::register("domain"), ());
        domain
            .ok()
            .filter(|value| !value.is_null())
            .and_then(|value| {
                let utf8: Result<*const c_char, _> =
                    (&*value).send_message(Sel::register("UTF8String"), ());
                utf8.ok()
            })
            .filter(|value| !value.is_null())
            .map(|value| CStr::from_ptr(value).to_string_lossy().into_owned())
    };
    let code = unsafe {
        let code: Result<isize, _> = error.send_message(Sel::register("code"), ());
        code.ok().and_then(|value| i64::try_from(value).ok())
    };
    (true, domain, code)
}
