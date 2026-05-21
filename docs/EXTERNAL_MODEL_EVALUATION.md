# External Model Evaluation

Sprint 63 evaluates imported external predictions deterministically against the bounded sequence export.

- classification metrics when labels are comparable
- calibration metrics from `p_win`
- ranking metrics from `rank_score`
- return proxy and cost-aware summaries
- risk-aware summaries that preserve RiskDenied / NoTrade semantics

These metrics are offline diagnostics only. They do **not** prove profitability or live readiness.

