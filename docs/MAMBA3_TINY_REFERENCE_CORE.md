# Tiny Mamba-3 SISO Reference Core

## Purpose

This is a small, portable Rust `f32` reference block for inspecting the Mamba-3 SISO state update. It is CPU-only, sequential, deterministic when parameters are supplied or initialized with an explicit seed, and intentionally not connected to the committee, risk, data-provider, or execution paths.

The implementation reuses the repository's existing tiny contiguous tensor storage, shape checks, finite-value checks, serialization derives, and deterministic seed convention. The legacy `Mamba3TemporalCellV0` remains an older paper-only simplified cell; the new `Mamba3Siso*V0` types are the executable reference path.

## Official Reference Lock

- Paper: *Mamba-3: Improved Sequence Modeling using State Space Principles*, arXiv:2603.15569.
- Implementation repository: `state-spaces/mamba`.
- Examined commit: `f577286d052741c35d39cd43bdc3fad27120f22c`.
- Primary implementation mapping: `mamba_ssm/modules/mamba3.py` and `mamba_ssm/ops/triton/mamba3/mamba3_siso_step.py`.
- Differential context: `mamba_ssm/modules/mamba2.py`, `mamba_ssm/modules/mamba2_simple.py`, and `mamba_ssm/modules/ssd_minimal.py`.

The paper is the mathematical reference. The official implementation is the ordering and layout reference. The core follows the official SISO step ordering: projection, BC RMS normalization, B/C bias, angle update and pair rotation, exponential-trapezoidal state update, skip, SiLU gate, and output projection.

The upstream project is Apache-2.0 licensed. This implementation is an independently written, small Rust translation of the documented behavior and does not copy upstream source.

## Implemented Subset

- SISO only, one group, `mimo_rank = 1`.
- Portable CPU `f32` scalar math.
- Input projection with official SISO layout: `z`, `x`, `B`, `C`, `dd_dt`, `dd_A`, `trap`, and angle projection.
- Per-vector BC RMS normalization with independent B/C scale vectors and bias applied after normalization.
- Official heavy-tail transition parameterization, negative transition floor, stable softplus `dt`, exponential decay, and trapezoidal coefficients.
- Paired real rotation of B and C using per-head persistent angle state.
- Persistent SSM state plus previous key and previous value, skip D, SiLU gate, output projection, reset, clone, validation, and deterministic seeded parameter creation.
- Full sequence forward as a loop over the public single-step semantic source of truth.
- JSON-ready typed parameter and state layout, metadata, and an explicit reference fixture format.

## Deferred Subset

- MIMO, multi-group layouts, optimized scans, short convolution, CUDA, Triton, TileLang, CuTe, Metal, SIMD fusion, multithreading, reduced precision, quantization, training, autograd, optimizer, checkpoints, model download, and all trading integration.
- An optional output normalization path is not included because it is separable from the selected SISO recurrence.

## Equation To Code Map

For each head, `mamba3_siso_step_v0` derives:

```text
dt       = softplus(dd_dt + dt_bias)
A        = min(-heavy_tail(dd_A), -a_floor)
alpha    = exp(A * dt)
r        = sigmoid(trap)
beta     = alpha * dt * (1 - r)
gamma    = r * dt
angle'   = wrap(angle + tanh(angle_projection) * dt * pi)
K, Q     = rotate_pairwise(BCNorm(B) + B_bias, BCNorm(C) + C_bias, angle')
S'       = alpha * S + beta * outer(previous_value, previous_key)
                    + gamma * outer(current_value, K)
y        = dot(S', Q) + D * current_value
output   = output_projection(y * silu(z))
```

`S` is stored in contiguous `inner_dim * state_dim` order. The pairwise rotation is a real representation of the complex phase behavior: `(a, b)` becomes `(a cos(theta) - b sin(theta), a sin(theta) + b cos(theta))`.

## Shapes And State

With `d_inner = input_dim * expansion`, `nheads = d_inner / head_dim`, and `nangles` selected from half or all of `state_dim`, the input projection has:

```text
2 * d_inner + 2 * state_dim + 3 * nheads + nangles
```

rows and `input_dim` columns. B/C biases are `[nheads, state_dim]`; their norm scales are `[state_dim]`; `dt_bias` and skip are `[nheads]`; the output projection is `[input_dim, d_inner]`.

`Mamba3SisoStateV0` owns `[nheads, nangles]` angles, `[d_inner, state_dim]` SSM values, `[nheads, state_dim]` previous keys, and `[nheads, head_dim]` previous values. There is no global or hidden state.

## Forward And Streaming

`mamba3_siso_forward_v0` begins from `Mamba3SisoStateV0::zero` and invokes `mamba3_siso_step_v0` for every token. Streaming callers retain and mutate their own explicit state. The test suite verifies bit-identical output and final state for both paths under identical inputs and parameters.

## Conformance Method

`Mamba3SisoReferenceFixtureV0` is an explicit JSON format for test/reference arrays, configuration metadata, upstream commit, output vectors, and a configured tolerance. Production math never reads fixture data unless a caller explicitly invokes the import API.

No official vector was generated in this environment because the official implementation requires its Python/PyTorch kernel environment. The harness therefore returns `OfficialOracleUnavailable` when expected vectors are absent. It does not fabricate vectors and does not claim official numerical parity. Once vectors are generated externally from the pinned official commit, the same fixture import and comparison path can validate them without adding Python to Rust builds or tests.

## Numerical Limits

This is a transparent scalar reference, not a high-performance kernel. It rejects non-finite input, parameter, state, and intermediate values rather than silently clamping the recurrence. Configuration is capped by the existing tiny-core allocation limit. Very large finite values can therefore be rejected if their exponential or recurrent update overflows.

## Boundaries

The block is reference-only. It has no HTTP, provider, credential, subprocess, Python, CUDA, Metal, model-download, training, or trading path. It does not vote, alter Chair behavior, alter Risk Governor behavior, or send orders.

## Next Stage

Generate a tiny official vector fixture from the pinned upstream commit on an explicitly prepared reference environment, compare output and optional per-step state, then consider only evidence-backed portability or performance work.
