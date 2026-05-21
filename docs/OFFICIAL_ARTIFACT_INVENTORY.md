# Official artifact inventory

`official-artifact-inventory` scans configured local paths and classifies readiness, reality, collection, canonical CSV, provenance, preflight, evidence-lane, committee-pack, reference-pack, sufficiency, and coverage artifacts.

## Command

```bash
cargo run --bin soma_experiment -- official-artifact-inventory --config examples/soma_official_artifact_inventory.toml
```

## Notes

- classification is deterministic and conservative
- invalid files become `Unknown`
- missing provenance, preflight, and candle coverage are counted explicitly
- output stays research-only and local-only
