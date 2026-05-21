# Committee smoke test

Use the committee smoke runner to exercise the three-persona MVP without adding any live path.

## Run

```bash
cargo run --bin soma_experiment -- committee-smoke --config examples/soma_committee_smoke_fixture.toml
cargo run --bin soma_experiment -- persona-cards
```

## Source kinds

- fixture input gives deterministic local smoke coverage
- Upbit smoke remains **crypto-only**
- yfinance smoke remains **research-only**

## Interpretation

- pass means the bounded committee path ran and produced deterministic paper/research output
- it does **not** mean live readiness
- all-no-trade output can still be a valid conservative result

