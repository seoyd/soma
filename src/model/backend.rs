use super::mamba3::{
    Mamba3SisoErrorV0, Mamba3SisoForwardResultV0, Mamba3SisoStateV0, TinyMamba3SisoV0,
};
use super::tiny_tensor::TinyTensor1D;
use serde::{Deserialize, Serialize};

#[cfg(all(target_os = "macos", feature = "backend-metal"))]
mod backend_metal;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Mamba3BackendKind {
    CpuReference,
    Metal,
    Cuda,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendPreference {
    Auto,
    Cpu,
    Metal,
    Cuda,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendFallbackPolicy {
    AllowCpuFallback,
    Strict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendReadiness {
    Unavailable,
    NotCompiled,
    RuntimeUnavailable,
    DeviceUnavailable,
    PartialOperations,
    SelfTestFailed,
    ReferenceParityPassed,
    FullInferenceReady,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelPrecision {
    F32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendOperation {
    DenseProjection,
    BcNormalization,
    TransitionCoefficient,
    ComplexStateTransition,
    StateReadout,
    Gate,
    OutputProjection,
    FullStep,
    FullForward,
    StateImport,
    StateExport,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackendOperationSet(u16);

impl BackendOperationSet {
    pub const EMPTY: Self = Self(0);
    pub const FULL_INFERENCE: Self = Self(
        Self::bit(BackendOperation::DenseProjection)
            | Self::bit(BackendOperation::BcNormalization)
            | Self::bit(BackendOperation::TransitionCoefficient)
            | Self::bit(BackendOperation::ComplexStateTransition)
            | Self::bit(BackendOperation::StateReadout)
            | Self::bit(BackendOperation::Gate)
            | Self::bit(BackendOperation::OutputProjection)
            | Self::bit(BackendOperation::FullStep)
            | Self::bit(BackendOperation::FullForward)
            | Self::bit(BackendOperation::StateImport)
            | Self::bit(BackendOperation::StateExport),
    );

    const fn bit(operation: BackendOperation) -> u16 {
        1 << operation as u8
    }

    pub const fn from_operation(operation: BackendOperation) -> Self {
        Self(Self::bit(operation))
    }

    pub const fn contains(self, operation: BackendOperation) -> bool {
        self.0 & Self::bit(operation) != 0
    }

    pub const fn contains_all(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendReasonCode {
    CpuReferenceAvailable,
    FeatureNotCompiled,
    UnsupportedTarget,
    RuntimeLibraryUnavailable,
    NoCompatibleDevice,
    PartialOperationCoverage,
    SelfTestNotRun,
    BackendParityNotRun,
    CpuFallbackUsed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendDeviceInfo {
    pub kind: Mamba3BackendKind,
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendCapabilities {
    pub kind: Mamba3BackendKind,
    pub readiness: BackendReadiness,
    pub supported_operations: BackendOperationSet,
    pub supported_precisions: Vec<ModelPrecision>,
    pub device_count: usize,
    pub selected_device: Option<BackendDeviceInfo>,
    pub reason_codes: Vec<BackendReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendSelectionRequest {
    pub preference: BackendPreference,
    pub fallback_policy: BackendFallbackPolicy,
    pub required_operations: BackendOperationSet,
    pub required_precision: ModelPrecision,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendSelection {
    pub selected: Mamba3BackendKind,
    pub fallback_reason: Option<BackendReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendError {
    UnsupportedTarget,
    BackendNotCompiled,
    RuntimeUnavailable,
    DeviceUnavailable,
    UnsupportedOperation,
    UnsupportedPrecision,
    StateBackendMismatch,
    StrictBackendUnavailable,
    InvalidTransitionInput,
    NumericalParityFailed,
    Model(Mamba3SisoErrorV0),
}

impl From<Mamba3SisoErrorV0> for BackendError {
    fn from(value: Mamba3SisoErrorV0) -> Self {
        Self::Model(value)
    }
}

pub trait BackendCapabilityProbe {
    fn probe_cpu(&self) -> BackendCapabilities;
    fn probe_metal(&self) -> BackendCapabilities;
    fn probe_cuda(&self) -> BackendCapabilities;
}

#[derive(Clone, Debug)]
pub struct StaticBackendCapabilityProbe {
    pub cpu: BackendCapabilities,
    pub metal: BackendCapabilities,
    pub cuda: BackendCapabilities,
}

impl BackendCapabilityProbe for StaticBackendCapabilityProbe {
    fn probe_cpu(&self) -> BackendCapabilities {
        self.cpu.clone()
    }
    fn probe_metal(&self) -> BackendCapabilities {
        self.metal.clone()
    }
    fn probe_cuda(&self) -> BackendCapabilities {
        self.cuda.clone()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemBackendCapabilityProbe;

impl BackendCapabilityProbe for SystemBackendCapabilityProbe {
    fn probe_cpu(&self) -> BackendCapabilities {
        cpu_capabilities()
    }

    fn probe_metal(&self) -> BackendCapabilities {
        #[cfg(all(target_os = "macos", feature = "backend-metal"))]
        {
            return backend_metal::probe_metal();
        }
        #[cfg(all(target_os = "macos", not(feature = "backend-metal")))]
        {
            return unavailable(
                Mamba3BackendKind::Metal,
                BackendReadiness::NotCompiled,
                BackendReasonCode::FeatureNotCompiled,
            );
        }
        #[cfg(not(target_os = "macos"))]
        {
            unavailable(
                Mamba3BackendKind::Metal,
                BackendReadiness::Unavailable,
                BackendReasonCode::UnsupportedTarget,
            )
        }
    }

    fn probe_cuda(&self) -> BackendCapabilities {
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            unavailable(
                Mamba3BackendKind::Cuda,
                BackendReadiness::NotCompiled,
                BackendReasonCode::FeatureNotCompiled,
            )
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            unavailable(
                Mamba3BackendKind::Cuda,
                BackendReadiness::Unavailable,
                BackendReasonCode::UnsupportedTarget,
            )
        }
    }
}

fn cpu_capabilities() -> BackendCapabilities {
    BackendCapabilities {
        kind: Mamba3BackendKind::CpuReference,
        readiness: BackendReadiness::FullInferenceReady,
        supported_operations: BackendOperationSet::FULL_INFERENCE,
        supported_precisions: vec![ModelPrecision::F32],
        device_count: 1,
        selected_device: None,
        reason_codes: vec![BackendReasonCode::CpuReferenceAvailable],
    }
}

fn unavailable(
    kind: Mamba3BackendKind,
    readiness: BackendReadiness,
    reason: BackendReasonCode,
) -> BackendCapabilities {
    BackendCapabilities {
        kind,
        readiness,
        supported_operations: BackendOperationSet::EMPTY,
        supported_precisions: vec![],
        device_count: 0,
        selected_device: None,
        reason_codes: vec![reason],
    }
}

fn eligible(capabilities: &BackendCapabilities, request: BackendSelectionRequest) -> bool {
    capabilities.readiness == BackendReadiness::FullInferenceReady
        && capabilities
            .supported_precisions
            .contains(&request.required_precision)
        && capabilities
            .supported_operations
            .contains_all(request.required_operations)
}

pub fn select_mamba3_backend(
    probe: &impl BackendCapabilityProbe,
    request: BackendSelectionRequest,
) -> Result<BackendSelection, BackendError> {
    let cpu = probe.probe_cpu();
    let requested = match request.preference {
        BackendPreference::Cpu => Some(cpu.clone()),
        BackendPreference::Metal => Some(probe.probe_metal()),
        BackendPreference::Cuda => Some(probe.probe_cuda()),
        BackendPreference::Auto => [probe.probe_metal(), probe.probe_cuda(), cpu.clone()]
            .into_iter()
            .find(|capabilities| eligible(capabilities, request)),
    };
    if let Some(capabilities) = requested.filter(|capabilities| eligible(capabilities, request)) {
        return Ok(BackendSelection {
            selected: capabilities.kind,
            fallback_reason: None,
        });
    }
    if request.fallback_policy == BackendFallbackPolicy::AllowCpuFallback && eligible(&cpu, request)
    {
        return Ok(BackendSelection {
            selected: Mamba3BackendKind::CpuReference,
            fallback_reason: Some(BackendReasonCode::CpuFallbackUsed),
        });
    }
    Err(BackendError::StrictBackendUnavailable)
}

#[derive(Clone, Debug, PartialEq)]
pub enum BackendState {
    Cpu(Mamba3SisoStateV0),
}

pub trait Mamba3ExecutionBackend {
    fn kind(&self) -> Mamba3BackendKind;
    fn capabilities(&self) -> &BackendCapabilities;
    fn create_state(&self, model: &TinyMamba3SisoV0) -> Result<BackendState, BackendError>;
    fn step(
        &self,
        model: &TinyMamba3SisoV0,
        input: &TinyTensor1D,
        state: &mut BackendState,
    ) -> Result<TinyTensor1D, BackendError>;
    fn forward(
        &self,
        model: &TinyMamba3SisoV0,
        input: &[TinyTensor1D],
    ) -> Result<Mamba3SisoForwardResultV0, BackendError>;
}

#[derive(Clone, Debug)]
pub struct CpuMamba3Backend {
    capabilities: BackendCapabilities,
}

impl Default for CpuMamba3Backend {
    fn default() -> Self {
        Self {
            capabilities: cpu_capabilities(),
        }
    }
}

impl Mamba3ExecutionBackend for CpuMamba3Backend {
    fn kind(&self) -> Mamba3BackendKind {
        Mamba3BackendKind::CpuReference
    }
    fn capabilities(&self) -> &BackendCapabilities {
        &self.capabilities
    }
    fn create_state(&self, model: &TinyMamba3SisoV0) -> Result<BackendState, BackendError> {
        Ok(BackendState::Cpu(model.zero_state()?))
    }
    fn step(
        &self,
        model: &TinyMamba3SisoV0,
        input: &TinyTensor1D,
        state: &mut BackendState,
    ) -> Result<TinyTensor1D, BackendError> {
        match state {
            BackendState::Cpu(state) => model.step(input, state).map_err(Into::into),
        }
    }
    fn forward(
        &self,
        model: &TinyMamba3SisoV0,
        input: &[TinyTensor1D],
    ) -> Result<Mamba3SisoForwardResultV0, BackendError> {
        model.forward(input).map_err(Into::into)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComplexTransitionInput {
    pub decay: f32,
    pub cosine: f32,
    pub sine: f32,
    pub previous_real: f32,
    pub previous_imaginary: f32,
    pub current_real_contribution: f32,
    pub current_imaginary_contribution: f32,
    pub trapezoidal_real_contribution: f32,
    pub trapezoidal_imaginary_contribution: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComplexTransitionOutput {
    pub real: f32,
    pub imaginary: f32,
}

pub fn cpu_complex_state_transition(
    input: ComplexTransitionInput,
) -> Result<ComplexTransitionOutput, BackendError> {
    let values = [
        input.decay,
        input.cosine,
        input.sine,
        input.previous_real,
        input.previous_imaginary,
        input.current_real_contribution,
        input.current_imaginary_contribution,
        input.trapezoidal_real_contribution,
        input.trapezoidal_imaginary_contribution,
    ];
    if values.iter().any(|value| !value.is_finite()) {
        return Err(BackendError::InvalidTransitionInput);
    }
    let real = input.decay
        * (input.cosine * input.previous_real - input.sine * input.previous_imaginary)
        + input.current_real_contribution
        + input.trapezoidal_real_contribution;
    let imaginary = input.decay
        * (input.sine * input.previous_real + input.cosine * input.previous_imaginary)
        + input.current_imaginary_contribution
        + input.trapezoidal_imaginary_contribution;
    if !real.is_finite() || !imaginary.is_finite() {
        return Err(BackendError::InvalidTransitionInput);
    }
    Ok(ComplexTransitionOutput { real, imaginary })
}

#[cfg(all(target_os = "macos", feature = "backend-metal"))]
pub use backend_metal::{MetalTransitionParityReport, MetalTransitionPilot};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::tiny_tensor::from_vec_1d;
    use crate::model::{
        Mamba3SisoConfigV0, Mamba3SisoPrecisionV0, Mamba3SisoRopeFractionV0,
        mamba3_siso_params_from_seed_v0,
    };

    fn model() -> TinyMamba3SisoV0 {
        let config = Mamba3SisoConfigV0 {
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
        };
        TinyMamba3SisoV0::new(
            config.clone(),
            mamba3_siso_params_from_seed_v0(&config, 7).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn cpu_backend_matches_direct_model() {
        let model = model();
        let backend = CpuMamba3Backend::default();
        let input = vec![
            from_vec_1d(vec![0.1, -0.2]).unwrap(),
            from_vec_1d(vec![0.3, 0.4]).unwrap(),
        ];
        assert_eq!(
            backend.forward(&model, &input).unwrap(),
            model.forward(&input).unwrap()
        );
        let mut backend_state = backend.create_state(&model).unwrap();
        let mut direct_state = model.zero_state().unwrap();
        assert_eq!(
            backend.step(&model, &input[0], &mut backend_state).unwrap(),
            model.step(&input[0], &mut direct_state).unwrap()
        );
        assert_eq!(backend_state, BackendState::Cpu(direct_state));
    }

    #[test]
    fn selector_never_selects_partial_accelerator_for_full_inference() {
        let cpu = cpu_capabilities();
        let mut metal = unavailable(
            Mamba3BackendKind::Metal,
            BackendReadiness::PartialOperations,
            BackendReasonCode::PartialOperationCoverage,
        );
        metal.supported_operations =
            BackendOperationSet::from_operation(BackendOperation::ComplexStateTransition);
        let probe = StaticBackendCapabilityProbe {
            cpu,
            metal,
            cuda: unavailable(
                Mamba3BackendKind::Cuda,
                BackendReadiness::NotCompiled,
                BackendReasonCode::FeatureNotCompiled,
            ),
        };
        let request = BackendSelectionRequest {
            preference: BackendPreference::Auto,
            fallback_policy: BackendFallbackPolicy::AllowCpuFallback,
            required_operations: BackendOperationSet::FULL_INFERENCE,
            required_precision: ModelPrecision::F32,
        };
        assert_eq!(
            select_mamba3_backend(&probe, request).unwrap().selected,
            Mamba3BackendKind::CpuReference
        );
        let strict = BackendSelectionRequest {
            preference: BackendPreference::Metal,
            fallback_policy: BackendFallbackPolicy::Strict,
            ..request
        };
        assert_eq!(
            select_mamba3_backend(&probe, strict),
            Err(BackendError::StrictBackendUnavailable)
        );
    }

    #[test]
    fn transition_uses_all_supplied_terms() {
        let input = ComplexTransitionInput {
            decay: 0.8,
            cosine: 0.6,
            sine: 0.8,
            previous_real: 1.0,
            previous_imaginary: -0.5,
            current_real_contribution: 0.2,
            current_imaginary_contribution: -0.1,
            trapezoidal_real_contribution: 0.3,
            trapezoidal_imaginary_contribution: 0.4,
        };
        let output = cpu_complex_state_transition(input).unwrap();
        assert_ne!(
            output,
            cpu_complex_state_transition(ComplexTransitionInput {
                trapezoidal_real_contribution: 0.0,
                trapezoidal_imaginary_contribution: 0.0,
                ..input
            })
            .unwrap()
        );
        assert!(
            cpu_complex_state_transition(ComplexTransitionInput {
                sine: f32::NAN,
                ..input
            })
            .is_err()
        );
    }
}
