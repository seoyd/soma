# Toss Read-only Smoke Test

No executable smoke binary is implemented in this sprint. The current project
does not have a reviewed real transport or private field mapping, so documenting
the guard design is safer than adding a partially usable network path.

## Required future guards

- Feature flag: `toss_live_readonly_smoke`
- Environment guard: `SOMA_ALLOW_TOSS_LIVE_READONLY=1`
- Manual local invocation only
- Never executed by unit tests or CI
- Read-only registered contracts only
- No order, cancel, balance mutation, or broker access
- No raw credentials, headers, URLs, responses, account IDs, or balances printed
- Redacted summary output only
- Immediate safe exit on every error

The future manual process would require `TOSS_APP_KEY`, `TOSS_APP_SECRET`, and
the explicit `SOMA_ALLOW_TOSS_LIVE_READONLY=1` guard in the local environment.
An account ID may be configured only for an approved account-read contract.
None of these values may be printed.

Command concept, not currently executable:

```text
cargo run --features toss_live_readonly_smoke --bin toss_readonly_smoke -- --symbol 005930
```

The operator should interpret success only as schema-compatible read-only data.
It is not trading readiness. Any unexpected endpoint, status, field, timestamp,
price, bid/ask, or spread stops the run. Stop by terminating the process; no
background service or persistent retry is permitted.

A safe output may contain only the symbol, timestamp, numeric quality score,
stable reason codes, and success/failure status. To confirm redaction, inspect
that no raw header, token, URL query, response body, account ID, or credential
appears. The current document-only design is disabled by default, absent from
Cargo features, and unreachable from `cargo test`.
