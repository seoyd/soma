# Residual Integration Suite Migration

Sprint 86 consolidates these residual families into grouped suites:

- official expansion
- committee scenario and replay
- Control Tower briefing
- external prediction
- experiment determinism
- model ops QA
- workspace legacy regression

The migration keeps `tests/external_model_research_ops_cli_safety.rs` separate and records that decision in the migration report instead of forcing it into a broader suite.
