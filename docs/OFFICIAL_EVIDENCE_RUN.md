# Official evidence run

Sprint 20 adds a post-collection runner that consumes `official_collection_report.json` and executes generated evidence configs only for ready entries.

## Main types

- `OfficialEvidenceRunConfig`
- `OfficialEvidenceRunner`
- `OfficialEvidenceRunReport`

## Flow

1. read the official collection report
2. keep only entries marked `ready_for_evidence = true`
3. run generated `real-evidence`
4. optionally run generated `batch`
5. optionally run generated `ablation`
6. emit a conservative recommendation

## Conservative rules

- skipped missing-auth entries remain visible
- ready-entry count is enforced separately from collected-entry count
- missing results keep the recommendation at `MissingAuth` or `NeedMoreExperiments`
- synthetic/test fixtures still do not count as real-market readiness evidence

## CLI

```bash
cargo run --bin soma_experiment -- evidence-run --from-collection target/soma_official_collection/sprint20_official_compact/official_collection_report.json --out target/soma_official_collection/sprint20_official_compact/official_evidence_run
```

Combined orchestration:

```bash
cargo run --bin soma_experiment -- collect-and-evaluate --config examples/soma_official_collection_compact.toml
```
