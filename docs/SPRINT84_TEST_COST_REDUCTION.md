# Sprint 84 Test Cost Reduction

Sprint 84 follows Sprint 83 because Sprint 83 diagnosed the long-running full-workspace bottleneck honestly, but did not yet reduce the integration-test binary surface.

- the current bottleneck is `TestBinaryExplosion`, not missing runtime features
- Sprint 84 reduces cost by grouping the narrow Sprint 82/83 integration tests into two suites
- grouped suites preserve assertions, CLI safety checks, and determinism checks
- the full workspace final gate still matters; grouped suites improve iteration but do not replace ship-gate truth
- the result remains local-only, deterministic, research-only, paper-only, and runtime deferred

