# Sprint 65 Report

## 1. Sprint summary

Implemented a separate, credential-free public Upbit daily-candle path with a
single immutable request budget, then passed its one accepted capsule to the
existing offline external-row admission path.

## 2. Baseline verification

Default and Metal workspace checks and serial workspace tests passed before
the public request phase. Existing warnings were unchanged and unrelated to
this protocol.

## 3. Immutable before-state

The implementation began at commit `1853ac3`. Before acquisition, the ignored
Sprint 64 admission-registration file SHA-256 was
`d59eb7fd451a98c97ea67fd601b3e981fc1706e18d9fd1db91066fe8dff08eea` and the
Momentum local-state file SHA-256 was
`736377e473f595d8d58aa819a4197bf35ab903acc614532bd63f3de90f77c7ad`.

## 4. Public API and HTTP-client audit

The new path uses only the fixed public HTTPS daily-candle endpoint through a
fresh curl invocation. It uses GET, `Accept: application/json`, an HTTPS-only
origin/redirect policy, zero redirects, one timeout, a body cap, and no auth,
cookie, account, order, streaming, or retry option.

## 5. Acquisition registration

The ignored local registration fixes provider, origin, path, market, cadence,
request count, concurrency, retry count, response count, timeout, response
limit, credential-free policy, and legacy-artifact immutability. It was written,
reopened, and validated in dry-run mode after the Phase A push.

## 6. Registration digest

The reopened acquisition registration digest is `85832d4b07bc4f58`.

## 7. Dry-run request fingerprint

The dry run produced fingerprint `63db484f7e7756ed` and UTC-exclusive boundary
`2026-07-18T00:00:00Z` with zero network requests.

## 8. Explicit consent audit

The only attempted request required both the existing network flag and the
dedicated single-request confirmation. Missing or unverified registration and
missing consent are covered by deterministic no-transport tests.

## 9. Actual request count

Exactly one public request was attempted. Subsequent status commands reopen the
stored receipt and do not construct a second transport or request.

## 10. HTTP outcome

The single request returned HTTP status class `2xx`.

## 11. Response validation

The response was JSON, contained exactly one `KRW-BTC` item, passed UTC daily
finality, price ordering, finite OHLCV, nonnegative volume/trade-value, and
last-trade interval validation. No OHLCV values are exposed in this report.

## 12. Acquisition receipt

The sanitized receipt digest is `f5e1a901343473ec`; it records one attempt,
zero retries, one returned item, and `CapsuleCreated`.

## 13. External capsule result

One ignored local network capsule was created with digest
`996eaa6f00679c02`. Its raw response is retained only in ignored local storage.

## 14. Offline admission result

The first offline admission result was `Admitted`. Later status replays report
the same stored row as `DuplicateRow`, which proves the existing capsule cannot
be admitted or requested again.

## 15. Shared raw-evidence reference

The admitted row created exactly one shared raw-evidence reference for the two
independent validations.

## 16. Momentum independent event result

Momentum independently validated the shared reference and sealed one pre-label
event using its frozen artifact boundary.

## 17. Risk independent event result

Cycle/Risk independently validated the same reference and sealed one pre-label
event using its frozen tournament boundary.

## 18. Pre-label isolation

Prospective label reads, mature outcomes, interim metrics, reward candidates,
and reward applications all remained zero. Public output contains no labels or
probability values.

## 19. Maturity and reward eligibility

The events are `AwaitingMaturity`; reward eligibility is
`IneligibleAwaitingMaturity`. No reward or penalty was applied.

## 20. Old-request budget and receipt freeze

The older blind-acquisition receipt and request registry were not read for
mutation, reset, or replacement. The runtime rechecked the old receipt list
after admission and reported it unchanged.

## 21. Network and authority counters

The final status shows one network request total, zero retries, zero label or
reward actions, and zero Chair, vote, promotion, cooldown, quarantine, handoff,
execution, or other authority actions.

## 22. Text/JSON determinism

Text and JSON status replays agree on registration, fingerprint, UTC boundary,
receipt/capsule digests, request count, response count, event counts, maturity,
reward eligibility, and all zero counters.

## 23. Files changed

The implementation changes the existing CLI, Upbit pilot, and data export
surface; it adds only the required acquisition policy and report documents and
updates the two existing prospective-admission documents.

## 24. Complete final verification

After the Phase B status-path adjustment, formatting and both default and
Metal workspace checks passed. Both serial workspace suites passed in full:
the library suites reported 362/363 tests for CPU/Metal, the integration suite
reported 404 tests, and the remaining suites completed with no failures.

## 25. Instruction-file boundary

No implementation, registration, fixture, capsule, report, or documentation
content embeds an instruction file.

## 26. Unrelated-file boundary

Only the scoped source and four permitted documentation paths are staged for
this Sprint; ignored local acquisition artifacts remain outside version control.

## 27. What was proven

One credential-free public request can create at most one finalized capsule,
admit it offline once, share its identity once, and seal independent pre-label
events without labels, metrics, rewards, governance, or execution.

## 28. What remains unproven

No label has matured. This does not prove predictive correctness, prospective
performance, profitability, reward eligibility, Chair learning, voting, or
execution readiness.

## 29. Commit/push result

Phase A was committed and pushed as `d35ccbd` before the request. The final
sanitized report and status-path completion are committed and pushed separately.

## 30. Next Sprint recommendation

Wait for the pre-registered maturity boundary and a separate explicit opening
authorization; do not fetch another candle or evaluate the sealed events early.
