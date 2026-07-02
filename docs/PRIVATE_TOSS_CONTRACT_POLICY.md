# Private Toss Contract Policy

## Local-only material

Private notes may exist only under ignored paths such as:

- `local_private/toss_contract.private.md`
- `local_private/toss_examples.private.json`
- `local_private/toss_field_mapping.private.md`

The repository ignores `local_private/`, `*.private.*`, `secrets/`, and
`credentials/`, `*.key`, and `*.pem`. Private documents, API keys, tokens,
account numbers, real balances, holdings, real order IDs, and raw responses must
never be committed or pasted into public docs, fixtures, logs, reports, or chat.

## Sanitization workflow

1. Review the private contract locally.
2. Copy only neutral field meanings into a new fake object.
3. Replace symbols, timestamps, prices, account references, balances, and
   statuses with fabricated values.
4. Remove headers, tokens, keys, request signatures, and personal data.
5. Store the result under `fixtures/toss/`.
6. Run the fixture safety scanner before review.
7. Compare only schema shape and parser outcome, never raw private content.

Public fixtures document Soma's neutral mapping layer, not the private Toss
contract. Final field-name mapping must be completed through local review.
Only the sanitized schema and fake field mapping may be documented publicly.

## Manual read-only verification

Any future verification must use an explicitly approved, manual-only,
read-only tool. It must redact request and response summaries, avoid raw
response persistence, and stop on every auth, schema, staleness, or quality
error. Unit tests and CI must continue to use `MockTossTransport` only.

Order and cancel endpoints remain deferred because they are outside the
read-only boundary and require a separate security and execution sprint.
