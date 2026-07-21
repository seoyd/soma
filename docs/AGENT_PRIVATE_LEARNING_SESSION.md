# Agent-Private Learning Session V0

## Scope

The private learning-session boundary is offline and retrospective. It consumes
only immutable local historical snapshots that are explicitly present in an
`AgentLearningDataViewV0`. It cannot call a provider, read prospective labels,
modify an active agent, submit a vote, reward a model, or execute a trade.

The trainer registry is explicit:

- `momentum_trend_fast` uses the existing frozen-Mamba Momentum campaign. The
  encoder stays frozen; only the existing logistic head training path runs.
- `cycle_risk_skeptic` uses the existing independent downside-risk shadow
  learner with its own features, labels, normalizers, candidate identities,
  and journal.
- `value_quality_filter` is `TrainerUnavailable`. No generic substitute is
  created, so its candidate count is zero.

## Evidence and leakage boundary

Evidence resolution compares the view's complete source-digest set with the
provided immutable snapshots. Dataset authorization, ownership, cutoff,
chronology, duplicate timestamps, finite OHLCV values, quality acceptance,
sanitization, read-only provenance, and semantic content digests are checked
again before materialization.

Shared canonical raw evidence may be referenced by multiple agents. Derived
features, labels, normalizers, examples, errors, and trainer state remain in the
owning session. Agent-private derived artifacts are never accepted as another
agent's raw input.

Each materialized manifest records a chronological training range, first purge
gap, validation range, second purge gap, and sealed historical test range. The
normalizer fit range equals training exactly. Validation parameter updates,
test checkpoint selections, prospective row reads, and prospective label reads
must all remain zero. The existing trainers retain their stricter label-horizon
and purge rules.

## Candidate safety contract

A candidate is emitted only when the registered existing trainer produces a
real sandbox model identity from accepted evidence. Every common envelope is
`ShadowOnly`, retrospective research only, and ineligible for the active
committee, promotion, or reward. Candidate metadata has no adapter into Chair,
InvestorVote, Risk Governor, PaperBroker, active persona state, voice power,
tier, speaking rights, or active model versions.

## Protobuf storage

The session manifest, private dataset manifest, candidate envelope, per-agent
journal, and capability registry snapshot use manually defined `prost::Message`
schemas. The semantic digest is calculated from the Rust-domain fields and is
not the hash of encoded bytes.

Writes use a temporary file, flush, `sync_all`, temporary reopen and semantic
verification, atomic rename, then final reopen and verification. A repeated
identical write is explicitly reported as a duplicate instead of overwriting
the artifact. The ignored namespace is `state/learning_data`, divided into
agent-owned session, dataset, candidate, and journal locations.

## CLI

The command is:

```text
--agent-private-learning-sessions --status|--dry-run|--execute-local
--output-format text|json
```

Exactly one mode is required. All modes reject network permission. Public
output is restricted to agent identity, intent/view/session digests, trainer
kind, source count, terminal status, candidate presence/digest, storage counts,
and zero authority counters. It excludes raw rows, private features, labels,
normalizers, weights, gradients, predictions, private metrics, and local paths.

The verified local execution produced independent Momentum and Cycle/Risk
Shadow candidates. Value/Quality remained explicitly unavailable with no
candidate. Repeating the same execution returned deterministic session and
candidate identities and duplicate-rejected all existing artifacts.
