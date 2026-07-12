# Mamba-3 Official Oracle Generator

This directory is developer-only tooling. Cargo does not invoke it, package it, download dependencies for it, or require Python, PyTorch, CUDA, network access, or this directory for normal Rust builds and tests.

The generator locks the official `state-spaces/mamba` checkout to commit `f577286d052741c35d39cd43bdc3fad27120f22c`. It rejects an unexpected origin, dirty checkout, missing source paths, missing dependencies, non-CUDA device, non-finite official output, and accidental fixture overwrite. It records source hashes, generator hash, exact environment versions, state, output, and a typed binary FNV-1a digest shared with the Rust fixture parser.

The pinned upstream `Mamba3.step` path states that it is tested on H100 and requires the official CUDA/CuTe stack. The current development machine does not have PyTorch installed, so fixture generation is intentionally unavailable here.

On a prepared official reference environment, run:

```bash
python3 tools/mamba3_reference/generate_oracle.py \
  --reference-root /path/to/state-spaces-mamba-checkout \
  --case E \
  --output tests/fixtures/mamba3/official_siso_reference_case_e.json
```

Generate cases `A` through `E` separately. Case A disables angle projection to isolate the exponential-trapezoidal update; B enables nonzero phase and state; C uses nonuniform B/C inputs; D supplies a longer streaming sequence; E exercises the combined block. Use `--force` only to intentionally replace an existing fixture.

The generated fixture is test data only. Review its provenance and digest before adding it to the repository, then run the Rust conformance tests. A failed digest or mismatch must remain a failure; do not alter tolerance without recording an upstream/device reduction-order reason.
