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

Session input is now either a verified persisted neutral-plane view or a view
planned from the canonical agent policy and the complete resolved request set.
The prior largest-row-count snapshot shortcut and its replacement one-dataset
policy are not used. Required and optional dataset contracts, allowed markets,
symbols, cadence, lookback, cutoff, staleness, all four policy digests, complete
source identities, private namespace, and training ledger remain bound to the
session identity.

Evidence resolution uses semantic request identity. It verifies market, symbol,
cadence, lookback, cutoff, finality, quality, authorization, ownership,
chronology, duplicate timestamps, finite values, read-only provenance, and
content digests. Missing required evidence blocks only that agent; missing
optional evidence is explicit and never replaced. A unique newer equivalent
artifact resolves deterministically, while an unresolved identity tie fails
closed.

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

## Trainer input projection

The complete view is projected explicitly instead of flattening heterogeneous
artifacts. Momentum consumes one supported chronological price series. The
Cycle/Risk adapter consumes one market-index series, matching its existing
single-series contract. Other authorized evidence stays bound to the source
view as referenced-but-unconsumed until a real adapter exists. Different
symbols, markets, cadences, and dataset kinds are never concatenated into a
synthetic price sequence. Value/Quality still has no projection or trainer.

The projection records the source view, consumed and referenced digests,
primary series, policy identity, and semantic projection identity. It is stored
as a manually defined Protobuf artifact and reopened before acceptance.

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
the artifact. Hardened capability registries, projections, and journals use
digest-addressed paths. Preliminary session and candidate artifacts and their
legacy journal/registry paths remain byte-identical retrospective records and
are superseded for future input binding. The ignored namespace is
`state/learning_data`, divided into agent-owned session, projection, dataset,
candidate, and journal locations.

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

## Candidate evaluation handoff

Private candidates are handed to evaluation through an additive lineage audit;
their original envelopes are not rewritten. The handoff verifies exactly one
candidate, its matching session and dataset manifest, and the trainer projection
before producing an evidence-usage ledger and candidate-identity audit. A
preliminary session without the complete hardened policy and projection binding
is retained as a retrospective record but is ineligible for registration.

Historical training, validation, and test uses are recorded separately. Test
inference, label access, metric calculation, checkpoint influence, and candidate
identity influence are not treated as interchangeable. In particular, a test
metric included in `private_metrics_digest` and then in the candidate digest
makes that historical test consumed for candidate identity; it cannot be called
a fresh sealed test.

The safe continuation is a pre-registered evaluation that accepts only evidence
strictly later than its derived cutoff. Registration does not read future rows,
open labels or probabilities, calculate a performance metric, retrain a model,
replace an active model, promote a candidate, apply a reward, or perform a
network request. Its artifacts live under the separate ignored
`state/learning_data/evaluation/{agent}` namespace and use the same verified,
non-overwriting Protobuf write protocol.
