use super::{
    BackendCapabilities, BackendError, BackendOperation, BackendOperationSet, BackendReadiness,
    BackendReasonCode, ComplexTransitionInput, ComplexTransitionOutput, Mamba3BackendKind,
    ModelPrecision, cpu_complex_state_transition,
};
use metal::{CompileOptions, Device, MTLResourceOptions, MTLSize};

const TRANSITION_SHADER: &str = r#"
#include <metal_stdlib>
using namespace metal;
kernel void complex_transition(
    constant float& decay [[buffer(0)]],
    constant float& cosine [[buffer(1)]],
    constant float& sine [[buffer(2)]],
    device const float* previous_real [[buffer(3)]],
    device const float* previous_imaginary [[buffer(4)]],
    device const float* current_real [[buffer(5)]],
    device const float* current_imaginary [[buffer(6)]],
    device const float* trapezoidal_real [[buffer(7)]],
    device const float* trapezoidal_imaginary [[buffer(8)]],
    device float* output_real [[buffer(9)]],
    device float* output_imaginary [[buffer(10)]],
    uint index [[thread_position_in_grid]]) {
    output_real[index] = decay * (cosine * previous_real[index] - sine * previous_imaginary[index]) + current_real[index] + trapezoidal_real[index];
    output_imaginary[index] = decay * (sine * previous_real[index] + cosine * previous_imaginary[index]) + current_imaginary[index] + trapezoidal_imaginary[index];
}
"#;

pub fn probe_metal() -> BackendCapabilities {
    match Device::system_default() {
        None => BackendCapabilities {
            kind: Mamba3BackendKind::Metal,
            readiness: BackendReadiness::DeviceUnavailable,
            supported_operations: BackendOperationSet::EMPTY,
            supported_precisions: vec![],
            device_count: 0,
            selected_device: None,
            reason_codes: vec![BackendReasonCode::NoCompatibleDevice],
        },
        Some(device) => {
            let pilot = MetalTransitionPilot { device };
            let readiness = if pilot.pipeline().is_ok() {
                BackendReadiness::PartialOperations
            } else {
                BackendReadiness::RuntimeUnavailable
            };
            BackendCapabilities {
                kind: Mamba3BackendKind::Metal,
                readiness,
                supported_operations: BackendOperationSet::from_operation(
                    BackendOperation::ComplexStateTransition,
                ),
                supported_precisions: vec![ModelPrecision::F32],
                device_count: 1,
                selected_device: None,
                reason_codes: vec![
                    BackendReasonCode::PartialOperationCoverage,
                    BackendReasonCode::BackendParityNotRun,
                ],
            }
        }
    }
}

pub struct MetalTransitionPilot {
    device: Device,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetalTransitionParityReport {
    pub max_absolute_error: f32,
    pub first_failing_index: Option<usize>,
}

impl MetalTransitionPilot {
    pub fn system_default() -> Option<Self> {
        Device::system_default().map(|device| Self { device })
    }

    fn pipeline(&self) -> Result<metal::ComputePipelineState, BackendError> {
        let options = CompileOptions::new();
        let library = self
            .device
            .new_library_with_source(TRANSITION_SHADER, &options)
            .map_err(|_| BackendError::RuntimeUnavailable)?;
        let function = library
            .get_function("complex_transition", None)
            .map_err(|_| BackendError::RuntimeUnavailable)?;
        self.device
            .new_compute_pipeline_state_with_function(&function)
            .map_err(|_| BackendError::RuntimeUnavailable)
    }

