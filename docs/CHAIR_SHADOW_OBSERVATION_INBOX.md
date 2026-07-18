# Chair Shadow Observation Inbox V0

## Purpose

This inbox exposes completed, historical V3 learned-agent deliberations to a
read-only Chair shadow consumer. It is an observation boundary, not a Chair
runtime entry point. It accepts retrospective development evidence only and
cannot create a vote, Chair input, speaker selection, council score, decision,
size multiplier, risk handoff, order, reward, penalty, or speaking-right
change.

## Source boundary

The only accepted source is the verified V3 replay registration, its completed
scope results, the relationship aggregate, and the replay ledger. Packet
construction recomputes all source links before any inbox mutation:

1. Registration version and digest are verified.
2. Aggregate and ledger are deterministically reconstructed from the V3
   results.
3. Each source-bound opinion and seal is reconstructed and checked against its
   participant record.
4. Deliberations must be exactly two rounds, retrospective-only, and actionless.
5. Packet references are recomputed before an acceptance receipt is issued.

The packet carries only digest references and sanitized identifiers:

- registration, aggregate, ledger, relationship, transcript, opinion, and seal
  digests;
- agent IDs and objective names only in the receipt;
- counts, relationship categories, scope caveats, and uncertainty categories.

It does not serialize raw probabilities, model outputs, vote weights, council
scores, trade proposals, orders, position sizes, or execution parameters.

## Authority

`ChairShadowObservationAuthorityV0` is exact-match policy, not a permissive
set of defaults:

- `advisory_only=true`, `observation_only=true`;
- decision, vote, speaker selection, reward/penalty, speaking-right change,
  risk handoff, and execution are all `false`.

The sole evidence class is `RetrospectiveDevelopmentOnly`; packets must set
`retrospective_only=true` and `prospective=false`. Any prospective claim or
authority relaxation is rejected fail-closed.

## Inbox and receipt

The inbox maintains accepted and rejected packet IDs and invariant action
counters. All of these counters remain zero: Chair runtime invocations,
decisions, votes, rewards, penalties, speaking-right changes, risk handoffs,
and executions.

Receipts use one of the following outcomes:

- `AcceptedRetrospectiveObservationOnly`
- `InvalidRegistration`, `InvalidLedger`, `InvalidAggregate`
- `InvalidOpinionSeal`, `InvalidTranscript`
- `AuthorityViolation`, `ProspectiveClaimForbidden`, `DuplicatePacket`
- `TechnicalFailure`

The receipt also records source aggregate and ledger digests, observed counts,
relationship-category totals, scope caveats, uncertainty flags, and the same
zero action counters.

## Decision firewall

`ChairShadowDecisionFirewallProofV0` proves the packet cannot become a vote or
Chair input. It also records that the Chair engine, speaker selection, council
score, decision, size multiplier, risk handoff, and execution were not
invoked. The proof contains no runtime adapter because the observation module
does not import Chair runtime types.

## Local storage

Accepted packet, receipt, and firewall proof are stored in an ignored local
JSON file. Writes are append-only for distinct packets and use temporary-file
rename for atomic replacement. Reopening revalidates every digest and all
zero-action invariants. The storage digest is semantic and excludes its file
path; an already stored identical packet is idempotent, while a conflicting
duplicate fails closed.

## Offline CLI

Use `--chair-shadow-observation-inbox` with a local historical snapshot
campaign configuration. The CLI is offline-only and reports the acceptance
status, evidence class, counts, relationship summary, uncertainty flags,
inbox/receipt/firewall digests, storage reopen result, and zero action/network
counters in both text and JSON modes.
