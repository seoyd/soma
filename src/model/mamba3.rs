use super::tiny_tensor::{
    TINY_TENSOR_MAX_ELEMENTS, TinyTensor1D, TinyTensor2D, deterministic_tiny_matrix,
    deterministic_tiny_value, from_vec_1d, tiny_tensor_memory_ok, zeros_1d, zeros_2d,
};
use serde::{Deserialize, Serialize};

// This is intentionally separate from the older paper-only temporal cell above.  It keeps the
// small tensor storage and deterministic initialization conventions, while following the Mamba-3
// SISO reference recurrence without attaching it to the committee or execution paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3SisoRopeFractionV0 {
    Half,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3SisoPrecisionV0 {
    F32,
    F64Unsupported,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3SisoConfigV0 {
    pub input_dim: usize,
    pub state_dim: usize,
    pub head_dim: usize,
    pub expansion: usize,
    pub rope_fraction: Mamba3SisoRopeFractionV0,
    pub norm_epsilon: f32,
    pub a_floor: f32,
    pub mimo_rank: usize,
    pub precision: Mamba3SisoPrecisionV0,
    #[serde(default)]
    pub short_convolution_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mamba3SisoErrorV0 {
    InvalidConfiguration,
    UnsupportedMimo,
    UnsupportedPrecision,
    UnsupportedShortConvolution,
    TensorShape,
    ParameterShape,
    StateShape,
    NonFiniteValue,
    EmptyInput,
    SequenceTooLong,
    Overflow,
    FixtureFormat,
    FixtureDigest,
}

impl std::fmt::Display for Mamba3SisoErrorV0 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidConfiguration => "invalid Mamba-3 SISO configuration",
            Self::UnsupportedMimo => "Mamba-3 SISO supports only mimo_rank=1",
            Self::UnsupportedPrecision => "Mamba-3 SISO supports only portable f32 precision",
            Self::UnsupportedShortConvolution => "Mamba-3 SISO does not support short convolution",
            Self::TensorShape => "Mamba-3 SISO tensor shape is invalid",
            Self::ParameterShape => "Mamba-3 SISO parameter shape is invalid",
            Self::StateShape => "Mamba-3 SISO state shape is invalid",
            Self::NonFiniteValue => "Mamba-3 SISO rejects NaN and infinite values",
            Self::EmptyInput => "Mamba-3 SISO step requires one input vector",
            Self::SequenceTooLong => "Mamba-3 SISO sequence exceeds tiny-core capacity",
            Self::Overflow => "Mamba-3 SISO dimension calculation overflowed",
            Self::FixtureFormat => "Mamba-3 SISO reference fixture is invalid",
            Self::FixtureDigest => "Mamba-3 SISO reference fixture digest is invalid",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for Mamba3SisoErrorV0 {}

impl Mamba3SisoConfigV0 {
    pub fn inner_dim(&self) -> Result<usize, Mamba3SisoErrorV0> {
        self.input_dim
            .checked_mul(self.expansion)
            .ok_or(Mamba3SisoErrorV0::Overflow)
    }

    pub fn head_count(&self) -> Result<usize, Mamba3SisoErrorV0> {
        let inner_dim = self.inner_dim()?;
        if self.head_dim == 0 || inner_dim % self.head_dim != 0 {
            return Err(Mamba3SisoErrorV0::InvalidConfiguration);
        }
        Ok(inner_dim / self.head_dim)
    }

    pub fn rope_angle_count(&self) -> Result<usize, Mamba3SisoErrorV0> {
        let fraction_dim = match self.rope_fraction {
            Mamba3SisoRopeFractionV0::Half => self.state_dim / 2,
            Mamba3SisoRopeFractionV0::Full => self.state_dim,
        };
        let rotary_dim = fraction_dim - (fraction_dim % 2);
        if rotary_dim == 0 {
            return Err(Mamba3SisoErrorV0::InvalidConfiguration);
        }
        Ok(rotary_dim / 2)
    }

    pub fn input_projection_rows(&self) -> Result<usize, Mamba3SisoErrorV0> {
        let inner_dim = self.inner_dim()?;
        let head_count = self.head_count()?;
        let rope_angles = self.rope_angle_count()?;
        inner_dim
            .checked_mul(2)
            .and_then(|value| value.checked_add(self.state_dim.checked_mul(2)?))
            .and_then(|value| value.checked_add(head_count.checked_mul(3)?))
            .and_then(|value| value.checked_add(rope_angles))
            .ok_or(Mamba3SisoErrorV0::Overflow)
    }

    pub fn validate(&self) -> Result<(), Mamba3SisoErrorV0> {
        if self.input_dim == 0
            || self.state_dim == 0
            || self.head_dim == 0
            || self.expansion == 0
            || !self.norm_epsilon.is_finite()
            || self.norm_epsilon <= 0.0
            || !self.a_floor.is_finite()
            || self.a_floor <= 0.0
        {
            return Err(Mamba3SisoErrorV0::InvalidConfiguration);
        }
        if self.mimo_rank != 1 {
            return Err(Mamba3SisoErrorV0::UnsupportedMimo);
        }
        if self.precision != Mamba3SisoPrecisionV0::F32 {
            return Err(Mamba3SisoErrorV0::UnsupportedPrecision);
        }
        if self.short_convolution_enabled {
            return Err(Mamba3SisoErrorV0::UnsupportedShortConvolution);
        }
        let inner_dim = self.inner_dim()?;
        let head_count = self.head_count()?;
        let rope_angles = self.rope_angle_count()?;
        let projection_elements = self
            .input_projection_rows()?
            .checked_mul(self.input_dim)
            .ok_or(Mamba3SisoErrorV0::Overflow)?;
        let state_elements = inner_dim
            .checked_mul(self.state_dim)
            .ok_or(Mamba3SisoErrorV0::Overflow)?;
        let angle_elements = head_count
            .checked_mul(rope_angles)
            .ok_or(Mamba3SisoErrorV0::Overflow)?;
        if !tiny_tensor_memory_ok(projection_elements)
            || !tiny_tensor_memory_ok(state_elements)
            || !tiny_tensor_memory_ok(angle_elements)
        {
            return Err(Mamba3SisoErrorV0::SequenceTooLong);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3SisoParamsV0 {
    pub input_projection: TinyTensor2D,
    pub dt_bias: TinyTensor1D,
    pub b_bias: TinyTensor2D,
    pub c_bias: TinyTensor2D,
    pub b_norm_scale: TinyTensor1D,
    pub c_norm_scale: TinyTensor1D,
    pub skip: TinyTensor1D,
    pub output_projection: TinyTensor2D,
}

impl Mamba3SisoParamsV0 {
    pub fn validate(&self, config: &Mamba3SisoConfigV0) -> Result<(), Mamba3SisoErrorV0> {
        config.validate()?;
        let inner_dim = config.inner_dim()?;
        let head_count = config.head_count()?;
        let projection_rows = config.input_projection_rows()?;
        if self.input_projection.rows != projection_rows
            || self.input_projection.cols != config.input_dim
            || self.dt_bias.dim != head_count
            || self.b_bias.rows != head_count
            || self.b_bias.cols != config.state_dim
            || self.c_bias.rows != head_count
            || self.c_bias.cols != config.state_dim
            || self.b_norm_scale.dim != config.state_dim
            || self.c_norm_scale.dim != config.state_dim
            || self.skip.dim != head_count
            || self.output_projection.rows != config.input_dim
            || self.output_projection.cols != inner_dim
        {
            return Err(Mamba3SisoErrorV0::ParameterShape);
        }
        if !self.input_projection.is_finite()
            || !self.dt_bias.is_finite()
            || !self.b_bias.is_finite()
            || !self.c_bias.is_finite()
            || !self.b_norm_scale.is_finite()
            || !self.c_norm_scale.is_finite()
            || !self.skip.is_finite()
            || !self.output_projection.is_finite()
        {
            return Err(Mamba3SisoErrorV0::NonFiniteValue);
        }
        Ok(())
    }

    pub fn parameter_count(&self) -> usize {
        self.input_projection.values.len()
            + self.dt_bias.values.len()
            + self.b_bias.values.len()
            + self.c_bias.values.len()
            + self.b_norm_scale.values.len()
            + self.c_norm_scale.values.len()
            + self.skip.values.len()
            + self.output_projection.values.len()
    }
}

pub fn mamba3_siso_params_from_seed_v0(
    config: &Mamba3SisoConfigV0,
    seed: u64,
) -> Result<Mamba3SisoParamsV0, Mamba3SisoErrorV0> {
    config.validate()?;
    let inner_dim = config.inner_dim()?;
    let head_count = config.head_count()?;
    let norm_scale = |salt: u64| -> Result<TinyTensor1D, Mamba3SisoErrorV0> {
        from_vec_1d(
            (0..config.state_dim)
                .map(|index| 1.0 + deterministic_tiny_value(seed, index, salt) * 0.1)
                .collect(),
        )
        .map_err(|_| Mamba3SisoErrorV0::TensorShape)
    };
    let result = Mamba3SisoParamsV0 {
        input_projection: deterministic_tiny_matrix(
            config.input_projection_rows()?,
            config.input_dim,
            seed,
            101,
        )
        .map_err(|_| Mamba3SisoErrorV0::TensorShape)?,
        dt_bias: from_vec_1d(vec![0.0; head_count]).map_err(|_| Mamba3SisoErrorV0::TensorShape)?,
        b_bias: deterministic_tiny_matrix(head_count, config.state_dim, seed, 102)
            .map_err(|_| Mamba3SisoErrorV0::TensorShape)?,
        c_bias: deterministic_tiny_matrix(head_count, config.state_dim, seed, 103)
            .map_err(|_| Mamba3SisoErrorV0::TensorShape)?,
        b_norm_scale: norm_scale(104)?,
        c_norm_scale: norm_scale(105)?,
        skip: from_vec_1d(
            (0..head_count)
                .map(|index| deterministic_tiny_value(seed, index, 106))
                .collect(),
        )
        .map_err(|_| Mamba3SisoErrorV0::TensorShape)?,
        output_projection: deterministic_tiny_matrix(config.input_dim, inner_dim, seed, 107)
            .map_err(|_| Mamba3SisoErrorV0::TensorShape)?,
    };
    result.validate(config)?;
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3SisoStateV0 {
    pub angle_state: TinyTensor2D,
    pub ssm_state: TinyTensor1D,
    pub previous_key: TinyTensor2D,
    pub previous_value: TinyTensor2D,
    pub step_index: usize,
}

impl Mamba3SisoStateV0 {
    pub fn zero(config: &Mamba3SisoConfigV0) -> Result<Self, Mamba3SisoErrorV0> {
        config.validate()?;
        let head_count = config.head_count()?;
        let inner_dim = config.inner_dim()?;
        let state = Self {
            angle_state: zeros_2d(head_count, config.rope_angle_count()?),
            ssm_state: zeros_1d(
                inner_dim
                    .checked_mul(config.state_dim)
                    .ok_or(Mamba3SisoErrorV0::Overflow)?,
            ),
            previous_key: zeros_2d(head_count, config.state_dim),
            previous_value: zeros_2d(head_count, config.head_dim),
            step_index: 0,
        };
        state.validate(config)?;
        Ok(state)
    }

    pub fn reset(&mut self) {
        self.angle_state.values.fill(0.0);
        self.ssm_state.values.fill(0.0);
        self.previous_key.values.fill(0.0);
        self.previous_value.values.fill(0.0);
        self.step_index = 0;
    }

    pub fn validate(&self, config: &Mamba3SisoConfigV0) -> Result<(), Mamba3SisoErrorV0> {
        config.validate()?;
        let head_count = config.head_count()?;
        let inner_dim = config.inner_dim()?;
        if self.angle_state.rows != head_count
            || self.angle_state.cols != config.rope_angle_count()?
            || self.ssm_state.dim
                != inner_dim
                    .checked_mul(config.state_dim)
                    .ok_or(Mamba3SisoErrorV0::Overflow)?
            || self.previous_key.rows != head_count
            || self.previous_key.cols != config.state_dim
            || self.previous_value.rows != head_count
            || self.previous_value.cols != config.head_dim
        {
            return Err(Mamba3SisoErrorV0::StateShape);
        }
        if !self.angle_state.is_finite()
            || !self.ssm_state.is_finite()
            || !self.previous_key.is_finite()
            || !self.previous_value.is_finite()
        {
            return Err(Mamba3SisoErrorV0::NonFiniteValue);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3SisoForwardResultV0 {
    pub output: Vec<TinyTensor1D>,
    pub final_state: Mamba3SisoStateV0,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3SisoModelMetadataV0 {
    pub format_version: u32,
    pub architecture: String,
    pub config: Mamba3SisoConfigV0,
    pub parameter_count: usize,
    pub reference_commit: String,
    pub reference_only: bool,
}

pub fn mamba3_siso_model_metadata_v0(
    config: &Mamba3SisoConfigV0,
    params: &Mamba3SisoParamsV0,
    reference_commit: impl Into<String>,
) -> Result<Mamba3SisoModelMetadataV0, Mamba3SisoErrorV0> {
    params.validate(config)?;
    Ok(Mamba3SisoModelMetadataV0 {
        format_version: 1,
        architecture: "mamba3-siso-reference-v0".to_string(),
        config: config.clone(),
        parameter_count: params.parameter_count(),
        reference_commit: reference_commit.into(),
        reference_only: true,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TinyMamba3SisoV0 {
    pub config: Mamba3SisoConfigV0,
    pub parameters: Mamba3SisoParamsV0,
}

impl TinyMamba3SisoV0 {
    pub fn new(
        config: Mamba3SisoConfigV0,
        parameters: Mamba3SisoParamsV0,
    ) -> Result<Self, Mamba3SisoErrorV0> {
        parameters.validate(&config)?;
        Ok(Self { config, parameters })
    }

    pub fn zero_state(&self) -> Result<Mamba3SisoStateV0, Mamba3SisoErrorV0> {
        Mamba3SisoStateV0::zero(&self.config)
    }

    pub fn step(
        &self,
        input: &TinyTensor1D,
        state: &mut Mamba3SisoStateV0,
    ) -> Result<TinyTensor1D, Mamba3SisoErrorV0> {
        mamba3_siso_step_v0(input, state, &self.parameters, &self.config)
    }

    pub fn forward(
        &self,
        input: &[TinyTensor1D],
    ) -> Result<Mamba3SisoForwardResultV0, Mamba3SisoErrorV0> {
        mamba3_siso_forward_v0(input, &self.parameters, &self.config)
    }
}

fn mamba3_siso_exact_matvec(
    matrix: &TinyTensor2D,
    input: &[f32],
) -> Result<Vec<f32>, Mamba3SisoErrorV0> {
    if matrix.cols != input.len() {
        return Err(Mamba3SisoErrorV0::TensorShape);
    }
    let mut output = Vec::with_capacity(matrix.rows);
    for row in 0..matrix.rows {
        let offset = row * matrix.cols;
        let value = matrix.values[offset..offset + matrix.cols]
            .iter()
            .zip(input)
            .map(|(weight, input)| weight * input)
            .sum::<f32>();
        if !value.is_finite() {
            return Err(Mamba3SisoErrorV0::NonFiniteValue);
        }
        output.push(value);
    }
    Ok(output)
}

fn mamba3_siso_softplus(value: f32) -> f32 {
    if value > 20.0 {
        value
    } else if value < -20.0 {
        value.exp()
    } else {
        value.exp().ln_1p()
    }
}

fn mamba3_siso_sigmoid(value: f32) -> f32 {
    if value >= 0.0 {
        1.0 / (1.0 + (-value).exp())
    } else {
        let exp = value.exp();
        exp / (1.0 + exp)
    }
}

fn mamba3_siso_heavy_tail(value: f32) -> f32 {
    if value >= 0.0 {
        1.0 + value
    } else {
        1.0 / (1.0 - value)
    }
}

fn mamba3_siso_silu(value: f32) -> f32 {
    value * mamba3_siso_sigmoid(value)
}

fn mamba3_siso_bc_norm(
    values: &[f32],
    scale: &[f32],
    epsilon: f32,
) -> Result<Vec<f32>, Mamba3SisoErrorV0> {
    if values.len() != scale.len() || values.is_empty() {
        return Err(Mamba3SisoErrorV0::TensorShape);
    }
    let mean_square = values.iter().map(|value| value * value).sum::<f32>() / values.len() as f32;
    let denominator = (mean_square + epsilon).sqrt();
    if !denominator.is_finite() || denominator == 0.0 {
        return Err(Mamba3SisoErrorV0::NonFiniteValue);
    }
    let output: Vec<f32> = values
        .iter()
        .zip(scale)
        .map(|(value, scale)| value * scale / denominator)
        .collect();
    if output.iter().any(|value| !value.is_finite()) {
        return Err(Mamba3SisoErrorV0::NonFiniteValue);
    }
    Ok(output)
}

fn mamba3_siso_rotate_pairwise(values: &mut [f32], angles: &[f32]) {
    for (pair, angle) in angles.iter().enumerate() {
        let index = pair * 2;
        let first = values[index];
        let second = values[index + 1];
        let cosine = angle.cos();
        let sine = angle.sin();
        values[index] = first * cosine - second * sine;
        values[index + 1] = first * sine + second * cosine;
    }
}

pub fn mamba3_siso_step_v0(
    input: &TinyTensor1D,
    state: &mut Mamba3SisoStateV0,
    params: &Mamba3SisoParamsV0,
    config: &Mamba3SisoConfigV0,
) -> Result<TinyTensor1D, Mamba3SisoErrorV0> {
    config.validate()?;
    params.validate(config)?;
    state.validate(config)?;
    if input.dim == 0 {
        return Err(Mamba3SisoErrorV0::EmptyInput);
    }
    if input.dim != config.input_dim {
        return Err(Mamba3SisoErrorV0::TensorShape);
    }
    if !input.is_finite() {
        return Err(Mamba3SisoErrorV0::NonFiniteValue);
    }

    let inner_dim = config.inner_dim()?;
    let head_count = config.head_count()?;
    let rope_angles = config.rope_angle_count()?;
    let projected = mamba3_siso_exact_matvec(&params.input_projection, &input.values)?;
    let mut offset = 0;
    let z = &projected[offset..offset + inner_dim];
    offset += inner_dim;
    let x = &projected[offset..offset + inner_dim];
    offset += inner_dim;
    let b = &projected[offset..offset + config.state_dim];
    offset += config.state_dim;
    let c = &projected[offset..offset + config.state_dim];
    offset += config.state_dim;
    let dd_dt = &projected[offset..offset + head_count];
    offset += head_count;
    let dd_a = &projected[offset..offset + head_count];
    offset += head_count;
    let trap = &projected[offset..offset + head_count];
    offset += head_count;
    let angle_projection = &projected[offset..offset + rope_angles];
    let normalized_b = mamba3_siso_bc_norm(b, &params.b_norm_scale.values, config.norm_epsilon)?;
    let normalized_c = mamba3_siso_bc_norm(c, &params.c_norm_scale.values, config.norm_epsilon)?;

    let mut inner_output = vec![0.0; inner_dim];
    let mut current_keys = vec![vec![0.0; config.state_dim]; head_count];
    for head in 0..head_count {
        let dt = mamba3_siso_softplus(dd_dt[head] + params.dt_bias.values[head]);
        let a = (-mamba3_siso_heavy_tail(dd_a[head])).min(-config.a_floor);
        let alpha = (a * dt).exp();
        let trapezoid = mamba3_siso_sigmoid(trap[head]);
        let beta = alpha * dt * (1.0 - trapezoid);
        let gamma = trapezoid * dt;
        if !dt.is_finite() || !alpha.is_finite() || !beta.is_finite() || !gamma.is_finite() {
            return Err(Mamba3SisoErrorV0::NonFiniteValue);
        }

        let angle_offset = head * rope_angles;
        let angles: Vec<f32> = angle_projection
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let next = state.angle_state.values[angle_offset + index]
                    + value.tanh() * dt * std::f32::consts::PI;
                next.rem_euclid(std::f32::consts::TAU)
            })
            .collect();
        state.angle_state.values[angle_offset..angle_offset + rope_angles].copy_from_slice(&angles);

        let bias_offset = head * config.state_dim;
        let mut key: Vec<f32> = normalized_b
            .iter()
            .zip(&params.b_bias.values[bias_offset..bias_offset + config.state_dim])
            .map(|(value, bias)| value + bias)
            .collect();
        let mut query: Vec<f32> = normalized_c
            .iter()
            .zip(&params.c_bias.values[bias_offset..bias_offset + config.state_dim])
            .map(|(value, bias)| value + bias)
            .collect();
        mamba3_siso_rotate_pairwise(&mut key, &angles);
        mamba3_siso_rotate_pairwise(&mut query, &angles);
        if key.iter().chain(&query).any(|value| !value.is_finite()) {
            return Err(Mamba3SisoErrorV0::NonFiniteValue);
        }

        for position in 0..config.head_dim {
            let inner_index = head * config.head_dim + position;
            let previous_value = state.previous_value.values[inner_index];
            let current_value = x[inner_index];
            let state_offset = inner_index * config.state_dim;
            let key_offset = head * config.state_dim;
            let mut output_value = 0.0;
            for state_index in 0..config.state_dim {
                let next = alpha * state.ssm_state.values[state_offset + state_index]
                    + beta * previous_value * state.previous_key.values[key_offset + state_index]
                    + gamma * current_value * key[state_index];
                if !next.is_finite() {
                    return Err(Mamba3SisoErrorV0::NonFiniteValue);
                }
                state.ssm_state.values[state_offset + state_index] = next;
                output_value += next * query[state_index];
            }
            inner_output[inner_index] = (output_value + params.skip.values[head] * current_value)
                * mamba3_siso_silu(z[inner_index]);
        }
        current_keys[head] = key;
    }
    for head in 0..head_count {
        let key_offset = head * config.state_dim;
        state.previous_key.values[key_offset..key_offset + config.state_dim]
            .copy_from_slice(&current_keys[head]);
    }
    state.previous_value.values.copy_from_slice(x);
    state.step_index = state
        .step_index
        .checked_add(1)
        .ok_or(Mamba3SisoErrorV0::Overflow)?;
    let output = mamba3_siso_exact_matvec(&params.output_projection, &inner_output)?;
    let output = from_vec_1d(output).map_err(|_| Mamba3SisoErrorV0::NonFiniteValue)?;
    state.validate(config)?;
    Ok(output)
}

pub fn mamba3_siso_forward_v0(
    sequence: &[TinyTensor1D],
    params: &Mamba3SisoParamsV0,
    config: &Mamba3SisoConfigV0,
) -> Result<Mamba3SisoForwardResultV0, Mamba3SisoErrorV0> {
    config.validate()?;
    params.validate(config)?;
    if sequence.len() > TINY_TENSOR_MAX_ELEMENTS {
        return Err(Mamba3SisoErrorV0::SequenceTooLong);
    }
    let mut state = Mamba3SisoStateV0::zero(config)?;
    let mut output = Vec::with_capacity(sequence.len());
    for input in sequence {
        output.push(mamba3_siso_step_v0(input, &mut state, params, config)?);
    }
    Ok(Mamba3SisoForwardResultV0 {
        output,
        final_state: state,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3SisoConformanceToleranceV0 {
    pub absolute: f32,
    pub relative: f32,
    pub state_absolute: f32,
}

impl Mamba3SisoConformanceToleranceV0 {
    pub fn validate(&self) -> Result<(), Mamba3SisoErrorV0> {
        if !self.absolute.is_finite()
            || !self.relative.is_finite()
            || !self.state_absolute.is_finite()
            || self.absolute < 0.0
            || self.relative < 0.0
            || self.state_absolute < 0.0
        {
            return Err(Mamba3SisoErrorV0::FixtureFormat);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3SisoFixtureProvenanceV0 {
    pub official_repository: String,
    pub official_commit: String,
    pub official_source_paths: Vec<String>,
    pub paper_identifier: String,
    pub python_version: String,
    pub pytorch_version: String,
    pub dtype: Mamba3SisoPrecisionV0,
    pub device: String,
    pub parameter_ordering: Vec<String>,
    pub parameter_count: usize,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3SisoReferenceFixtureV0 {
    pub format_version: u32,
    pub metadata: Mamba3SisoModelMetadataV0,
    pub provenance: Mamba3SisoFixtureProvenanceV0,
    pub parameters: Mamba3SisoParamsV0,
    pub initial_state: Mamba3SisoStateV0,
    pub input: Vec<TinyTensor1D>,
    #[serde(default)]
    pub expected_output: Option<Vec<TinyTensor1D>>,
    #[serde(default)]
    pub expected_state: Option<Vec<Mamba3SisoStateV0>>,
    pub tolerance: Mamba3SisoConformanceToleranceV0,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3ConformanceStatusV0 {
    OfficialOracleUnavailable,
    InternalReferenceOnly,
    OfficialOutputMatched,
    OfficialOutputAndStateMatched,
    OfficialMismatch,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3ConformanceReportV0 {
    pub status: Mamba3ConformanceStatusV0,
    pub max_output_abs_error: Option<f32>,
    pub max_output_rel_error: Option<f32>,
    pub max_state_abs_error: Option<f32>,
    pub failing_step: Option<usize>,
    pub failing_index: Option<usize>,
    pub compared_output_values: usize,
    pub compared_state_values: usize,
}

impl Mamba3SisoReferenceFixtureV0 {
    pub fn from_json(input: &str) -> Result<Self, Mamba3SisoErrorV0> {
        let fixture: Self =
            serde_json::from_str(input).map_err(|_| Mamba3SisoErrorV0::FixtureFormat)?;
        fixture.validate()?;
        Ok(fixture)
    }

    pub fn refresh_digest(&mut self) -> Result<(), Mamba3SisoErrorV0> {
        self.provenance.digest = self.computed_digest()?;
        Ok(())
    }

    pub fn computed_digest(&self) -> Result<String, Mamba3SisoErrorV0> {
        let mut canonical = self.clone();
        canonical.provenance.digest.clear();
        let text =
            serde_json::to_string(&canonical).map_err(|_| Mamba3SisoErrorV0::FixtureFormat)?;
        let digest = text.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ byte as u64).wrapping_mul(0x100000001b3)
        });
        Ok(format!("fnv1a64-{digest:016x}"))
    }

    pub fn validate(&self) -> Result<(), Mamba3SisoErrorV0> {
        if self.format_version != 1
            || self.metadata.format_version != 1
            || self.metadata.architecture != "mamba3-siso-reference-v0"
            || self.metadata.reference_commit.is_empty()
            || self.provenance.official_repository.is_empty()
            || self.provenance.official_commit.is_empty()
            || self.provenance.official_source_paths.is_empty()
            || self.provenance.paper_identifier.is_empty()
            || self.provenance.python_version.is_empty()
            || self.provenance.pytorch_version.is_empty()
            || self.provenance.device.is_empty()
            || self.provenance.parameter_ordering.is_empty()
            || self.provenance.dtype != Mamba3SisoPrecisionV0::F32
            || self.provenance.digest.is_empty()
            || self.provenance.official_commit != self.metadata.reference_commit
        {
            return Err(Mamba3SisoErrorV0::FixtureFormat);
        }
        self.tolerance.validate()?;
        self.parameters.validate(&self.metadata.config)?;
        self.initial_state.validate(&self.metadata.config)?;
        if self.parameters.parameter_count() != self.metadata.parameter_count
            || self.parameters.parameter_count() != self.provenance.parameter_count
        {
            return Err(Mamba3SisoErrorV0::FixtureFormat);
        }
        if self
            .input
            .iter()
            .any(|value| value.dim != self.metadata.config.input_dim || !value.is_finite())
        {
            return Err(Mamba3SisoErrorV0::FixtureFormat);
        }
        if let Some(expected) = &self.expected_output {
            if expected.len() != self.input.len()
                || expected
                    .iter()
                    .any(|value| value.dim != self.metadata.config.input_dim || !value.is_finite())
            {
                return Err(Mamba3SisoErrorV0::FixtureFormat);
            }
        }
        if let Some(expected) = &self.expected_state {
            if expected.len() != self.input.len()
                || expected
                    .iter()
                    .any(|state| state.validate(&self.metadata.config).is_err())
            {
                return Err(Mamba3SisoErrorV0::FixtureFormat);
            }
        }
        if self.provenance.digest != self.computed_digest()? {
            return Err(Mamba3SisoErrorV0::FixtureDigest);
        }
        Ok(())
    }
}

fn mamba3_siso_compare_values(
    actual: &[f32],
    expected: &[f32],
    absolute: f32,
    relative: f32,
    step: usize,
    max_absolute: &mut f32,
    max_relative: &mut f32,
    failing_step: &mut Option<usize>,
    failing_index: &mut Option<usize>,
    compared_values: &mut usize,
) {
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        let absolute_error = (actual - expected).abs();
        let relative_error = absolute_error / expected.abs().max(f32::MIN_POSITIVE);
        *max_absolute = (*max_absolute).max(absolute_error);
        *max_relative = (*max_relative).max(relative_error);
        *compared_values += 1;
        if failing_step.is_none() && absolute_error > absolute + relative * expected.abs() {
            *failing_step = Some(step);
            *failing_index = Some(index);
        }
    }
}

fn mamba3_siso_state_values(state: &Mamba3SisoStateV0) -> impl Iterator<Item = f32> + '_ {
    state
        .angle_state
        .values
        .iter()
        .chain(&state.ssm_state.values)
        .chain(&state.previous_key.values)
        .chain(&state.previous_value.values)
        .copied()
}

pub fn mamba3_siso_conformance_v0(
    fixture: &Mamba3SisoReferenceFixtureV0,
) -> Result<Mamba3ConformanceReportV0, Mamba3SisoErrorV0> {
    fixture.validate()?;
    let Some(expected_output) = fixture.expected_output.as_ref() else {
        return Ok(Mamba3ConformanceReportV0 {
            status: Mamba3ConformanceStatusV0::OfficialOracleUnavailable,
            max_output_abs_error: None,
            max_output_rel_error: None,
            max_state_abs_error: None,
            failing_step: None,
            failing_index: None,
            compared_output_values: 0,
            compared_state_values: 0,
        });
    };
    let mut state = fixture.initial_state.clone();
    let mut max_output_abs_error = 0.0_f32;
    let mut max_output_rel_error = 0.0_f32;
    let mut max_state_abs_error = 0.0_f32;
    let mut failing_step = None;
    let mut failing_index = None;
    let mut compared_output_values = 0;
    let mut compared_state_values = 0;
    for (step, (input, expected)) in fixture.input.iter().zip(expected_output).enumerate() {
        let actual = mamba3_siso_step_v0(
            input,
            &mut state,
            &fixture.parameters,
            &fixture.metadata.config,
        )?;
        mamba3_siso_compare_values(
            &actual.values,
            &expected.values,
            fixture.tolerance.absolute,
            fixture.tolerance.relative,
            step,
            &mut max_output_abs_error,
            &mut max_output_rel_error,
            &mut failing_step,
            &mut failing_index,
            &mut compared_output_values,
        );
        if let Some(expected_states) = &fixture.expected_state {
            let actual_state: Vec<f32> = mamba3_siso_state_values(&state).collect();
            let expected_state: Vec<f32> =
                mamba3_siso_state_values(&expected_states[step]).collect();
            mamba3_siso_compare_values(
                &actual_state,
                &expected_state,
                fixture.tolerance.state_absolute,
                0.0,
                step,
                &mut max_state_abs_error,
                &mut max_output_rel_error,
                &mut failing_step,
                &mut failing_index,
                &mut compared_state_values,
            );
        }
    }
    let mismatch = failing_step.is_some();
    Ok(Mamba3ConformanceReportV0 {
        status: if mismatch {
            Mamba3ConformanceStatusV0::OfficialMismatch
        } else if fixture.expected_state.is_some() {
            Mamba3ConformanceStatusV0::OfficialOutputAndStateMatched
        } else {
            Mamba3ConformanceStatusV0::OfficialOutputMatched
        },
        max_output_abs_error: Some(max_output_abs_error),
        max_output_rel_error: Some(max_output_rel_error),
        max_state_abs_error: fixture.expected_state.as_ref().map(|_| max_state_abs_error),
        failing_step,
        failing_index,
        compared_output_values,
        compared_state_values,
    })
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
struct Mamba3SisoStepTraceV0 {
    projected_input: Vec<f32>,
    dt: Vec<f32>,
    normalized_b: Vec<f32>,
    normalized_c: Vec<f32>,
    output: TinyTensor1D,
    state: Mamba3SisoStateV0,
}

#[cfg(test)]
fn mamba3_siso_step_trace_v0(
    input: &TinyTensor1D,
    state: &mut Mamba3SisoStateV0,
    params: &Mamba3SisoParamsV0,
    config: &Mamba3SisoConfigV0,
) -> Result<Mamba3SisoStepTraceV0, Mamba3SisoErrorV0> {
    params.validate(config)?;
    state.validate(config)?;
    let projected_input = mamba3_siso_exact_matvec(&params.input_projection, &input.values)?;
    let inner_dim = config.inner_dim()?;
    let head_count = config.head_count()?;
    let b_start = inner_dim * 2;
    let c_start = b_start + config.state_dim;
    let dt_start = c_start + config.state_dim;
    let normalized_b = mamba3_siso_bc_norm(
        &projected_input[b_start..b_start + config.state_dim],
        &params.b_norm_scale.values,
        config.norm_epsilon,
    )?;
    let normalized_c = mamba3_siso_bc_norm(
        &projected_input[c_start..c_start + config.state_dim],
        &params.c_norm_scale.values,
        config.norm_epsilon,
    )?;
    let dt = (0..head_count)
        .map(|head| {
            mamba3_siso_softplus(projected_input[dt_start + head] + params.dt_bias.values[head])
        })
        .collect();
    let output = mamba3_siso_step_v0(input, state, params, config)?;
    Ok(Mamba3SisoStepTraceV0 {
        projected_input,
        dt,
        normalized_b,
        normalized_c,
        output,
        state: state.clone(),
    })
}

#[cfg(test)]
mod mamba3_siso_reference_core_tests {
    use super::*;
    use crate::model::tiny_tensor::from_vec_2d;

    fn config() -> Mamba3SisoConfigV0 {
        Mamba3SisoConfigV0 {
            input_dim: 2,
            state_dim: 4,
            head_dim: 2,
            expansion: 1,
            rope_fraction: Mamba3SisoRopeFractionV0::Half,
            norm_epsilon: 1e-5,
            a_floor: 1e-4,
            mimo_rank: 1,
            precision: Mamba3SisoPrecisionV0::F32,
            short_convolution_enabled: false,
        }
    }

    fn sequence() -> Vec<TinyTensor1D> {
        vec![
            from_vec_1d(vec![0.1, -0.2]).unwrap(),
            from_vec_1d(vec![0.3, 0.4]).unwrap(),
            from_vec_1d(vec![-0.2, 0.5]).unwrap(),
        ]
    }

    fn fixture(
        config: Mamba3SisoConfigV0,
        parameters: Mamba3SisoParamsV0,
        input: Vec<TinyTensor1D>,
        expected_output: Option<Vec<TinyTensor1D>>,
        expected_state: Option<Vec<Mamba3SisoStateV0>>,
    ) -> Mamba3SisoReferenceFixtureV0 {
        let reference_commit = "f577286d052741c35d39cd43bdc3fad27120f22c";
        let mut fixture = Mamba3SisoReferenceFixtureV0 {
            format_version: 1,
            metadata: mamba3_siso_model_metadata_v0(&config, &parameters, reference_commit)
                .unwrap(),
            provenance: Mamba3SisoFixtureProvenanceV0 {
                official_repository: "state-spaces/mamba".to_string(),
                official_commit: reference_commit.to_string(),
                official_source_paths: vec![
                    "mamba_ssm/modules/mamba3.py".to_string(),
                    "mamba_ssm/ops/triton/mamba3/mamba3_siso_step.py".to_string(),
                ],
                paper_identifier: "arXiv:2603.15569".to_string(),
                python_version: "test-only".to_string(),
                pytorch_version: "test-only".to_string(),
                dtype: Mamba3SisoPrecisionV0::F32,
                device: "cpu-reference".to_string(),
                parameter_ordering: vec![
                    "input_projection".to_string(),
                    "dt_bias".to_string(),
                    "b_bias".to_string(),
                    "c_bias".to_string(),
                    "b_norm_scale".to_string(),
                    "c_norm_scale".to_string(),
                    "skip".to_string(),
                    "output_projection".to_string(),
                ],
                parameter_count: parameters.parameter_count(),
                digest: String::new(),
            },
            initial_state: Mamba3SisoStateV0::zero(&config).unwrap(),
            parameters,
            input,
            expected_output,
            expected_state,
            tolerance: Mamba3SisoConformanceToleranceV0 {
                absolute: 1e-6,
                relative: 1e-6,
                state_absolute: 1e-6,
            },
        };
        fixture.refresh_digest().unwrap();
        fixture
    }

    #[test]
    fn mamba3_siso_rejects_unsupported_shapes_and_modes() {
        assert!(config().validate().is_ok());
        let mut invalid = config();
        invalid.input_dim = 0;
        assert_eq!(
            invalid.validate(),
            Err(Mamba3SisoErrorV0::InvalidConfiguration)
        );
        let mut invalid = config();
        invalid.head_dim = 3;
        assert_eq!(
            invalid.validate(),
            Err(Mamba3SisoErrorV0::InvalidConfiguration)
        );
        let mut invalid = config();
        invalid.mimo_rank = 2;
        assert_eq!(invalid.validate(), Err(Mamba3SisoErrorV0::UnsupportedMimo));
        let mut invalid = config();
        invalid.precision = Mamba3SisoPrecisionV0::F64Unsupported;
        assert_eq!(
            invalid.validate(),
            Err(Mamba3SisoErrorV0::UnsupportedPrecision)
        );
        let mut invalid = config();
        invalid.short_convolution_enabled = true;
        assert_eq!(
            invalid.validate(),
            Err(Mamba3SisoErrorV0::UnsupportedShortConvolution)
        );
        let mut invalid = config();
        invalid.norm_epsilon = 0.0;
        assert_eq!(
            invalid.validate(),
            Err(Mamba3SisoErrorV0::InvalidConfiguration)
        );
    }

    #[test]
    fn mamba3_siso_seeded_parameters_are_deterministic_and_validated() {
        let config = config();
        let first = mamba3_siso_params_from_seed_v0(&config, 17).unwrap();
        let second = mamba3_siso_params_from_seed_v0(&config, 17).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.parameter_count(), second.parameter_count());
        let mut wrong_shape = first.clone();
        wrong_shape.input_projection.rows -= 1;
        assert_eq!(
            wrong_shape.validate(&config),
            Err(Mamba3SisoErrorV0::ParameterShape)
        );
        let mut invalid = first.clone();
        invalid.skip.values[0] = f32::NAN;
        assert_eq!(
            invalid.validate(&config),
            Err(Mamba3SisoErrorV0::NonFiniteValue)
        );
        invalid.skip.values[0] = f32::INFINITY;
        assert_eq!(
            invalid.validate(&config),
            Err(Mamba3SisoErrorV0::NonFiniteValue)
        );
    }

    #[test]
    fn mamba3_siso_bc_norm_and_rotation_follow_real_pair_rules() {
        let normalized = mamba3_siso_bc_norm(&[3.0, 4.0], &[1.0, 1.0], 0.0).unwrap();
        assert!((normalized[0] - 0.848_528_15).abs() < 1e-6);
        assert!((normalized[1] - 1.131_370_9).abs() < 1e-6);
        let mut pair = vec![1.0, 0.0];
        mamba3_siso_rotate_pairwise(&mut pair, &[std::f32::consts::FRAC_PI_2]);
        assert!(pair[0].abs() < 1e-6);
        assert!((pair[1] - 1.0).abs() < 1e-6);
        let mut unchanged = vec![0.25, -0.5];
        mamba3_siso_rotate_pairwise(&mut unchanged, &[0.0]);
        assert_eq!(unchanged, vec![0.25, -0.5]);
        assert_eq!(
            mamba3_siso_bc_norm(&[1.0, 2.0], &[1.0], 1e-5),
            Err(Mamba3SisoErrorV0::TensorShape)
        );
        assert_eq!(
            mamba3_siso_bc_norm(&[0.0, 0.0], &[1.0, 1.0], 1e-5).unwrap(),
            vec![0.0, 0.0]
        );
    }

    #[test]
    fn mamba3_siso_step_updates_the_exponential_trapezoidal_state() {
        let config = Mamba3SisoConfigV0 {
            input_dim: 1,
            state_dim: 2,
            head_dim: 1,
            expansion: 1,
            rope_fraction: Mamba3SisoRopeFractionV0::Full,
            norm_epsilon: 0.0_f32.max(1e-6),
            a_floor: 1e-4,
            mimo_rank: 1,
            precision: Mamba3SisoPrecisionV0::F32,
            short_convolution_enabled: false,
        };
        let projection = from_vec_2d(
            10,
            1,
            vec![1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        )
        .unwrap();
        let params = Mamba3SisoParamsV0 {
            input_projection: projection,
            dt_bias: from_vec_1d(vec![0.0]).unwrap(),
            b_bias: zeros_2d(1, 2),
            c_bias: zeros_2d(1, 2),
            b_norm_scale: from_vec_1d(vec![1.0, 1.0]).unwrap(),
            c_norm_scale: from_vec_1d(vec![1.0, 1.0]).unwrap(),
            skip: from_vec_1d(vec![0.0]).unwrap(),
            output_projection: from_vec_2d(1, 1, vec![1.0]).unwrap(),
        };
        let mut state = Mamba3SisoStateV0::zero(&config).unwrap();
        let output = mamba3_siso_step_v0(
            &from_vec_1d(vec![1.0]).unwrap(),
            &mut state,
            &params,
            &config,
        )
        .unwrap();
        let dt = mamba3_siso_softplus(0.0);
        let key = 1.0 / ((0.5 + config.norm_epsilon).sqrt());
        let expected_state = mamba3_siso_sigmoid(0.0) * dt * key;
        assert!((state.ssm_state.values[0] - expected_state).abs() < 1e-6);
        assert!(output.values[0] > 0.0);
    }

    #[test]
    fn mamba3_siso_full_forward_matches_persistent_streaming_state() {
        let config = config();
        let params = mamba3_siso_params_from_seed_v0(&config, 23).unwrap();
        let input = sequence();
        let full = mamba3_siso_forward_v0(&input, &params, &config).unwrap();
        let mut streaming_state = Mamba3SisoStateV0::zero(&config).unwrap();
        let streamed: Vec<TinyTensor1D> = input
            .iter()
            .map(|row| mamba3_siso_step_v0(row, &mut streaming_state, &params, &config).unwrap())
            .collect();
        assert_eq!(full.output, streamed);
        assert_eq!(full.final_state, streaming_state);
    }

    #[test]
    fn mamba3_siso_test_trace_uses_the_same_step_semantics() {
        let config = config();
        let params = mamba3_siso_params_from_seed_v0(&config, 27).unwrap();
        let input = sequence().remove(0);
        let mut traced_state = Mamba3SisoStateV0::zero(&config).unwrap();
        let trace = mamba3_siso_step_trace_v0(&input, &mut traced_state, &params, &config).unwrap();
        let mut plain_state = Mamba3SisoStateV0::zero(&config).unwrap();
        let plain = mamba3_siso_step_v0(&input, &mut plain_state, &params, &config).unwrap();
        assert_eq!(trace.output, plain);
        assert_eq!(trace.state, plain_state);
        assert_eq!(trace.dt.len(), config.head_count().unwrap());
        assert_eq!(trace.normalized_b.len(), config.state_dim);
        assert_eq!(trace.normalized_c.len(), config.state_dim);
    }

    #[test]
    fn mamba3_siso_validates_state_and_keeps_callers_input_and_parameters_unchanged() {
        let config = config();
        let params = mamba3_siso_params_from_seed_v0(&config, 29).unwrap();
        let input = sequence();
        let input_before = input.clone();
        let params_before = params.clone();
        let output = mamba3_siso_forward_v0(&input, &params, &config).unwrap();
        assert_eq!(input, input_before);
        assert_eq!(params, params_before);
        assert_eq!(output.output.len(), input.len());
        let mut invalid_state = output.final_state.clone();
        invalid_state.previous_key.cols = 1;
        assert_eq!(
            invalid_state.validate(&config),
            Err(Mamba3SisoErrorV0::StateShape)
        );
        let mut invalid_state = output.final_state;
        invalid_state.ssm_state.values[0] = f32::INFINITY;
        assert_eq!(
            invalid_state.validate(&config),
            Err(Mamba3SisoErrorV0::NonFiniteValue)
        );
    }

    #[test]
    fn mamba3_siso_empty_and_zero_sequences_have_explicit_reference_behavior() {
        let config = config();
        let params = mamba3_siso_params_from_seed_v0(&config, 37).unwrap();
        let empty = mamba3_siso_forward_v0(&[], &params, &config).unwrap();
        assert!(empty.output.is_empty());
        assert_eq!(empty.final_state, Mamba3SisoStateV0::zero(&config).unwrap());
        let zero = from_vec_1d(vec![0.0, 0.0]).unwrap();
        let mut state = Mamba3SisoStateV0::zero(&config).unwrap();
        let output = mamba3_siso_step_v0(&zero, &mut state, &params, &config).unwrap();
        assert!(output.values.iter().all(|value| *value == 0.0));
        assert!(state.ssm_state.values.iter().all(|value| *value == 0.0));
    }

    #[test]
    fn mamba3_siso_output_depends_on_parameters_and_explicit_state() {
        let config = config();
        let params = mamba3_siso_params_from_seed_v0(&config, 39).unwrap();
        let model = TinyMamba3SisoV0::new(config.clone(), params.clone()).unwrap();
        let token = sequence().remove(0);
        let mut first_state = model.zero_state().unwrap();
        let first = model.step(&token, &mut first_state).unwrap();
        let mut continued_state = first_state.clone();
        let continued = model.step(&token, &mut continued_state).unwrap();
        let mut reset_state = model.zero_state().unwrap();
        let repeated_first = model.step(&token, &mut reset_state).unwrap();
        assert_eq!(first, repeated_first);
        assert_ne!(first, continued);
        let mut changed_params = params;
        changed_params.output_projection.values[0] += 0.5;
        let changed = TinyMamba3SisoV0::new(config, changed_params)
            .unwrap()
            .forward(&[token])
            .unwrap();
        assert_ne!(changed.output[0], first);
    }

    #[test]
    fn mamba3_siso_state_reset_returns_to_zero_state() {
        let config = config();
        let params = mamba3_siso_params_from_seed_v0(&config, 31).unwrap();
        let mut state = Mamba3SisoStateV0::zero(&config).unwrap();
        mamba3_siso_step_v0(&sequence()[0], &mut state, &params, &config).unwrap();
        state.reset();
        assert_eq!(state, Mamba3SisoStateV0::zero(&config).unwrap());
    }

    #[test]
    fn mamba3_siso_metadata_and_fixture_keep_oracle_availability_explicit() {
        let config = config();
        let params = mamba3_siso_params_from_seed_v0(&config, 41).unwrap();
        let fixture = fixture(config, params, sequence(), None, None);
        assert!(fixture.metadata.reference_only);
        let parsed =
            Mamba3SisoReferenceFixtureV0::from_json(&serde_json::to_string(&fixture).unwrap())
                .unwrap();
        assert_eq!(
            parsed.metadata.reference_commit,
            "f577286d052741c35d39cd43bdc3fad27120f22c"
        );
        let report = mamba3_siso_conformance_v0(&parsed).unwrap();
        assert_eq!(
            report.status,
            Mamba3ConformanceStatusV0::OfficialOracleUnavailable
        );
        assert_eq!(report.compared_output_values, 0);
        assert_eq!(
            Mamba3SisoReferenceFixtureV0::from_json("{not-valid-json"),
            Err(Mamba3SisoErrorV0::FixtureFormat)
        );
        let mut corrupted = fixture;
        corrupted.provenance.digest.push('0');
        assert_eq!(corrupted.validate(), Err(Mamba3SisoErrorV0::FixtureDigest));
    }

    #[test]
    fn mamba3_siso_fixture_rejects_missing_provenance_and_mimo() {
        let config = config();
        let params = mamba3_siso_params_from_seed_v0(&config, 47).unwrap();
        let mut missing_commit = fixture(config.clone(), params.clone(), sequence(), None, None);
        missing_commit.provenance.official_commit.clear();
        missing_commit.refresh_digest().unwrap();
        assert_eq!(
            missing_commit.validate(),
            Err(Mamba3SisoErrorV0::FixtureFormat)
        );
        let mut mimo = fixture(config, params, sequence(), None, None);
        mimo.metadata.config.mimo_rank = 2;
        mimo.refresh_digest().unwrap();
        assert_eq!(mimo.validate(), Err(Mamba3SisoErrorV0::UnsupportedMimo));
    }

    #[test]
    fn mamba3_siso_fixture_compares_only_supplied_reference_outputs() {
        let config = config();
        let params = mamba3_siso_params_from_seed_v0(&config, 53).unwrap();
        let input = sequence();
        let output = mamba3_siso_forward_v0(&input, &params, &config)
            .unwrap()
            .output;
        let mut state = Mamba3SisoStateV0::zero(&config).unwrap();
        let expected_state = input
            .iter()
            .map(|token| {
                mamba3_siso_step_v0(token, &mut state, &params, &config).unwrap();
                state.clone()
            })
            .collect();
        let fixture = fixture(config, params, input, Some(output), Some(expected_state));
        assert_eq!(
            mamba3_siso_conformance_v0(&fixture).unwrap().status,
            Mamba3ConformanceStatusV0::OfficialOutputAndStateMatched
        );
        let mut tolerated = fixture.clone();
        tolerated.expected_output.as_mut().unwrap()[0].values[0] += 1e-7;
        tolerated.refresh_digest().unwrap();
        assert_eq!(
            mamba3_siso_conformance_v0(&tolerated).unwrap().status,
            Mamba3ConformanceStatusV0::OfficialOutputAndStateMatched
        );
        let mut mismatched = fixture;
        mismatched.expected_output.as_mut().unwrap()[1].values[0] += 0.1;
        mismatched.refresh_digest().unwrap();
        let report = mamba3_siso_conformance_v0(&mismatched).unwrap();
        assert_eq!(report.status, Mamba3ConformanceStatusV0::OfficialMismatch);
        assert_eq!(report.failing_step, Some(1));
        let mut state_mismatch = tolerated;
        state_mismatch.expected_state.as_mut().unwrap()[0]
            .ssm_state
            .values[0] += 0.1;
        state_mismatch.refresh_digest().unwrap();
        let report = mamba3_siso_conformance_v0(&state_mismatch).unwrap();
        assert_eq!(report.status, Mamba3ConformanceStatusV0::OfficialMismatch);
        assert_eq!(report.failing_step, Some(0));
        assert!(report.max_state_abs_error.unwrap() >= 0.1);
    }
}
