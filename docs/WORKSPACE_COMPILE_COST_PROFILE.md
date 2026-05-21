# Workspace Compile Cost Profile

The compile-cost profile in Sprint 106 combines three conservative inputs:

1. cargo JSON capture for no-run artifact/message counts
2. test binary inventory for integration target fanout
3. explosion/ranking attribution for likely high-cost families

Link, macro, fixture, artifact-render, and CLI-smoke costs are treated as diagnostic attribution only. They guide safe consolidation work, but they are not runtime readiness and they are not proof of a measured performance win.
