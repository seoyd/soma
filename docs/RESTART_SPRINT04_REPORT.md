# Restart Sprint 04 Report

## Verification Result

Cargo formatting, workspace checking, and workspace tests were not run because
the owner explicitly requested an implementation-only pass. No green workspace
claim is made. Exact deferred commands and scope are recorded in
`RESTART_SPRINT04_VERIFICATION.md`.

## Fixes

- Isolated the temporary instruction artifact from repository artifacts.
- Split sensitive field rejection from other mapping validation failures.
- Removed environment dependence from committed fixture scanner assertions.
- Covered missing mapped price, missing mapped timestamp, and stale owner input.
- Preserved the existing read-only, paper-only decision path.

## Instruction Boundary

The local instruction artifact is ignored and classified
`INSTRUCTION_ONLY_LOCAL`. Static searches show no source, documentation, test,
Cargo, build-script, include-macro, fixture, audit, or runtime reference.

## Toss Contract Stabilization

Only health and quote read contracts may reach the mock transport. Candle and
account shapes remain disabled pending locally reviewed mappings. Token auth,
order, cancellation, unknown, and mutated non-read-only contracts remain
blocked before transport invocation.

## Field Mapping Status

The default mapping remains explicitly fabricated and neutral. Sensitive field
names return `SensitiveFieldNameRejected`; empty or duplicate mapping keys
return `MappingValidationFailed`. Errors expose categories, not field contents.

## Fixture Safety

All required fabricated fixtures are present. The scanner covers authorization
headers, bearer tokens, sensitive field names, injected known secrets, optional
private account identifiers, and obvious secret-like strings. Deterministic
committed assertions no longer depend on operator environment variables.

## Quote Parser And Data Quality

Price and timestamp are required, price must be positive, numeric values must be
finite, and complete bid/ask pairs must be ordered. Stale data and wide spreads
lower quality; missing fields, malformed input, and invalid bid/ask return
structured failures. Mapping-specific missing-field coverage was added.

## Owner Input Policy

Owner input remains advisory. It cannot force a trade or bypass Chair or
`RiskGovernor`. Stale owner-requested actions now have explicit coverage for
`OwnerRequestedButStaleData`; rejection explanations remain fixed templates
without a runtime LLM.

## Smoke Harness Status

No smoke binary or real transport exists. The future path remains documentation
only, manual only, read-only, disabled by default, excluded from tests and CI,
and guarded against raw output. No smoke command ran.

## Pipeline Status

The reviewed path remains:

```text
MockTossTransport
-> TossClient
-> TossReadOnlyAdapter
-> MarketSnapshot
-> signal path
-> three delegates
-> Chair
-> RiskGovernor
-> PaperBroker
```

The adapter creates data inputs only. It cannot create an `OrderPlan`, call a
broker, or skip decision and risk stages. API failure defaults to `NoTrade`.

## Tests

Assertions were added or strengthened for deterministic known-secret scanning,
mapping reason codes, missing mapped fields, and stale owner rejection. Tests
were not executed in this pass.

## Security Review

No key was requested or added. No private provider document, account value, raw
credential, real network path, real broker, order path, cancellation path, or
runtime LLM path was introduced. Debug and audit boundaries remain redacted and
structured.

## Deferred

Full Cargo verification, real provider schema review, candle/account enabling,
and any manual read-only transport remain deferred. Real orders, cancellation,
live execution, online learning, model/router work, league expansion, web UI,
and deployment remain out of scope.

## Next Sprint

Run the deferred Cargo verification gate first, fix only observed failures, and
review the resulting diff before considering any additional read-only mapping
work.
