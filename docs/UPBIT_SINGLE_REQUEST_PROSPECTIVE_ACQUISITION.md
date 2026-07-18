# Upbit Single-Request Prospective Acquisition

This protocol is a separate, credential-free public quotation path. It has a
fixed HTTPS `GET` origin and daily-candle path, one `KRW-BTC` candle response,
one in-flight request, zero retries, a fixed timeout, and a bounded response.
It sends only `Accept: application/json`; no authorization, API-key, cookie,
account, trading, redirect, WebSocket, or background-polling capability is
available.

The registration is stored only under ignored local configuration. A dry run
creates and reopens that registration, computes a sanitized fingerprint and
the current UTC midnight-exclusive boundary, and performs no network I/O.
Execution requires the existing network gate plus the dedicated single-request
confirmation. Any transport, timeout, 429, 5xx, parse, or validation outcome
consumes the budget and cannot cause a retry.

Only HTTP 200 JSON arrays containing exactly one finalized daily candle are
eligible. The returned market, UTC candle start, last-trade timestamp,
finite OHLCV values, volume, trade value, and price ordering are checked before
canonical identity creation. Values are retained only in ignored local raw and
capsule artifacts; public status output contains digests and aggregate counts.

After the client returns, the network capsule is reopened and converted into
the pre-registered offline external-row intake. No further network request is
available in admission, independent Momentum/Cycle-Risk validation, event
sealing, maturity, or reward handling.
