# KRX Bounded Collection Smoke

Sprint 50 adds a bounded KRX collection smoke that stays **research-only, market-data-only, local-first, and secret-safe**.

## Dry run vs live collection

- `krx-collection-dry-run` checks env-var presence only.
- `krx-collection-plan` builds deterministic fixture/local/live-disabled jobs.
- live KRX collection stays disabled by default and only becomes runnable when an operator explicitly enables bounded live collection.
- repository tests never execute real KRX network calls.

## Secret safety

- `KRX_API_KEY` is read from env only.
- reports never print secret values.
- `KRX_ENDPOINT_TEMPLATE` previews are redacted.
- raw request metadata is always redacted.

## Bounds

- `max_symbols <= 5`
- `max_rows_per_symbol <= 300`
- `max_requests <= 10`
- `max_days <= 365`
- raw, canonical, and total byte budgets remain bounded

## Scope

The Sprint 50 smoke only covers KRX market-data collection and local replay/import flows. It does **not** add broker, order, account, runtime-LLM, Mamba, or live-trading paths.
