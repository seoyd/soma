# UI Secret Safety

Sprint 52 adds dashboard secret redaction before JSON/TXT/HTML output.

Policy:
- redact or reject `key`, `secret`, `token`, `password`, approval-key, and base-url-like fields,
- redact token-like URL/query content,
- never print secret values from KIS/KRX env configuration,
- never render account/order/balance/holdings data,
- keep outputs local only.

Examples of protected material:
- `KIS_APP_KEY`
- `KIS_APP_SECRET`
- `KIS_WS_APPROVAL_KEY`
- `KRX_API_KEY`
- token-like `KIS_BASE_URL` query content

Only redacted previews may appear in output artifacts.
