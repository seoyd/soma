# Sanitized Toss Fixtures

These fixtures define Soma's public, fake read-only quote schema. They are not
raw Toss responses and do not reproduce private documentation.

Rules:

- Use fake symbols and values only.
- Never add keys, tokens, authorization headers, account numbers, balances, or
  holdings.
- Convert private local examples into this neutral schema by hand.
- Keep private notes under ignored `local_private/`.
- Order and cancel fixtures are forbidden.

The final field mapping remains deferred until the owner reviews the private
contract locally.
