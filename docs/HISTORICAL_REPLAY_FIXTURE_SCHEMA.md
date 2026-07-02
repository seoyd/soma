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
