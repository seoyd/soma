# Tiny Mamba-3 SISO Reference Core

## Purpose

This is a small, portable Rust `f32` reference block for inspecting the Mamba-3 SISO state update. It is CPU-only, sequential, deterministic when parameters are supplied or initialized with an explicit seed, and intentionally not connected to the committee, risk, data-provider, or execution paths.

The executable block is isolated in `src/model/mamba3.rs` and uses generic contiguous storage from `src/model/tiny_tensor.rs`. The legacy `Mamba3TemporalCellV0` remains a separate paper-only simplified committee cell. The `Mamba3Siso*V0` types are the reference path and do not depend on league governance.

## Official Reference Lock

- Paper: *Mamba-3: Improved Sequence Modeling using State Space Principles*, arXiv:2603.15569.
- Implementation repository: `state-spaces/mamba`.
- Examined commit: `f577286d052741c35d39cd43bdc3fad27120f22c`.
- Commit date: `2026-07-07T04:22:25-04:00`.
- Primary implementation mapping: `mamba_ssm/modules/mamba3.py` and `mamba_ssm/ops/triton/mamba3/mamba3_siso_step.py`.
- Differential context: `mamba_ssm/modules/mamba2.py`, `mamba_ssm/modules/mamba2_simple.py`, and `mamba_ssm/modules/ssd_minimal.py`.

The available local host has Python 3.9.6 but no PyTorch installation. The pinned upstream per-step API also documents an H100/CUDA/CuTe implementation requirement. Therefore no official vector was generated on this host.

## Official Execution Route

The pinned checkout is clean at the recorded commit and its official source hashes are checked by the developer generator. No faithful CPU path was found in the inspected Mamba-3 module: the SISO combined and step paths import Triton/CUDA code, while `Mamba3.step` requires the CuTe step function and documents H100 testing. The selected local result is therefore `OfficialOracleExecutionBlocked`, not an output or state parity claim.

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

## Parameter And State Mapping

| Rust field | Official Mamba-3 field | Shape/order |
| --- | --- | --- |
| `input_projection` | `in_proj.weight` | `[projection_rows, d_model]`, row-major |
| `dt_bias` | `dt_bias` | `[nheads]` |
| `b_bias`, `c_bias` | `B_bias`, `C_bias` | `[nheads, d_state]`, SISO rank squeezed |
| `b_norm_scale`, `c_norm_scale` | `B_norm.weight`, `C_norm.weight` | `[d_state]` |
| `skip` | `D` | `[nheads]` |
| `output_projection` | `out_proj.weight` | `[d_model, d_inner]`, row-major |

The official cache maps to Rust state without a batch dimension: `angle_dt_state[0]`, `ssm_state[0]`, `k_state[0, 0]`, and `v_state[0]`. The selected SISO implementation stores real-valued SSM entries; phase is represented by persistent angle pairs used to rotate key/query coordinates. A fixture must never invent a separate imaginary state when the official cache does not expose one.

## Forward And Streaming

`mamba3_siso_forward_v0` begins from `Mamba3SisoStateV0::zero` and invokes `mamba3_siso_step_v0` for every token. Streaming callers retain and mutate their own explicit state. The test suite verifies bit-identical output and final state for both paths under identical inputs and parameters.

## Conformance Method

`Mamba3SisoReferenceFixtureV0` is an explicit JSON format for test/reference arrays, configuration metadata, upstream commit, source paths, Python/PyTorch/device provenance, deterministic parameter ordering, initial state, per-step output/state vectors, tolerances, and an FNV-1a digest. Production math never reads fixture data unless a caller explicitly invokes the import API.

The fixture parser requires a case id, official source SHA-256 values, generator SHA-256, known `float32` dtype, complete state/parameter shapes, and a typed binary FNV-1a digest. It rejects missing provenance, corrupted digests, non-finite values, unsupported MIMO, and invalid source hashes.

No official vector was generated in this environment because the official implementation requires its Python/PyTorch kernel environment. The harness therefore returns `OfficialOracleUnavailable` when expected vectors are absent. It does not fabricate vectors and does not claim official numerical parity. Once vectors are generated externally from the pinned official commit, the same fixture import and comparison path validates output and every stored recurrent state without adding Python to Rust builds or tests. The developer-only generator is `tools/mamba3_reference/generate_oracle.py`.

The tolerance profile has separate output absolute/relative and state absolute limits. On a mismatch, comparison records the first step/index and should be investigated in projection, time step, B/C normalization and bias, rotation, decay, trapezoidal contribution, state, readout, skip, gate, then output-projection order. Tolerances are not changed before that first divergence is explained.

## Numerical Limits

This is a transparent scalar reference, not a high-performance kernel. It rejects non-finite input, parameter, state, and intermediate values rather than silently clamping the recurrence. Configuration is capped by the existing tiny-core allocation limit. Very large finite values can therefore be rejected if their exponential or recurrent update overflows.

## Boundaries

The block is reference-only. It has no HTTP, provider, credential, subprocess, Python, CUDA, Metal, model-download, training, or trading path. It does not vote, alter Chair behavior, alter Risk Governor behavior, or send orders.

## Next Stage

Run the developer-only generator on a clean CUDA environment that satisfies the pinned official Mamba-3 dependencies, review each case A-E fixture, and compare output and state before considering training or performance work.
