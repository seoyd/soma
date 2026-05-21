# Measured Gate Reruns

Sprint 89 keeps candle-local reruns and real workspace gate attempts separate. A candle no-run rerun is still compile-only interpretation, and a full workspace rerun only counts when `cargo test --workspace --quiet` finishes and passes.

`MeasuredTargetDeltaReportV5` keeps measured data and sample-backed fallbacks distinct so the final report can remain conservative when timing or target-count evidence is incomplete.
