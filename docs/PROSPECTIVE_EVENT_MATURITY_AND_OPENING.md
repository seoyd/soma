# Prospective Event Maturity and One-Time Opening

## Scope

This boundary prepares already sealed Momentum and Cycle/Risk prospective
events for a future outcome opening. It does not acquire data, inspect outcome
rows, open a label, calculate a metric, create a reward candidate, or change
any authority state.

## Immutable maturity plans

Each plan is derived from its sealed event identity, prediction timestamp,
maturity timestamp, horizon digest, objective, and frozen label policy. The
required range starts with the first daily row strictly after the prediction
timestamp and ends at the sealed maturity timestamp. The directional Momentum
and downside-risk plans remain separate even when their ranges overlap.

The registration fixes the two plan digests, the shared raw-evidence digest,
the credential-free public BTC daily source policy, finalized-row policy, label
and metric policy digests, and the exact union row count. It permits at most
one future request, one concurrent transport, and zero retries. A range above
the conservative response cap is rejected rather than truncated.

## Readiness

An event is ready only when its wall-clock maturity boundary has passed and
every required daily timestamp is present as a verified finalized row. Empty,
partial, missing, duplicate, non-finalized, wrong-series, out-of-order, or
extra-later evidence cannot open an event. Complete but unverified rows remain
closed.

The only future opening requires the exact registered events and evidence,
zero prior label opens, explicit owner authorization, and one-time-only
authorization. This Sprint defines and validates that authorization shape but
does not create one.

## Offline preflight

The preflight reopens and validates the sealed journals, vaults, admission
chain, and opening registration. It intentionally supplies no outcome rows and
reports readiness plus zero counters for provider calls, transports, consent,
credential access, labels, metrics, rewards, penalties, and authority actions.
It never reports raw candles, probabilities, labels, returns, model parameters,
local paths, or per-event error details.
