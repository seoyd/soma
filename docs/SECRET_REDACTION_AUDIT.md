# Secret Redaction Audit

Sprint 58 adds a local artifact audit that rejects leaked secret values, token-like values, account-like fields, and order-like fields.

## CLI

```bash
cargo run --quiet --bin soma_experiment -- secret-redaction-audit --config examples/soma_secret_redaction_audit.toml
```

## Inputs

- local artifact paths only
- configured secret env var names

## Outputs

- `secret_redaction_audit.json`
- `secret_redaction_audit.txt`

## Safety

- the audit scans local artifacts only
- broker/account/order fields are treated as unsafe even if they are mock values
