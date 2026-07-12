# Mamba-3 Official Oracle Pack

This directory is developer-only tooling. Cargo does not invoke it, package it, download dependencies for it, or require Python, PyTorch, CUDA, network access, or this directory for normal Rust builds and tests. The official CUDA oracle validates Rust reference mathematics; it is not a Soma CUDA inference backend.

The pack is pinned to `state-spaces/mamba` commit `f577286d052741c35d39cd43bdc3fad27120f22c`. Before execution it verifies the official origin, exact commit, clean checkout, and SHA-256 values of the Mamba-3 module, CuTe step implementation, normalization implementation, and rotary step implementation. The generator imports `mamba_ssm.modules.mamba3`, constructs `Mamba3`, and invokes its official `step` method. It never imports Rust, reads an existing fixture as output, or implements a duplicate recurrence.

The selected upstream step path documents H100 testing and uses the official CUDA/CuTe stack. The pre-flight intentionally treats a non-H100 device, missing official dependency, or unavailable CuTe route as a blocker. The current development host reaches `PyTorchUnavailable`, so no fixture is generated here.

## Portable invocation

Set only non-secret local paths and selectors:

```bash
export SOMA_MAMBA_OFFICIAL_DIR=/path/to/state-spaces-mamba-checkout
export SOMA_MAMBA_ORACLE_OUT=target/mamba3_oracle
export SOMA_MAMBA_ORACLE_DEVICE=cuda:0
export SOMA_MAMBA_ORACLE_DTYPE=float32
```

Run the machine-readable pre-flight first:

```bash
python3 -B tools/mamba3_reference/verify_environment.py
```

Only `ReadyF32` permits fixture generation. On a prepared H100 environment, generate all focused cases atomically:

```bash
python3 -B tools/mamba3_reference/run_oracle_pack.py --cases A,B,C,D,E
```

The single-case form is also available:

```bash
python3 -B tools/mamba3_reference/generate_oracle.py \
  --reference-root "$SOMA_MAMBA_OFFICIAL_DIR" \
  --device "$SOMA_MAMBA_ORACLE_DEVICE" \
  --dtype "$SOMA_MAMBA_ORACLE_DTYPE" \
  --case E \
  --output "$SOMA_MAMBA_ORACLE_OUT/official_siso_reference_case_e.json"
```

`--overwrite` is required to replace any existing fixture. `--seed` makes the parameter construction explicit and deterministic. The command accepts `--instrumentation-patch` only to reject it deliberately: no patch is required because the official cache and `step` return expose the selected state, and a silent or unverified source modification would invalidate the pinned oracle.

## Case and state contract

- A isolates the trapezoidal recurrence with zero angle projection.
- B uses nonzero angle and recurrent cache state.
- C uses nonuniform B/C inputs with normalization and bias enabled.
- D uses a longer continuous step stream and verifies full-versus-split continuation output and final cache state on the official route.
- E exercises the combined supported SISO configuration.

Each generated fixture is tiny test data. It contains deterministic parameters, input, initial cache, per-step output, per-step cache state, source and generator hashes, environment versions, device class, selected F32 route, and the typed binary FNV-1a digest shared with the Rust parser. State is copied directly from the official angle, SSM, key, and value caches with the batch dimension removed; no Rust-derived state is reconstructed.

Review the provenance and digest before committing a fixture under `tests/fixtures/mamba3/official/`, then run the offline Rust conformance tests. A failed digest or mismatch remains a failure. Do not alter tolerance without an identified official/device reduction-order explanation.
