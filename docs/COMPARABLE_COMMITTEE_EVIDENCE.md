# Comparable Committee Evidence

Sprint 41 adds normalized comparable rows for committee evidence.

- Rows keep source boundaries: official non-crypto, crypto-only, controlled diagnostic, yfinance research-only, fixture architecture-test-only.
- Completeness is conservative: missing outcome, baseline, NoTrade, RiskDenied, or no-lookahead safety keeps a row incomplete.
- Summary-derived rows are preserved as summary-derived; they are never silently upgraded to row-level evidence.
- Controlled, crypto-only, yfinance, and fixture rows can be included for diagnostics, but they do not become official usefulness claims.

Main command:

```bash
cargo run --bin soma_experiment -- comparable-evidence --config examples/soma_comparable_evidence_official_replication.toml
```
