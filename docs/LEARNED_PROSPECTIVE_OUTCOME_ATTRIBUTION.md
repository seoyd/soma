# Learned Prospective Outcome Attribution

## Purpose

This contract registers how a sealed learned-agent prediction may later become
eligible for an existing bounded Chair reward or penalty computation. It is an
offline, shadow-only boundary. Registration is not a reward, a Chair decision,
a voice change, or a deployment signal.

## Registration

`LearnedRewardEligibilityRegistrationV0` binds the sealed Momentum challenge
and sealed Cycle/Risk tournament to attribution, maturity, sample-gate,
objective-mapping, and integrity policy digests. Its digest also commits to all
of the following prohibitions:

- retrospective evidence, owner input, and interim metrics are forbidden;
- a finalized outcome and one-time label opening are required;
- reward application, voice mutation, cooldown mutation, and promotion mutation
  are forbidden by this bridge.

The registration accepts only the exact Momentum shadow agent for directional
Momentum and the exact Cycle/Risk shadow agent for downside risk. Each contract
is sealed and shadow-only, has a nonzero information cutoff, and identifies the
challenge, frozen model artifact, and prediction horizon.

## Event and Opening Rules

An attribution contains exactly one prospective event, agent, objective,
sealed opinion identity, challenge, frozen model, raw-evidence digest, and
horizon. An event must be later than its contract cutoff. Duplicate event IDs,
missing opinion seals, mismatched objectives, agents, challenges, models, or
horizons are rejected.

Labels remain unavailable until required finalized rows, identity checks,
challenge validity, maturity, and explicit authorization are all present.
Early access and duplicate opening are invalid states. Historical replay,
Sprint 61/62 observation records, owner advisory records, and interim metrics
cannot be converted into prospective events.

## Objective Separation and Abstention

Momentum and Cycle/Risk outcome payloads are distinct types. They do not share
labels or allow a directional outcome to stand in for a downside-risk outcome.
Both preserve the following explicit abstention classifications:

- `JustifiedCapitalProtection`
- `CorrectUncertainty`
- `MissedMaterialOpportunity`
- `FailedToWarnMaterialRisk`
- `NeutralUninformative`
- `NotYetEvaluable`

The status is derived from ledger evidence, never hard-coded for the current
state. With no registered prospective events it is
`IneligibleNoProspectiveOutcomes`.
