# Sprint 85 Workspace Gate Recovery

Sprint 85 extends Sprint 84 from the narrowed Sprint 82/83 slice to the remaining workspace-wide integration-test surface.

- audits the remaining named bottlenecks (`complete_row_closure_v2`, `artifact_render_cache_plan`, `persona_operational_status`)
- classifies remaining binaries into conservative domain families
- adds grouped domain suites for complete-row closure, artifact rendering, persona operational status, safety guards, CLI safety, and determinism
- preserves a keep-separate path for higher-risk files such as `tests/persona_readiness.rs`
- keeps the final acceptance truth on the full workspace gate

The result remains local-only, deterministic, research-only, paper-only, runtime deferred, and training deferred.

