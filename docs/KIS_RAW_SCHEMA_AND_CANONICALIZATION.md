# KIS raw schema and canonicalization

- Domestic raw fixtures use `output1` rows with `stck_*`, `acml_*`, `bidp1`, `askp1` fields.
- Overseas raw fixtures use `output1` rows with `xymd`, `open`, `high`, `low`, `clos`, `tvol`, `tamt`, `bid`, `ask`.
- Canonical CSVs use `timestamp_ms,open,high,low,close,volume,trade_value,bid,ask,spread_bps`.
- Sidecars: `_provenance.json`, `_preflight.json`, optional `_manifest.json`.
