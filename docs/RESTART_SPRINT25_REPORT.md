# Restart Sprint 25 Report

## Baseline Verification

The starting branch was `main` at `5a68d9d`. Formatting, workspace checking, the full workspace test suite, and diff checks passed. The baseline contained 560 passing tests and two existing unused-private-helper warnings in `persona_card.rs`.

## Extraction Map

`Mamba3Siso*V0`, its recurrence, fixture parser, conformance comparison, and tests moved from `src/league/minimal_ai_committee_core.rs` to `src/model/mamba3.rs`. Generic `TinyTensor1D`, `TinyTensor2D`, deterministic initialization, and small matrix helpers moved to `src/model/tiny_tensor.rs`. The league module keeps compatibility re-exports only; it no longer owns Mamba-3 recurrence code.

## Model Dependency Boundary

The focused model code depends only on the standard library, Serde, Serde JSON, and the generic local tiny-tensor module. It does not import Chair, Risk Governor, PaperBroker, personas, providers, report paths, network clients, or runtime process helpers.

## Official Reference Lock

The reference is the Mamba-3 paper, arXiv:2603.15569, and `state-spaces/mamba` commit `f577286d052741c35d39cd43bdc3fad27120f22c` from `2026-07-07T04:22:25-04:00`. The inspected paths are the Mamba-3 module, Mamba-3 SISO step kernel, Mamba-2 module, Mamba-2 simple module, and minimal SSD module.

## Oracle Generation Status

`tools/mamba3_reference/generate_oracle.py` is developer-only and checks the exact local official checkout, Python dependencies, CUDA availability, official importability, source paths, and commit before creating a fixture. This host has Python 3.9.6 but no PyTorch, and the upstream step API requires the CUDA/CuTe reference environment. Generation is therefore blocked here without fabricating vectors.

## Fixture Cases

The generator emits a tiny deterministic SISO sequence with nonzero B/C normalization, bias, decay, trapezoidal state contribution, and rotation. It retains per-step output and full recurrent state so it can serve both basic and split-stream verification after review.

## Fixture Provenance And Digest

The fixture schema validates architecture, format version, commit, source paths, paper id, Python/PyTorch versions, device, dtype, parameter ordering/count, initial state, shapes, finite values, MIMO exclusion, tolerances, and a digest. Corrupt digests are rejected.

## Conformance Status

The computed status remains `OfficialOracleUnavailable` until a genuine fixture supplies expected output. It changes to output-only or output-and-state matched only after strict comparison. No official numerical claim is made in this sprint.

## Streaming And Stability

The semantic source remains the one-step recurrence. Full forward and streaming use that same update. Existing state reset, validation, non-finite rejection, deterministic parameter, input sensitivity, state sensitivity, and output/state parity checks remain in place.

## Isolation And Hardcoding

No agent, Chair, Risk Governor, PaperBroker, acquisition, provider, network, training, optimizer, MIMO, CUDA, Metal, or runtime Python integration changed. The conformance result is computed from supplied fixture values; there is no fixture-specific inference branch or fixed output path.

## What Was Proven

The SISO reference core is structurally independent of league governance, preserves its internal recurrence tests after extraction, validates fixture provenance/digest, and can compare per-step output and state when an official fixture is available.

## What Remains Unproven

Official numerical parity, training behavior, forecasting quality, trading value, and optimized kernel performance remain unproven. The next step is generating and reviewing the fixture on a prepared upstream CUDA environment.
