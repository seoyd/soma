# Agent State Canonicalization

## Decision

The canonical state is `league::CanonicalAgentState`, implemented next to the
existing `PersonaCard` family in `src/league/persona_card.rs`.

This path was selected because the active three-agent paper loop already uses
`PersonaCard`, `ImmutableDoctrine`, `MutablePolicy`, `VoiceConfig`, `Horizon`,
and `PersonaTier`. Canonicalization extends that active family instead of
making the offline committee experiment the production contract.

## Existing Type Inventory

| Concept | Existing types | Classification |
| --- | --- | --- |
| Identity and active style | `PersonaCard`, three concrete persona modules | `KEEP_CANONICAL` |
| Doctrine | `ImmutableDoctrine`, `DoctrineRule`, `DoctrineObservation` | `KEEP_CANONICAL` |
| Mutable policy | `MutablePolicy`, `PersonaMutablePolicy` | `KEEP_CANONICAL` / adapter |
| Vote and proposal | `InvestorVote`, `PersonaVote`, `MemberOpinion` | `ADAPT_TO_CANONICAL` |
| Score and tier | `SurvivalScoreComponents`, `PersonaEvaluationOutput`, `PersonaTier` | `KEEP_CANONICAL` |
| Voice | `VoiceConfig`, `update_voice_power` | `KEEP_CANONICAL` |
| Memory | `MemberMemoryState`, experience records | `ADAPT_TO_CANONICAL` |
| Feedback | `PaperOutcomeFeedback`, evaluation inputs | `ADAPT_TO_CANONICAL` |
| Offline journal | `MemberLearningJournal`, score updates | `ADAPT_TO_CANONICAL` |
| Shadow governance | Chair shadow contracts, ledgers, safety guards | `DEFER` |
| Offline committee member | `AICommitteeMember`, `MemberOpinion` | `LEGACY_DO_NOT_USE` for new active code |
| Chair output | `ChairOutput`, `ChairmanDecision` | active output / offline adapter |
| Risk output | `RiskDecision` | `KEEP_CANONICAL` safety authority |
| Model readiness | Mamba/Gated DeltaNet contracts | `DEFER` |
| Unclassified historical report types | sprint-specific storage/report types | `UNKNOWN_KEEP` |

## Canonical Contract

`CanonicalAgentState` owns:

- stable `AgentId` and one of the three current `AgentKind` values,
- `AgentStatus`,
- `AgentDoctrine`,
- the existing `MutablePolicy`,
- `AgentVoiceState`,
- `AgentMemorySummary`,
- parent-linked `AgentVersion`,
- stable reason codes.

`AgentStateSnapshot` stores the complete parent-linked result of one paper
feedback cycle. `AgentStateJournal` is the canonical in-memory persistence
boundary for this sprint; it rejects non-paper, duplicate, inconsistent, and
unsafe sandbox snapshots.

The three current states are built from `active_persona_cards()`. A future
placeholder is explicitly disabled, sandbox-only, has zero voice, and cannot
vote.

## Adapter Boundary

Existing systems remain intact:

- `PersonaCard` remains the source for doctrine, policy, horizon, tier, and
  initial voice.
- `InvestorVote` remains the active Chair input.
- `RiskDecision` remains the only approval/denial authority.
- Offline committee memory and journals may be converted into feedback later,
  but new active code must not depend directly on the experimental member type.
- Shadow governance remains evidence and candidate metadata; it is not a live
  state mutation API.

No alias is introduced for historical types because an alias would imply
identical semantics where none exist.

## State Transition Boundary

```text
live decision reads CanonicalAgentState vN
-> decision completes
-> finalized OutcomeRecord becomes AgentFeedback
-> pure memory and reward calculation
-> new CanonicalAgentState vN+1 and AgentStateSnapshot
```

The input state is cloned and remains unchanged. A caller must explicitly adopt
the returned version after the feedback phase. No function accepts a broker,
network client, clock, random generator, or mutable live state.

## Non-Goals

- No deletion or migration of historical types.
- No full league refactor.
- No automatic conversion of offline committee output.
- No model, training, parameter search, live promotion, or real trading.
- No activation of future eight-agent placeholders.
