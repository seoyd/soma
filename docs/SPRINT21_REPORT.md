## 1. Sprint summary

Sprint 21 completed a bounded official-data AI benchmark layer. The repo can now start from official collection coverage, run baseline and optional external prediction evaluation, summarize calibration and Risk Governor behavior, apply usefulness gates, and emit a conservative next-step recommendation.

## 2. Files added

- `src/experiment/ai_benchmark.rs`
- `src/experiment/ai_usefulness.rs`
- `src/experiment/model_gates.rs`
- `src/experiment/risk_ai_interaction.rs`
- `src/experiment/official_coverage.rs`
- `src/experiment/storage_audit.rs`
- `tests/official_ai_benchmark_config.rs`
- `tests/official_ai_benchmark_runner.rs`
- `tests/ai_signal_usefulness.rs`
- `tests/model_usefulness_gates.rs`
- `tests/risk_ai_interaction.rs`
- `tests/official_dataset_coverage.rs`
- `tests/benchmark_storage_audit.rs`
- `tests/ai_benchmark_cli_safety.rs`
- `tests/ai_benchmark_determinism.rs`
- `examples/soma_ai_benchmark_upbit_only.toml`
- `examples/soma_ai_benchmark_official_compact.toml`
- `examples/soma_ai_benchmark_existing_predictions.toml`
- `docs/OFFICIAL_AI_BENCHMARK.md`
- `docs/AI_SIGNAL_USEFULNESS_GATES.md`
- `docs/RISK_AI_INTERACTION.md`
- `docs/SPRINT21_REPORT.md`

## 3. Files changed

- `src/experiment/mod.rs`
- `src/experiment/render.rs`
- `src/bin/soma_experiment.rs`
- `src/lib.rs`
- `src/core/reason.rs`
- `README.md`
- session `plan.md`

## 4. Official AI benchmark config

- added `OfficialAiBenchmarkConfig`
- supports loading an existing collection report or running an official collection plan first
- keeps paths local-only
- baseline evaluation is enabled by default
- Python training is disabled by default

## 5. Dataset coverage report

- added `OfficialDatasetCoverageReport`
- separates crypto / Korean equity / US equity readiness
- tracks missing-auth providers and skipped-budget / failed-preflight counts
- keeps mock-fixture ready entries out of official readiness counts

## 6. Usefulness gates

- added `ModelUsefulnessGateConfig` and `ModelUsefulnessGateResult`
- gates cover schema validity, outcomes, calibration, drawdown, return, profit factor, risk stability, leakage, data quality, and storage budget
- added `AiSignalStatus` and `AiSignalUsefulnessReport`
- status stays conservative: `MissingOfficialData`, `PipelineOnly`, `BaselineEvaluated`, `ExternalModelEvaluated`, `UsefulCandidate`, and failure states are explicit

## 7. Risk AI interaction report

- added `RiskAiInteractionReport`
- reports approvals, risk denials, no-trade counts, emergency stops, cooldowns, avoided losses, missed gains, defensive value, and opportunity cost
- denial is treated as context, not automatically bad

## 8. Benchmark runner

- added `OfficialAiBenchmarkRunner`
- reuses existing official collection and experiment runners
- supports baseline-only mode without Python
- supports optional external prediction evaluation from an existing prediction CSV or Python bridge configuration
- invalid external predictions no longer erase the baseline path

## 9. CLI and examples

- added `ai-benchmark --config ...`
- added `collect-train-evaluate --config ...` alias
- added safe example configs for upbit-only, official compact, and existing-prediction flows

## 10. AI signal status

This sprint adds the **decision framework**, not a production claim. In fixture tests the framework can reach:

- `BaselineEvaluated` for bounded official baseline-only runs
- `ExternalModelEvaluated` when external predictions are valid but not yet promoted
- `UsefulCandidate` in controlled gate-passing fixture cases

That does **not** claim live readiness or real-money readiness.

## 11. Tests added

- config defaults / remote-path rejection / example parsing
- official coverage counting and missing-auth handling
- usefulness gate failures and pass cases
- risk interaction counts and warning behavior
- storage audit byte counting and budget flags
- benchmark runner baseline-only / invalid external / valid external paths
- CLI help and local-only safety
- deterministic benchmark report generation

## 12. Test results

- `cargo fmt --all` passed
- `cargo check --workspace` passed
- `cargo test --workspace --quiet` passed

## 13. Risk review

- no runtime LLM path added
- no live trading, broker, order, or account path added
- provider/auth gaps remain explicit and reason-coded
- Upbit-only evidence remains crypto-only
- missing KRX / AlphaVantage auth blocks overclaiming by coverage report
- storage budget remains audited and bounded
- Risk Governor veto remains final

## 14. Deferred items

- benchmark-triggered ablation execution is still deferred to the existing ablation workflow
- this sprint does not claim generalized all-market readiness from one symbol per venue
- live trading, broker integration, realtime feeds, and persona/model expansion remain deferred

## 15. Next gstack sprint recommendation

Sprint 22 should stay conservative and focus on comparing baseline vs external results across more than one bounded official dataset per venue before any stronger usefulness claim. The next safe step is better coverage breadth and cross-dataset consistency, not live execution.
