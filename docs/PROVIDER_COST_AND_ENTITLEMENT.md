# Provider cost and entitlement

Sprint 30 separates auth from entitlement and approval.

## KRX

- Cost tier: `RequiresApproval`
- Interpretation: KRX Open API is not automatically ready just because env vars exist.
- Operator meaning: approval must be granted before claiming Korean official collection readiness.

## AlphaVantage

- Default/free interpretation: compact or EOD/historical research
- Premium interpretation: delayed/realtime requires explicit premium entitlement
- Important rule: compact/free must not be described as realtime

## Alpaca

- Basic/free interpretation: `RealtimeIex` with limited coverage
- Paid interpretation: broader SIP/all-exchange coverage
- Important rule: IEX-only does not imply full US market coverage

## data.go.kr and yfinance

- `data.go.kr`: `FreeWithLimits`, public service-key-based fallback
- `yfinance`: research-only supplemental path, never official readiness

## Safety

- Store env var names only
- Never print secret values
- Missing premium entitlement must be surfaced explicitly

