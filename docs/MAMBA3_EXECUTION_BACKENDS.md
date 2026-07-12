# Mamba-3 Execution Backends

The CPU reference backend is the only full-inference backend. It uses the existing Rust SISO model without duplicating recurrence math and is always selected when no accelerator is fully ready.

Backends are selected from compile-time feature availability, runtime capability probing, operation coverage, readiness, requested precision, and fallback policy. `Auto` selects only `FullInferenceReady`; `Strict` returns an error rather than falling back. A partial Metal backend never receives full-model inference.

The optional macOS Metal feature provides a real compute pilot for a paired complex-state transition with decay, rotation, current-input, and trapezoidal contributions. It is `PartialOperations`, not full inference. CPU-Metal parity is internal parity only and does not change official Mamba-3 oracle status.

CUDA is an API and target boundary for Linux/Windows only. No CUDA implementation is claimed, and macOS builds do not depend on CUDA. Backend state is explicit; only canonical CPU state can execute full model operations. The model remains reference-only with no trading integration.

The experimental frozen-Mamba sandbox agent uses the same selector and requires `FullInferenceReady` for inference. On the current platform this selects CPU; partial Metal and unavailable CUDA are never selected for the full encoder. Its head-only training is deliberately CPU-only and does not change accelerator readiness, runtime CUDA status, or the separate official CUDA oracle requirement.
