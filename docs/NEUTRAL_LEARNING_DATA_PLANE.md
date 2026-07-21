# Neutral Learning Data Plane V0

## Authority boundary

Learning-data ownership is split by role. Each agent creates its own validated
intent and owns its cutoff, feature policy, label policy, curriculum, private
namespace, and training ledger. The neutral broker validates those intents,
selects a read-only provider, deduplicates semantic requests, validates raw
evidence, stores canonical artifacts, and fans out references. It does not
choose features, labels, trades, winning agents, or Chair conclusions.

Chair has no learning-data permit. The machine-verifiable firewall denies Chair
intent creation, broker invocation, provider selection, view modification,
cutoff changes, label selection, and private-artifact access. Existing Chair
input, voting, and evaluation paths do not carry learning intents or views.

## Intent and acquisition flow

`AgentLearningIntentV0` extends the existing `AgentDataIntent` only with the
learning semantics that were absent: a fixed information cutoff and semantic
digests for source, feature, label, and curriculum policies. Collections are
sorted and deduplicated before the intent digest is calculated. Unknown agents,
markets, datasets, empty required evidence, policy violations, and cutoff
violations fail closed.

The three active agents derive their intents from the existing active-agent
states, configured universe, and `AgentDataPolicy` values. No resulting digest
is embedded in production source. The neutral planning facade converts a valid
learning intent to the existing `AgentDataIntent` without modifying it and then
calls `build_acquisition_plan`.

Deduplication identity includes dataset kind, market, sorted symbols, cadence,
lookback range, and staleness requirement. Agent identity, paths, display data,
and fetch duration are excluded. Existing `requested_by_agents`,
`required_by_agents`, `agent_request_mapping`, and deduplicated-request counts
remain authoritative.

Snapshot replay first uses the exact semantic request key. A non-exact fallback
is admitted only when the snapshot carries explicit compatibility metadata for
cadence, adjustment semantics, normalized source schema, requested cutoff,
staleness contract, and row finality. Dataset kind, market, sorted symbols, and
the complete lookback must also match. The fallback additionally revalidates
chronology, finite OHLCV values, accepted quality and row counts, sanitized
read-only provenance, and the content digest. Legacy snapshots without this
metadata remain available to exact-key replay but fail closed as fallbacks.

## Visibility and independent views

Four explicit classes are enforced:

- `SharedCanonicalRaw` is validated public evidence stored once.
- `AgentAuthorizedRaw` is a reference restricted to its authorized agent.
- `AgentPrivateDerived` is restricted to its owner and never enters another
  agent's view.
- `CommitteeVisibleSummary` cannot be used as agent training input.

`AgentLearningDataViewV0` binds source identities, authorized dataset kinds,
the cutoff, policy identities, private namespace, training ledger, missing
required evidence, and an evidence decision gate. Evidence newer than the
cutoff, unauthorized datasets, cross-agent private artifacts, and invalid
source identities reject. Missing required evidence produces `Abstain` with no
fabricated replacement.

The independence proof derives distinct intent, view, feature-policy,
label-policy, private-namespace, and training-ledger identities for the current
three agents. Shared raw evidence is permitted while private learning state
remains separate.

## Network and evaluation isolation

The optional learning pilot has a separate ignored `state/learning_data`
namespace, a one-request/one-concurrency/zero-retry budget, and
`ResearchOnlyUnconsumed` classification. No non-overlapping safe request was
proven in this run, so the pilot is
`DeferredToProtectProspectiveEvaluation`. No network, credential, label,
training, reward, Chair-decision, vote, voice-change, or execution path ran.

The sealed prospective evaluation lane remains byte-frozen. Learning artifacts
cannot be written into its storage or substituted for its evidence.
