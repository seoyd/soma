# Assertion Migration Ledger

Sprint 107 records moved assertions, preserved assertions, source targets, and destination targets explicitly. The retired narrow target in this patch is `tests/shared_fixture_harness_expansion_plan_v2.rs`, and its assertions were moved into `tests/fixture_setup_cost_attribution_v2.rs`.

Retired narrow target semantics are strict: retirement is allowed only after migration or equivalent coverage is recorded, and it is never interpreted as assertion deletion. The ledger keeps assertion deltas visible so hidden drops cannot be normalized away.
