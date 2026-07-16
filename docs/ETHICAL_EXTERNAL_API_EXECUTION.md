# Ethical external API execution

The Upbit BTC daily backfill is a manually initiated, one-provider, one-symbol,
read-only research operation. It is disabled unless the local configuration is
valid, explicitly grants manual local network consent, enables the smoke gate,
and the CLI supplies the network flag.

Before a request, the implementation derives the exact additional row count
needed for chronological BTC regimes, the minimum whole-page request count,
and a sanitized plan digest. The derived plan must fit the configured maximum
pages, request count, response-byte bound, wall-clock bound, and positive
minimum inter-request delay. Request execution is sequential; no background
polling, parallelism, symbols, endpoints, account access, orders, or credential
handling are present.

Only transient transport failures may use the configured bounded retry count.
HTTP 429 and HTTP 401/403 are classified separately and stop immediately with
no retry. The implementation never reduces its configured delay dynamically.
Provider-side retry instructions are not consumed because the bounded endpoint
client deliberately does not persist response headers; a later protocol change
must add a validated, capped header contract before it can rely on one.

For strict historical repair, the live policy overrides configured retry
allowance to zero. It derives the request `to` bound only from the oldest
accepted canonical timestamp and rejects every response row at or after that
boundary.

Each accepted page is normalized and verified through the existing acquisition
broker. Pages are chronology-checked, deduplicated only when identical, and
merged deterministically. Existing snapshots are never overwritten: a merged
Protobuf V1 snapshot receives a new semantic digest and identifier and is
written only after readback verification.

For a multi-regime expansion, the next request cursor is the oldest accepted
timestamp of the selected immutable snapshot. This prevents a bounded request
from re-fetching the newest page merely because that is the configured default
cursor. A cached verified page may be merged offline; it never authorizes a
second external request.
