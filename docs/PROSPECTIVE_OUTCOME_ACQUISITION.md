# Prospective Outcome Acquisition

## Scope

This protocol executes the outcome-row request that was fixed by the existing
one-time opening registration. It does not replace that registration, acquire a
partial horizon, open a label, compute a metric, or create a reward signal.

## Readiness

Readiness is recalculated from the reopened registration, both immutable
maturity plans, the verified sealed-event chain, the current UTC timestamp, and
any existing request receipt or outcome capsule. A daily row becomes eligible
only at the UTC midnight after that row's registered timestamp. Consequently,
the shared request remains blocked until the last Cycle/Risk row is finalized,
even after its candle start timestamp has arrived.

Registration or event-integrity failure, an existing request attempt, and an
existing evidence capsule all fail closed before transport construction.

## Exact request plan

The plan uses the sorted union of the two maturity-plan timestamp sets. The
query count equals that union size, and the exclusive `to` boundary is the UTC
midnight immediately following the last required timestamp. Provider, market,
cadence, one-request budget, one-request concurrency, and zero-retry policy are
validated against the already registered public Upbit source chain.

No moving lookback, current-date boundary, larger convenience count, per-agent
request, or partial timestamp selection is accepted. The dry-run exposes only
the registered range metadata, plan digest, and sanitized request fingerprint.

## Network and response contract

Future execution uses one credential-free HTTPS `GET` to the fixed daily-candle
endpoint. Curl is constrained to HTTPS, zero redirects, a fixed connection and
total timeout, a bounded response, one invocation, and no retry. It sends an
`Accept: application/json` header and has no authorization or cookie surface.

Any timeout, transport failure, non-200 response, oversized body, parse error,
or validation failure consumes the one request budget. The response must have
exactly the registered item count and timestamp set. Duplicate, missing, extra,
wrong-market, non-finalized, non-finite, negative-volume/trade-value, and invalid
OHLC relationships are rejected. Identity is verified before newest-first rows
are normalized into chronological order.

## Receipt and capsule

Status and dry-run never create a receipt. A receipt is created only after a
real transport attempt and records one request, zero retries, sanitized status,
returned/verified counts, and the optional capsule digest.

A successful response creates one ignored local evidence capsule containing
the exact canonical rows and their digests. The capsule must be complete,
finalized, read-only, sanitized, credential-free, and sealed with
`labels_opened = false`. Acquisition leaves opening as a separate explicitly
authorized operation and performs no label, return, adverse-excursion, Brier,
AUC, calibration, correctness, reward, penalty, Chair, vote, or execution work.

## CLI

Use `--prospective-outcome-acquisition` with exactly one of `--status`,
`--dry-run`, or `--execute`. Status and dry-run are offline-only. Execute can
construct transport only when readiness is explicit-request ready, the request
and evidence stores are empty, `--allow-network` is present, and
`--confirm-one-time-outcome-request` is present.
