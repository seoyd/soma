# Storage Format Decision

Sprint 78 selects **CSV + JSON manifest now**.

Why now:

- matches the repo's existing deterministic artifact patterns,
- keeps dependencies light,
- keeps test and review cost predictable,
- makes provenance and manifest inspection straightforward.

Why Parquet/Arrow are deferred:

- current sprint freezes contracts, not scale-driven storage optimization,
- larger columnar dependencies would raise compile/test/runtime complexity,
- real dataset scale evidence should justify them before adoption.

