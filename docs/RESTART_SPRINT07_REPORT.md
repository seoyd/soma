# Restart Sprint 07 Report

## Canonical Type Decision

`CanonicalAgentState` extends the existing `PersonaCard` family and is the
canonical contract for new paper-learning state work. The active three persona
cards build the three canonical initial states.

## Reused Types

- `ImmutableDoctrine`
- `MutablePolicy`
- `Horizon`
- `PersonaTier`
- `ReasonCode`
- active persona cards
- existing owner rejection template

Offline committee memory, journals, and shadow governance remain adapter or
deferred inputs rather than the new active contract.

## New Types And Functions

Added canonical identity/status/doctrine/voice/memory/version, proposal,
feedback, reward/penalty, and sandbox-candidate types.

Pure functions cover memory aggregation, doctrine detection, Chair
reward/penalty, voice/status update, versioned state update, and deterministic
sandbox metadata.

## Tests

Focused tests were added for the three current states, disabled future
placeholder, paper memory, high-confidence loss, `NoTrade` reward, doctrine
quarantine, state immutability, parent versions, sandbox isolation, non-paper
rejection, and deterministic updates.

Tests were not executed in this implementation pass.

## Verification

No Cargo format/check/test/build command was run. Edited Rust files were parsed
and formatted with standalone `rustfmt`; static boundary searches and
`git diff --check` were used. No post-change compile or test pass is claimed.

## Risks

- Historical persona and offline committee types still overlap.
- No runtime adapter yet converts active votes and outcomes into the canonical
  feedback type.
- New tests require a later Cargo verification pass.
- Policy search and promotion evaluation are deliberately absent.

## Deferred

Model training, Mamba/Gated DeltaNet, online learning, active promotion, future
eight-agent activation, Toss live transport, real broker, order/cancel, runtime
LLM, and live trading remain deferred.

## Next Sprint

Verify the workspace first, then add one narrow adapter from completed paper
outcome attribution into `AgentFeedback`. Do not add model work until that
adapter and version persistence are proven deterministic.
