# Restart Sprint 28 Report

## Baseline Verification

The restart baseline was clean at `e4d1d04`. Default formatting, workspace check, workspace tests, and diff checks passed before this change. The existing two unused-function warnings remain outside this scope.

## Available CUDA Environment

This host is macOS arm64. No NVIDIA CUDA device is available, and its Python 3.9.6 installation cannot import PyTorch. No remote or hosted environment was provisioned.

## Pre-flight Result

The local machine-readable pre-flight result is `PyTorchUnavailable` for requested `cuda:0` and `float32`. It exits nonzero and prevents generation.

## Official Checkout Result

The external official checkout is clean at `f577286d052741c35d39cd43bdc3fad27120f22c`, with the expected `state-spaces/mamba` origin. The pack verifies the pinned hashes for the Mamba-3 module, CuTe step path, BC normalization path, and rotary step path before execution.

## Generator Truth Audit

`generate_oracle.py` imports `mamba_ssm.modules.mamba3`, creates the official `Mamba3` module, assigns deterministic test-only parameters, and calls official `Mamba3.step`. Its local helpers only construct inputs, check shapes, capture returned cache tensors, serialize data, and calculate a fixture digest. It has no Rust import, fixture-output import, or local recurrence fallback.

| Generator output | Official source location | Capture method | Independent recurrence |
| --- | --- | --- | --- |
| output | `Mamba3.step` | returned output tensor | no |
| angle/SSM/key/value state | `Mamba3.step` cache and return values | direct cache copy | no |
| parameters | `Mamba3` fields | explicit validated assignment | no |

## Selected Route

The only route that can close same-dtype conformance is official CUDA F32. The pack requires the upstream-documented H100/CuTe step route. BF16 is identified separately by pre-flight but is not accepted by the current Rust F32 fixture contract.

## Parameter and State Mapping

The generator asserts the SISO projection row count, parameter ordering, and each assigned tensor length. It maps official angle, SSM, key, and value cache tensors by removing only the batch dimension and validates every resulting cache shape.

## Fixture Cases and Provenance

Cases A-E are available through the runner with deterministic seeds. No fixture was generated on this host. A generated F32 fixture is atomically written only after pre-flight, checkout verification, finite-value checks, digest calculation, and temporary-file revalidation.

## Output and State Conformance

No official output or recurrent-state comparison was executed. The current honest status remains `OfficialOracleExecutionBlocked`; output parity and state parity are unproven.

## Numerical Errors and First Divergence

There are no genuine oracle arrays, so no numerical error or first divergence exists to report. No tolerance was changed and no Rust mathematics was modified.

## CPU-Metal and Runtime CUDA

No CPU recurrence correction occurred. The Metal transition pilot remains partial, CPU remains the only full-inference-ready backend, and the runtime CUDA backend remains unavailable and unselected. The developer oracle pack does not alter runtime backend capability.

## Hardcoding and Model Isolation

Production model and backend code contains no fixture-specific output, state, GPU, or readiness branch from this sprint. The reference model remains isolated from agents, Chair, Risk Governor, PaperBroker, acquisition, and trading decisions.

## Test Results

`cargo fmt --all --check`, `cargo check --workspace`, and diff checks passed. Default tests passed 568 tests (152 library, 404 integration, and 12 additional tests). `cargo check --workspace --features backend-metal` passed, and Metal-feature tests passed 569 tests (153 library, 404 integration, and 12 additional tests). The existing two unused-function warnings remain outside this scope. The pre-flight command itself was exercised on the local host and returned the expected blocking status without generating a fixture.

## Input Isolation Audit

The planning input remains ignored and is not referenced by project source, documentation, Cargo configuration, tests, fixtures, or tooling.

## What Was Proven

The portable pack can independently verify the pinned checkout and reject an unsuitable local environment before any fixture write. The existing Rust offline fixture path and conformance statuses remain intact.

## What Remains Unproven

Actual official F32 execution, all A-E fixture provenance, output conformance, recurrent-state conformance, and any first divergence await a prepared H100/CUDA/CuTe environment.

## Next Recommendation

Run the pre-flight and `run_oracle_pack.py --cases A,B,C,D,E` on a developer-controlled H100 environment, review the resulting fixtures, add them under the offline test fixture directory, and then perform the Rust output/state comparison without changing runtime CUDA readiness.
