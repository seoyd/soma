# Sprint 46 restart report

The prior cross-regime `DiagnosticFailure` was reproduced as a legacy
aggregation fallback. It represented a valid result shape that the old
aggregator did not classify, rather than a failed campaign, pack, snapshot, or
model replay.

The closure path now seals a deterministic report for each chronological regime
and distinguishes execution health, diagnostic completeness, model evidence,
and Shadow operation. The observed two-regime replay completed technically. In
both regimes the support gate was unavailable, so both reports remain explicit
Shadow abstentions with zero accepted predictive versions. This is incomplete
predictive evidence, not a system failure.

The cross-regime result therefore remains conservative: no useful predictive
signal, promotion, voting, execution, recurrence claim, or live readiness is
created. The replay is offline and preserves immutable Protobuf evidence,
frozen packs, the evidence ledger, and the sealed prospective holdout.
