# Momentum T10 Failure Forensics V1

## Purpose

This operation explains the failed Sprint 102 T10 challengers without creating
new model authority. No Sprint 102 challenger passed its frozen development and
validation gate, and no eligible T10 holdout cohort was opened.

The original development and validation receipts remain immutable. An additive
receipt reclassifies both partitions as consumed research-design evidence.

## Sealed evidence split

The existing untouched T10 holdout is reopened as metadata only. The split uses
ordered timestamps, event identities, and the derived parent count:

- the oldest `floor(N / 2)` events become fresh challenger validation;
- the remaining newest events become the final sealed holdout;
- an odd event therefore remains in the final holdout.

The split reads no labels, predictions, metrics, returns, correctness values, or
regime results. Both children remain closed and execution-unauthorized. Their
ordered, disjoint union is validated against the parent boundary, while event
identities and timestamps remain private.

## Registered diagnostics

The failure-forensics registration is persisted before private consumed-event
values are reopened. It binds the frozen Sprint 102 screening registration and
result, both aggregate receipts, development and validation
prediction/evaluation shard digests, all C0–C4 identities, diagnostic policies,
evidence classification, sealed split, and access prohibitions. Private shard
identities remain absent from public JSON.

For C1–C4, the operation records aggregate-only diagnostics:

- Brier delta against C0 across development-derived target-magnitude quintiles;
- correctness, weighted calibration gap, support, and saturation per quintile;
- fixed confidence-distance bands from probability 0.5;
- confidence-band coverage, Brier, C0 Brier, paired Brier delta, correctness,
  calibration, magnitude, and saturation;
- aggregate smallest-magnitude excess-Brier concentration and saturation-event
  concentration;
- one canonical failure disposition per participant.

Magnitude boundaries are derived from consumed development evidence, persisted,
reopened, and only then applied to consumed validation evidence. Fresh
validation and final holdout evidence cannot enter the diagnostics.

## Authority boundary

This is historical, post-screening research-design evidence only. It does not
train a model, create a prediction, evaluate either new sealed partition, alter
the live roster, apply reward or penalty, invoke governance, access the network,
or authorize trading.
