# Sprint 64 Report

## Implemented items

- external model artifact registry
- model-card lineage
- prediction artifact lineage
- external evaluation history
- calibration drift
- external model version comparison
- conservative external leaderboard
- leaderboard promotion policy
- previous external comparison
- registry audit
- static external leaderboard panel
- Sprint 64 CLI/example fixtures

## Tests

- focused Sprint 64 integration tests for registry, lineage, history, drift, leaderboard, policy, previous comparison, audit, CLI safety, and determinism

## Registry status

- example registry fixture finishes `RegistryReadyWithWarnings`
- incompatible or missing artifacts remain conservative blockers instead of silent fallbacks

## Evaluation history

- example history keeps `ext-model-a` latest/previous tracking
- single-version models remain `NeedMoreVersions`

## Drift status

- example fixture reports stable drift for the tracked multi-version model
- insufficient history remains explicit for single-version models

## Leaderboard status

- example fixture produces a conservative offline leaderboard with two eligible entries and one blocked entry

## Mamba family status

- Mamba3Fin-lite remains contract-only
- runtime remains deferred

## Risk review

- no live trading
- no broker/order/account APIs
- no runtime LLM decision path
- no Mamba runtime
- no training path

## Next sprint recommendation

- keep building conservative offline artifact comparison and Control Tower visibility
- do not move to runtime Mamba or live promotion

