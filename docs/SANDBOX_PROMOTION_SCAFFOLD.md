# Sandbox Promotion Scaffold

## Purpose

`SandboxPromotionCandidate` records a possible child version for later
evaluation. It does not search parameters, train a model, vote, or replace an
active state.

## Parent And Child

Every candidate records:

- candidate ID,
- agent ID,
- parent version ID,
- candidate version ID,
- sorted and deduplicated feedback IDs,
- proposed policy delta metadata,
- sandbox-only flag,
- promotion status,
- reason codes.

The current scaffold leaves `proposed_policy_delta` empty because policy search
is deferred. Parent-child lineage is still explicit and deterministic.

## Initial Status

A newly built candidate:

- has `sandbox_only = true`,
- starts as `Proposed`,
- cannot affect a live decision,
- cannot be built from non-paper or cross-agent feedback,
- cannot be built for disabled or quarantined agents.

It never starts as `Promoted`.

The builder requires either three distinct valid paper feedback events or a
review trigger: a high-confidence miss, overtrade event, or useful avoided-loss
`NoTrade`. These are review triggers, not promotion criteria. The candidate
remains attached to the updated parent version and cannot alter that version.

## Future Evaluation Sequence

```text
Proposed
-> BacktestPending
-> PaperPending
-> EligibleForOwnerReview
-> Promoted or Rejected
```

Future promotion requires deterministic replay, adequate samples, cost-aware
performance, calibration, doctrine consistency, no safety regression, and Risk
Governor invariant preservation.

Owner review is advisory. It cannot skip backtest/paper stages or force the
`Promoted` status.

## No Live Effect

The candidate type has no conversion into an active vote, Chair input,
`RiskDecision`, `OrderPlan`, or broker call. Its live-effect query always
returns false.

No automatic production replacement, neural training, live policy mutation, or
child-policy search exists.

## Learning Loop Integration

The three-agent paper learning loop may call
`build_sandbox_promotion_candidate` only after finalized feedback has produced
a separate child state. Candidates are returned beside, never in place of, the
updated roster.

The loop never reads candidate metadata during vote, Chair, Risk Governor, or
paper-order phases. `can_vote_live()` and `can_affect_live_decision()` remain
false, and the initial status remains `Proposed`. Doctrine quarantine suppresses
candidate creation rather than creating an unsafe escape path.

Across a learning chain, candidates are accumulated only in the result. The
next episode receives the previous canonical states, not candidate metadata.
Candidate IDs therefore cannot appear in the fixed three-agent vote list, and
their status remains `Proposed` or a later explicit sandbox review state rather
than automatic promotion.

Long replay preserves the same separation. `allow_sandbox_candidates` controls
whether candidate metadata is collected in the replay-level result; it never
converts a candidate into canonical state or gives it cooldown authority.
