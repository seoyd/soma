# KRX secret-safety isolation

The raw archive sentinel is `tests/krx_raw_archive_secret_safety.rs::archive_redaction_assertions`.

## Separate sentinel policy

Keeping the sentinel separate is acceptable when:

- redaction assertions are still present
- auth/header redaction remains asserted
- secret-like values are still rejected
- the isolated state is reported explicitly

## When isolation is acceptable

Sprint 92 keeps the sentinel isolated because it remains a high-signal secret-safety check. That isolation does not automatically block closure if it is preserved and documented.

## When isolation blocks advancement

If the sentinel is missing, regressed, or merged unsafely, `UnsafeToMerge` is reported and the queue must stay blocked by `KrxEvidence`.
