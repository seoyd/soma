# Market Data Collector

Sprint 18 adds a research-only collector that turns official candle API responses into local canonical OHLCV, writes provenance/manifest files, and auto-runs preflight.

## Scope

- market data only
- no broker/account/auth scope
- no trading execution
- local file output only
- offline fixture path for tests

## CLI

```bash
cargo run --bin soma_experiment -- collect-candles \
  --provider upbit \
  --symbol KRW-BTC \
  --timeframe 1m \
  --start 1711929600000 \
  --end 1711953840000 \
  --out data/collected
```

Offline fixture replay:

```bash
cargo run --bin soma_experiment -- collect-candles \
  --provider mock-fixture \
  --symbol KRW-BTC \
  --timeframe 1m \
  --start 1711929600000 \
  --end 1711929840000 \
  --out target/collector_fixture \
  --fixture tests/fixtures/provider/upbit_minutes_fixture.json
```

## Output layout

```text
data/collected/<provider>/<symbol>/<timeframe>/
  raw/
    request_000001.json
  canonical/
    <symbol>_<timeframe>_<start>_<end>.csv
  data_manifest.txt
  data_provenance.txt
  preflight_report.json
  preflight_report.txt
  real_evidence_rerun_plan.json
  real_evidence_rerun_plan.txt
```

## Canonical CSV

Header:

```text
timestamp_ms,open,high,low,close,volume,trade_value,bid,ask,spread_bps
```

Rows are sorted ascending and remain local research artifacts until preflight finishes.
