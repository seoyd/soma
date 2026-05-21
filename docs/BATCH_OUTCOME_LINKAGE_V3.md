# Batch Outcome Linkage V3

Sprint 47 batch-links multiple eligible official rows through the existing outcome linkage v3 logic.

## Behavior
- filters rows deterministically
- respects exact symbol / horizon / no-lookahead rules from the underlying runner
- reuses local candle coverage artifacts only
- emits flattened counters for generated/official outcomes, skipped reasons, label counts, and a batch linkage status

## CLI
`cargo run --bin soma_experiment -- batch-outcome-linkage-v3 --config examples/soma_batch_outcome_linkage_v3_multi_row.toml`
