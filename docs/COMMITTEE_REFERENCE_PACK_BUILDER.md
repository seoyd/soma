# Committee Reference Pack Builder

Sprint 38 adds a deterministic reference-pack builder for committee scenarios.

## Scope
- aligns local committee scenarios to local candle series
- generates triple-barrier outcome references from local candles only
- generates baseline references from existing artifacts or conservative deterministic fallback
- generates no-trade and risk-denied counterfactual references from the same local candle path
- writes deterministic outputs under `target/soma_committee_reference_pack/<reference_pack_id>/`

## Boundaries
- research-only and paper-only
- no live trading, broker, order, or account integration
- no runtime LLM, no Mamba runtime, no persona activation changes
- yfinance references remain research-only
- fixture references remain fixture-only unless a controlled fixture mode is explicitly enabled
- controlled fixture improvement does not equal official readiness

## Official readiness
A generated reference is only counted as official-ready when it is:
- derived from local candles
- no-lookahead safe
- tied to official or real-local scenario provenance
- not marked diagnostic-only

## Primary outputs
- `candle_alignment_report.txt`
- `generated_reference_pack.txt`
- `reference_pack_quality.txt`
- `sufficiency_closure.txt`
- `committee_reference_pack_summary.txt`
