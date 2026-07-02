# Secret Safety

## Environment variables

The default names are:

- `TOSS_APP_KEY`
- `TOSS_APP_SECRET`
- `TOSS_ACCOUNT_ID` (optional and sensitive)

Real values must be exported by the operator environment. The project does not
load `.env` because no dotenv dependency is present. `.env` and `.env.*` are
ignored by Git, with only `.env.example` explicitly allowed. That example
contains placeholders only.

Private API notes, secrets, and credentials are also excluded through
`local_private/`, `*.private.*`, `secrets/`, `credentials/`, `*.key`, and
`*.pem`.

## Redaction

`SecretRedactor` masks configured credential values, application key/secret
fields, bearer tokens, authorization headers, sensitive URL query parameters,
and the account ID when account sensitivity is enabled. The module also exposes
`redact_header_value`, `redact_json_like_text`, `redact_url_query`, and
`safe_debug_string`.

Credential fields are private. Credential debug output is always redacted.
Request bodies, response bodies, authorization headers, and account references
are never included raw in debug output. Audit events contain only stable reason
codes, numeric status, safety flags, and redacted endpoint text.

## Logging and test rules

- Never print or serialize `TossCredentials`.
- Never log raw request or response bodies.
- Never include raw headers in audit records or snapshots.
- Never put a real key, secret, bearer token, or account ID in fixtures.
- Unit tests use only local placeholder values and `MockTossTransport`.
- Unit tests must not use the real network.
- Errors expose structured categories, not raw transport response bodies.

If an unrecognized field might contain a secret, omit it from output rather than
attempting partial masking.

## Fixture safety

Public Toss fixtures are fabricated and stored under `fixtures/toss/`. The
fixture scanner rejects configured secret values, authorization text, Bearer
tokens, `app_key`, `app_secret`, `access_token`, `refresh_token`, account/private
markers, the configured private account ID, and obvious long secret-like
values. Private documentation and raw examples must remain local and ignored.

Committed fixture tests inject known values directly into the scanner instead
of reading credentials from the process environment. The environment-aware
entry point remains available for an explicit local review, while deterministic
tests cannot depend on whether an operator has configured credentials.
