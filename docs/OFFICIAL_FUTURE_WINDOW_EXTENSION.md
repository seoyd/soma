# Official Future Window Extension

The Sprint 46 extension plan is local-first and bounded.

- Existing local CSVs are reused before any extension job is considered.
- If a CSV exists but ends too early, the planner emits a `LocalCsvWindowExtension` job.
- Provider collection remains disabled by default and is only planned when policy allows it.
- Missing provenance or preflight sidecars are surfaced as skipped jobs plus operator actions, not silently bypassed.
- All paths are local-only and the plan remains research-only, paper-only, and deterministic.
