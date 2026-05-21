## 1. Sprint summary

Sprint 20 completed the bounded official-data orchestration layer on top of the existing collector. The repo now supports plan-driven official collection, bounded storage accounting, conservative retention/compression handling, and a post-collection official evidence runner that only counts ready entries.

## 2. Files added

- `src/data/official_collection.rs`
- `src/experiment/official_evidence.rs`
- `tests/official_collection_runner.rs`
- `tests/official_evidence_runner.rs`
- `tests/official_collection_cli_safety.rs`
- `examples/soma_official_collection_compact.toml`
- `examples/soma_official_collection_crypto_only.toml`
- `examples/soma_official_collection_equity_compact.toml`
- `examples/soma_official_evidence_run.toml`
- `docs/OFFICIAL_COLLECTION_PLAN.md`
- `docs/STORAGE_COMPRESSION_RETENTION.md`
- `docs/OFFICIAL_EVIDENCE_RUN.md`
- `docs/SPRINT20_REPORT.md`

## 3. Files changed

- `src/data/collector.rs`
- `src/data/mod.rs`
- `src/experiment/mod.rs`
- `src/lib.rs`
- `src/bin/soma_experiment.rs`
- `src/core/reason.rs`
- `README.md`
- `data/collected/README.md`

## 4. Official collection plan

- added `OfficialCollectionPlan` / `OfficialCollectionEntry`
- supports bounded whitelist execution across crypto, KRX, US equity, and fixture-only test flows
- supports `continue_on_missing_auth` and `continue_on_provider_failure`
- writes `official_collection_report.json` and `official_collection_report.txt`

## 5. Storage compression and retention

- added `CompressionPolicy`, `StorageBudget`, and `StorageBudgetReport`
- extended retention with `DeleteRawAfterCanonicalAndManifest` and `ArchiveCompressedRawOnly`
- compression modes are modeled but still conservatively reason-coded as deferred
- retention actions are explicit and never silently remove canonical evidence files

## 6. Collection runner

- added `OfficialCollectionRunner`
- reuses the existing collector instead of creating a second fetch engine
- enforces plan-level row/request/byte limits
- marks entries as `Collected`, `SkippedMissingAuth`, `SkippedBudgetExceeded`, `FailedProvider`, or `DiagnosticOnly`

## 7. Official evidence runner

- added `OfficialEvidenceRunConfig`, `OfficialEvidenceRunner`, and `OfficialEvidenceRunReport`
- consumes collection reports and only executes generated configs for ready entries
- runs real-evidence and optionally batch/ablation
- keeps recommendations conservative when auth is missing or evidence is still too small

## 8. CLI and examples

- added `collect-plan --config ...`
- added `evidence-run --from-collection ... --out ...`
- added `collect-and-evaluate --config ...`
- added compact example configs for mixed, crypto-only, equity-only, and evidence-run flows

## 9. Data-size budget behavior

- plan caps stop later entries before row/request explosion
- storage report tracks raw/canonical/manifest bytes and file counts
- compact collection remains the default
- full-history default remains denied

## 10. Provider/auth status

- `Upbit`: public bounded collection path remains usable
- `KrxOpenApi`: still requires explicit endpoint template and API-key env var for live use
- `AlphaVantage`: still requires explicit API-key env var for live use
- `MockFixture`: deterministic offline path for tests and smoke runs
- `Alpaca`: still stub/deferred

## 11. Tests added

- official collection TOML/example parsing
- missing-auth skip behavior
- retention deleting raw while keeping canonical files
- storage-budget determinism
- official evidence runner config discovery from collection output dirs
- conservative recommendation behavior
- CLI help/local-only/mock-fixture coverage

## 12. Test results

- `cargo fmt --all` passed
- `cargo check --workspace` passed
- `cargo test --workspace --quiet` passed

## 13. Risk review

- no trading, broker, account, or runtime-LLM paths were added
- tests remain fixture/mock only
- missing auth stays reason-coded instead of silently passing
- ready-entry counting stays separate from collected-entry counting
- official data is still gated by preflight before evidence use

## 14. Deferred items

- actual gzip/zip artifact generation is still deferred
- Alpaca live collection remains deferred
- KRX live endpoint specifics are still operator-supplied
- this sprint does not claim strategy readiness or live-trading readiness

## 15. Next gstack sprint recommendation

Keep Sprint 21 focused on turning the official evidence reports into a tighter readiness decision layer: compare official-vs-synthetic evidence directly, surface plan-level coverage gaps, and only then revisit whether more symbols or venues are justified.
