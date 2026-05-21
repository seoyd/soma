# Official evidence replication

Sprint 39 adds a local-only, research-only replication path that inventories official artifacts, injects conservative committee rows, checks candle coverage, rebuilds references, evaluates sufficiency, and writes a bounded bundle under `target/soma_official_replication/<replication_id>/`.

## Command

```bash
cargo run --bin soma_experiment -- official-replication --config examples/soma_official_replication_aapl_controlled_official.toml
```

## Inputs

- provider readiness / reality reports
- official canonical CSV files
- adjacent `preflight_report.json` and `official_provenance.json`
- optional evidence-lane reports, official committee packs, prior sufficiency artifacts

## Outputs

- `official_replication_report.json`
- `official_replication_report.txt`
- `official_replication_operator_actions.json`
- `official_replication_inventory.json`
- `official_row_injection.json`
- `official_candle_coverage_report.json`
- `official_reference_replication_report.json` when built
- `official_sufficiency_replication_report.json`

## Constraints

- local paths only
- no live execution
- no secret values in output
- controlled, fixture, and research-only evidence never upgrades into official evidence implicitly
