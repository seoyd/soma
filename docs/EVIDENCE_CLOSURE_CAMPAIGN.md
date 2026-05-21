# Evidence Closure Campaign

Sprint 15 exists to close the minimum evidence gap identified in Sprint 14 without expanding personas, touching live trading paths, or overclaiming synthetic data.

## Targets

The closure campaign must add:

1. `+1` usable dataset
2. `+20` additional outcome records
3. `+2` additional comparable ablation variants

## What was added

- `tests/fixtures/market_data/generic_ohlcv_valid_alt.csv`
- `examples/soma_evidence_closure_matrix.toml`
- `examples/soma_evidence_closure.toml`
- `soma-experiment evidence-close --config ...`

The example closure matrix stays local-only, uses `BaselineOnly`, and reuses Sprint 13 ablation dimensions.

## Definitions used in Sprint 15

### Usable dataset

A dataset counts only when it is local, validated, and ends with `Good` or `Warning` quality under the closure policy. Synthetic fixtures may count for pipeline-completeness only when explicitly allowed and tagged.

### Outcome record

Sprint 15 counts additional outcome records from a valid campaign run. They must come from the deterministic experiment path and remain cost-aware. No live or broker path is involved.

### Comparable variant

A variant counts only when:

- the baseline run exists,
- the variant run exists,
- dataset scope is compatible with the baseline,
- the variant run did not fail,
- the variant is not flagged `NotComparable`,
- and both sides have enough outcome coverage for comparison.

## Synthetic fixture limitation

`generic_ohlcv_valid_alt.csv` is intentionally synthetic and deterministic. It closes evidence coverage for local research plumbing, but it does **not** prove market edge, live robustness, or real-money readiness.

## Result from the example closure run

Running:

```bash
cargo run --bin soma_experiment -- evidence-close --config examples/soma_evidence_closure.toml
```

produces:

- `+1` usable dataset (`valid_alt_fixture`)
- `+32` additional outcome records
- `+2` additional comparable variants

The minimum evidence gap is closed, but the final recommendation still remains conservative: `NeedMoreExperiments`.
