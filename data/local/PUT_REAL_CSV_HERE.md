# Put real local CSV here

Drop **user-provided local market CSV files** into `data/local/`.

## Supported profiles

- `GenericOhlcv`
- `BinanceKline`-like
- `UpbitCandle`-like
- `KrxOhlcv`-like
- custom column map via Sprint 17 onboarding config

## Minimum guidance

- keep the file local only
- do not use remote URLs
- keep timestamps deterministic and monotonic
- keep OHLC invariants valid
- provide enough rows for walk-forward and triple-barrier checks

## Run preflight

```bash
cargo run --bin soma_experiment -- data-preflight --input data/local/BTCUSDT_1m.csv --out target/soma_data_onboarding --symbol BTC-USDT --timeframe 1m
```

## Run onboarding plan generation

```bash
cargo run --bin soma_experiment -- onboard-data --config examples/soma_data_onboarding.toml
```

Soma does **not** download this data for you, and synthetic/test fixtures still do **not** count as real-market readiness evidence.
