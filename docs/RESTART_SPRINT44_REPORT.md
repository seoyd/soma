# Sprint 44 restart report

This report is completed only after the bounded execution path has passed its
targeted tests and the sanitized CLI report has been produced. It records no
raw candles, local configuration values, URLs, headers, credentials, or local
snapshot paths.

## Execution record

The bounded dry run found 200 accepted rows and a 312-row two-regime
requirement. It required 112 additional rows, one minimum whole-page request,
and one configured maximum request. The local preflight was ready only for the
explicitly authorized execution; all later verification ran without network
consent.

One sequential public daily BTC request was made. It returned one verified
200-row page. No delay was needed before a second request because none was
made. No rate-limit, permission, retry, concurrency, credential, account, or
trading action occurred.

The cached page was then considered for offline deterministic merge. Its
overlapping timestamp values conflicted with the immutable accepted snapshot,
so the merge was rejected rather than overwriting, deduplicating, or inventing
data. The original snapshot remained valid and its prior temporal-report
digest reproduced. Consequently no expanded snapshot, multi-regime pack,
campaign result, accepted predictive version, or future holdout was created.

The implementation now derives the next backfill cursor from the oldest
accepted timestamp, so a future separately authorized bounded attempt targets
older history rather than re-requesting the newest configured page.
