# BaselineSignal Real Reduction

Sprint 96 reduces the `BaselineSignal` family conservatively by reusing the grouped suite and reporting exactly what moved and what stayed separate.

- primary grouped suite remains `tests/baseline_signal_suite.rs`
- donor/sentinel files remain explicit when they carry queue-entry or readiness meaning
- assertion migration is conservative: one grouped assertion is represented and two entry/precheck sentinels stay isolated with reasons
- fixture/setup reduction stays minimal and only removes a duplicated output-dir surface
- feature/regime flow, score calculation, and deterministic summary remain preserved

This sprint does **not** claim signal usefulness, profitability, live-readiness, runtime inference, or full workspace acceptance.
