# Sprint 22 report

## Sprint summary

Sprint 22 added a conservative readiness layer on top of Sprint 21 benchmark outputs:

- `OfficialConsistencyReport`
- `Mamba3FinGapAnalysisReport`
- `SequenceDatasetSpec`
- `Mamba3FinCandidateReport`
- `ModelEscalationGateResult`
- `MambaReadinessBenchmarkReport`

It also wired `mamba-readiness --config ...`, added example TOMLs, and added Sprint 22 integration tests.

## Current Mamba3 reflection status

Soma Zero still does **not** implement:

- Mamba3 recurrence
- complex-valued state updates
- MIMO SSM runtime
- Rust-native neural inference
- Rust-native neural training

Sprint 22 only answers whether a later **external-first** prototype is justified.

## Official consistency status

The new consistency layer keeps these outcomes explicit:

- Upbit-only evidence => `CryptoOnly`
- missing KRX auth => `MissingAuth`
- missing AlphaVantage auth => `MissingAuth`
- low outcome count => `InsufficientOutcomes`
- unstable metrics / drawdown spread => `InconsistentMetrics`
- poor calibration or risk instability => blocked escalation states

Mock/non-official readiness does not count as official cross-dataset evidence.

## Gate result

The escalation gate can now recommend:

- improve official data first
- improve signal model first
- improve risk governor first
- build sequence dataset first
- keep baseline + external bridge
- build a **Mamba3Fin external prototype**

It never selects a Rust-native Mamba runtime in this sprint.

## Risk review

- Risk Governor remains absolute veto
- sequence export must remain bounded before any prototype work
- calibration and drawdown stability remain mandatory
- crypto-only evidence can be allowed only with explicit config and is still labeled as crypto-only

## Implemented items

- CLI wiring for `mamba-readiness`
- local-path validation and deterministic report writing
- sequence CSV parsing aligned to real exported `dataset.csv` headers
- conservative crypto-only prototype gating
- docs and example configs for readiness / sequence planning
- Sprint 22 tests covering gap analysis, consistency, escalation, CLI safety, and determinism

## Validation result

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test --workspace --quiet`

All passed.

## Next sprint recommendation

Do **not** implement full Mamba3 next.

Prefer:

1. expand official evidence breadth across KRX and US equity where credentials allow
2. accumulate more bounded benchmark reports across venues
3. strengthen external-model evidence before any later prototype sprint
4. keep any future model work external-first and research-only
