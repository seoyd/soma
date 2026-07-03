# Historical Replay Fixture Schema

## Scope

Historical replay accepts committed, sanitized, local CSV fixture strings only.
The current adapter does not download, request, refresh, or discover market
data.

## CSV Columns

Required:

| Column | Type | Rule |
| --- | --- | --- |
| `symbol` | string | non-empty and identical for all rows |
| `timestamp_ms` | unsigned integer | positive and strictly increasing |
| `open` | number | finite and positive |
| `high` | number | finite, positive, and not below `low` |
| `low` | number | finite and positive |
| `close` | number | finite and positive |
| `volume` | number | finite and non-negative |

Optional:

| Column | Type | Rule |
| --- | --- | --- |
| `trade_value` | number | blank or finite and non-negative |
| `source` | string | blank, `fixture`, `synthetic`, or a namespaced variant |

Unknown or duplicate columns are rejected. This deliberately narrow fixture
format uses comma-separated unquoted scalar fields.

## Validation

The adapter rejects:

- empty datasets,
- excess rows,
- malformed rows,
- NaN or infinity,
- non-positive prices,
- negative volume or trade value,
- `high < low`,
- open or close outside the low/high range in strict mode,
- non-monotonic timestamps,
- mixed symbols or sources,
- non-fixture/non-synthetic sources,
- credential, authorization, token, account, raw provider response, private
  mapping, local-private, environment-file, or temporary-instruction markers.

## Safety Boundary

Fixtures must be synthetic and fake. They must contain no live API data, real
account data, broker data, secrets, private provider mappings, or private
documents.

The adapter performs no live API call and has no broker or account access. Its
paper episodes use the existing Chair and Risk Governor path and represent
counterfactual `NoExecution` outcomes rather than fabricated fills.

## Market Profiles

Korean stock fixtures may use `timestamp_ms` or `date` plus `time`; date/time
is interpreted as synthetic UTC, not an exchange calendar. Optional fields are
`trade_value`, `market`, `source`, and `currency`.

US stock fixtures support the same timestamp alternatives and may add
`adjusted_close`. The adjusted value is validated but never substitutes for
canonical `close`.

BTC fixtures require `timestamp_ms` and may add `quote_volume`, `trade_count`,
`source`, `exchange`, and `currency`. Quote volume may populate canonical trade
value.

Forbidden columns include account, order, credential, authorization, token,
wallet, private, and raw-response fields. Duplicate timestamps and
multi-symbol files are rejected.

## Batch Use

The batch runner applies the same profile and adapter validation independently
to every source. Accepted fixture quality summaries are retained in the source
performance table. Rejected fixtures remain visible with zero replay counts
and stable reason codes.

Multiple accepted fixtures run sequentially through the same three-agent
paper-learning state in sequential mode or reset to the same initial state in
independent mode. Both preserve deterministic behavior without adding network
ingestion, live execution, or a second normalization path.

## Expanded Fixture Pack

The expanded pack contains four 20-row files:

- `expanded_synthetic_mixed.csv`
- `expanded_kr_stock.csv`
- `expanded_us_stock.csv`
- `expanded_btc_crypto.csv`

Every file is fake and uses the existing profile schema. Each provides trend,
range, drawdown or volatility, and recovery segments with varying volume.
These examples are diagnostic fixtures, not real provider exports or evidence
of production data quality.

## Quality Fixture Pack

The V2 quality pack adds five fake 20-row files:

- `quality_clean_synthetic.csv`
- `quality_gap_kr_stock.csv`
- `quality_scale_us_stock.csv`
- `quality_volume_btc_crypto.csv`
- `quality_missing_optional_btc_crypto.csv`

They isolate clean cadence, timestamp gaps, suspicious scale, abnormal volume,
and missing optional fields. Warning fixtures remain syntactically valid so
quality policies can be tested independently of hard parser rejection.
