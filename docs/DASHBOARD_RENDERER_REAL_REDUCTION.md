# DashboardRenderer real reduction

Sprint 94 reduces DashboardRenderer compile/test surface by moving standalone dashboard snapshot/state/panel assertions into the grouped dashboard suite, preserving the remaining high-risk redaction assertions separately for isolation.

## Assertion migration

- migrated snapshot/state/panel/dashboard-v1 assertions into `tests/dashboard_renderer_suite.rs`
- kept secret redaction isolation in `tests/artifact_rendering_suite.rs`
- never deleted assertions

## Fixture/setup reduction

- reused the grouped suites already present in Sprint 93
- removed duplicate per-file output-dir/setup binaries by deleting the migrated standalone test files
- kept shared fixture harness usage for deterministic comparisons

## Golden output reduction

HTML/JSON/TXT checks remain present in grouped suites, while duplicate dashboard-v1 and snapshot golden coverage is consolidated.

## Deterministic rendering preservation

Dashboard state build, HTML render, JSON render, TXT render, fingerprints, and storage summaries remain deterministic.
