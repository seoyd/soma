# Restart Sprint 08 Report

## Summary

Sprint 08 connects the existing completed paper outcome to canonical agent
feedback, deterministic Chair updates, parent-linked agent state snapshots, and
an in-memory version journal. It does not add a trading path or model.

## Verification

Cargo formatting, checking, building, and testing were not run during this
implementation session under the owner's explicit no-test instruction. The
latest known full green baseline predates Sprint 07 and Sprint 08. The current
changes therefore remain unverified by Cargo.

## Adapter

`build_agent_feedback_from_paper_outcome` reuses `OutcomeRecord`,
`AgentProposal`, and `AgentFeedback`. It rejects cross-agent, non-paper,
unfinished, and non-finite input. It maps return, adverse excursion,
avoided-loss, missed-gain, confidence, doctrine, data quality, Risk Governor
alignment, and `NoTrade` evidence with stable reason codes.

## Feedback Cycle

`apply_paper_feedback_cycle` performs:

```text
finalized OutcomeRecord
-> AgentFeedback
-> ChairRewardPenalty
-> CanonicalAgentState child version
-> AgentStateSnapshot
-> optional sandbox-only candidate metadata
```

The input state remains unchanged. Doctrine and mutable policy remain equal.
Risk Governor configuration is not accepted or modified by the API.

## Persistence And Journal

`AgentStateJournal` is in-memory only. It appends auditable
`AgentStateSnapshot` values and supports latest, list, count, and version
existence queries. It rejects duplicate versions, non-paper snapshots,
state/metadata mismatches, and live-enabled sandbox snapshots.

No secrets, tokens, provider payloads, private documents, or owner notes are
part of the snapshot schema.

## Sandbox Scaffold

Feedback may create a `SandboxPromotionCandidate` review trigger. The candidate
is always `sandbox_only`, starts `Proposed`, has explicit parent lineage, cannot
vote, and cannot affect a live decision or Risk Governor. Creation requires
three distinct feedback events or a high-confidence miss, overtrade event, or
useful avoided-loss `NoTrade` review trigger.

## Owner Advisory

The owner policy and rejection explanation path are unchanged. Owner input
cannot call the feedback cycle, force a trade, change a Risk Governor result,
or promote sandbox metadata.

## Tests

Focused unit-test code was added for adapter success and rejection, conservative
missing return, `NoTrade` outcomes, high-confidence loss, doctrine violation,
bad data, risk-veto alignment/opposition, deterministic feedback cycles,
immutable input state, child versions, and journal safety. Existing Sprint 07
tests continue to cover memory counters, reward/penalty, cooldown, quarantine,
and sandbox isolation.

The tests were not executed in this session.

## Risks

- The latest code has not been compiled or tested.
- Paper-only status is an explicit trusted context assertion because
  `OutcomeRecord` has no `paper_only` field.
- Missing executed return is conservatively recorded as zero with a reason code.
- The journal is process-local and is not durable across restarts.
- Sandbox criteria are review triggers, not validated promotion criteria.

## Deferred

Real trading, order/cancel operations, real broker integration, live provider
network access, online learning, policy search, durable storage, automatic
promotion, neural training, runtime LLM, heavy models, and activation beyond
the current three agents remain deferred.

## Next Sprint

After full workspace verification, the next sprint should address only
compile/test findings and then add deterministic journal serialization or
replay evaluation behind the same paper-only boundary. Live promotion and real
trading should remain blocked.