    pub fn transition(
        &self,
        input: &[ComplexTransitionInput],
    ) -> Result<Vec<ComplexTransitionOutput>, BackendError> {
        if input.is_empty() {
            return Ok(vec![]);
        }
        if input
            .iter()
            .any(|value| cpu_complex_state_transition(*value).is_err())
        {
            return Err(BackendError::InvalidTransitionInput);
        }
        let pipeline = self.pipeline()?;
        let scalar = |values: Vec<f32>| self.buffer(&values);
        let decay = scalar(input.iter().map(|value| value.decay).collect());
        let cosine = scalar(input.iter().map(|value| value.cosine).collect());
        let sine = scalar(input.iter().map(|value| value.sine).collect());
        let previous_real = scalar(input.iter().map(|value| value.previous_real).collect());
        let previous_imaginary =
            scalar(input.iter().map(|value| value.previous_imaginary).collect());
        let current_real = scalar(
            input
                .iter()
                .map(|value| value.current_real_contribution)
                .collect(),
        );
        let current_imaginary = scalar(
            input
                .iter()
                .map(|value| value.current_imaginary_contribution)
                .collect(),
        );
        let trapezoidal_real = scalar(
            input
                .iter()
                .map(|value| value.trapezoidal_real_contribution)
                .collect(),
        );
        let trapezoidal_imaginary = scalar(
            input
                .iter()
                .map(|value| value.trapezoidal_imaginary_contribution)
                .collect(),
        );
        let output_real = self.device.new_buffer(
            (input.len() * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let output_imaginary = self.device.new_buffer(
            (input.len() * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let queue = self.device.new_command_queue();
        let command = queue.new_command_buffer();
        let encoder = command.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&pipeline);
        for (index, buffer) in [
            &decay,
            &cosine,
            &sine,
            &previous_real,
            &previous_imaginary,
            &current_real,
            &current_imaginary,
            &trapezoidal_real,
            &trapezoidal_imaginary,
            &output_real,
            &output_imaginary,
        ]
        .iter()
        .enumerate()
        {
            encoder.set_buffer(index as u64, Some(buffer), 0);
        }
        let width = pipeline.thread_execution_width().max(1);
        encoder.dispatch_threads(
            MTLSize::new(input.len() as u64, 1, 1),
            MTLSize::new(width, 1, 1),
        );
        encoder.end_encoding();
        command.commit();
        command.wait_until_completed();
        Ok(self.read(&output_real, &output_imaginary, input.len()))
    }

    pub fn parity(
        &self,
        input: &[ComplexTransitionInput],
        tolerance: f32,
    ) -> Result<MetalTransitionParityReport, BackendError> {
        let output = self.transition(input)?;
        let mut max_absolute_error = 0.0_f32;
        let mut first_failing_index = None;
        for (index, (metal, value)) in output.iter().zip(input).enumerate() {
            let cpu = cpu_complex_state_transition(*value)?;
            let error = (metal.real - cpu.real)
                .abs()
                .max((metal.imaginary - cpu.imaginary).abs());
            max_absolute_error = max_absolute_error.max(error);
            if first_failing_index.is_none() && error > tolerance {
                first_failing_index = Some(index);
            }
        }
        Ok(MetalTransitionParityReport {
            max_absolute_error,
            first_failing_index,
        })
    }

    fn buffer(&self, values: &[f32]) -> metal::Buffer {
        let buffer = self.device.new_buffer(
            (values.len() * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        // StorageModeShared provides CPU-visible memory and the allocation is exactly values.len() f32 elements.
        unsafe {
            std::ptr::copy_nonoverlapping(
                values.as_ptr(),
                buffer.contents().cast::<f32>(),
                values.len(),
            );
        }
        buffer
    }

    fn read(
        &self,
        real: &metal::Buffer,
        imaginary: &metal::Buffer,
        len: usize,
    ) -> Vec<ComplexTransitionOutput> {
        // The command buffer has completed and both shared buffers contain len initialized f32 elements.
        let real = unsafe { std::slice::from_raw_parts(real.contents().cast::<f32>(), len) };
        let imaginary =
            unsafe { std::slice::from_raw_parts(imaginary.contents().cast::<f32>(), len) };
        real.iter()
            .zip(imaginary)
            .map(|(real, imaginary)| ComplexTransitionOutput {
                real: *real,
                imaginary: *imaginary,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metal_transition_pilot_matches_cpu_when_a_device_is_available() {
        let Some(pilot) = MetalTransitionPilot::system_default() else {
            return;
        };
        let input = [ComplexTransitionInput {
            decay: 0.8,
            cosine: 0.6,
            sine: 0.8,
            previous_real: 1.0,
            previous_imaginary: -0.5,
            current_real_contribution: 0.2,
            current_imaginary_contribution: -0.1,
            trapezoidal_real_contribution: 0.3,
            trapezoidal_imaginary_contribution: 0.4,
        }];
        let report = pilot.parity(&input, 1e-6).unwrap();
        assert_eq!(report.first_failing_index, None);
    }
}
