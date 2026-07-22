# Prospective Outcome Opening V0

## Boundary

Prospective outcome opening is a separate, offline-only command. Acquisition
receipts and evidence capsules are inputs; the command cannot construct a
transport or modify the sealed Momentum state, sealed Cycle/Risk state, source
capsule, outcome capsule, Chair state, agent authority, or execution state.

Status and dry-run validate the complete chain without opening outcomes. Local
execution additionally requires the explicit one-time confirmation flag. The
authorization stores only digest-bound identities and counters; it never stores
the owner conversation.

## Preflight and authorization

Preflight reopens and verifies the opening registration, both sealed events,
both maturity plans, shared raw-evidence identity, terminal acquisition
receipt, verified evidence capsule, and every canonical row identity. Evidence
must be complete, finalized, exact, and ready. Both the prior opening-attempt
count and opened-event count must be zero.

The manual-Protobuf authorization binds the registration, acquisition receipt,
evidence capsule, two event identities, and exact row identities. It records
explicit owner authorization and a one-time-only contract. A missing or changed
binding fails closed.

## Atomic objective opening

Momentum and Cycle/Risk are derived independently from their frozen policies.
Momentum uses its registered directional horizon and dead zone. Cycle/Risk uses
its registered future adverse-excursion horizon and the thresholds rebuilt from
the frozen historical tournament. No label policy is substituted between
objectives.

Both results are derived in memory before persistence. Success stores one
manual-Protobuf bundle containing exactly two objective outcomes, their
attributions, a shared receipt, and the attribution journal. If either
derivation fails, the stored terminal receipt contains zero opened outcomes.
There is no partial-success artifact.

## Attribution and reward boundary

The opening path feeds each result into the existing learned prospective event
attribution and reward-eligibility functions. It does not define a parallel
formula. Abstention attribution remains objective-specific and is derived from
the registered event and verified outcome evidence.

Reward handling is compute-only. A sample-gate result and, when the existing
formula permits it, a candidate may be computed. Reward and penalty application,
voice mutation, cooldown, promotion, quarantine, active-model mutation, Chair
decision, vote, and execution remain unavailable.

## Persistence and replay

Authorization and opening bundle are ignored manual-Protobuf artifacts. Writes
use create-new temporary storage, flush, `sync_all`, temporary reopen and
verification, atomic rename, and final reopen and verification. Duplicate or
corrupt artifacts reject. A terminal bundle prevents a repeated execution from
constructing work or opening labels again.

Public text and JSON expose only statuses, semantic identities, attribution
class names, eligibility status, candidate presence, and safety counters. Raw
market values, returns, adverse excursions, numeric labels, probabilities,
private metrics, parameters, and local paths are excluded.
