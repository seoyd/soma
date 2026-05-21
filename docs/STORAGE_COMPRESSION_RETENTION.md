# Storage compression retention

Sprint 20 hardens storage handling for bounded official-data runs.

## Storage controls

- `StorageBudget` caps total bytes, raw bytes, canonical bytes, manifest bytes, and file count
- `StorageBudgetReport` records measured usage, skipped files, and retention actions
- plan-level row/request caps stop later entries before storage runs away

## Compression policy

Supported policy values:

- `None`
- `GzipRawOnly`
- `GzipCanonicalOnly`
- `GzipRawAndCanonical`
- `ZipBundle`

Current implementation is intentionally conservative:

- non-`None` modes are **reason-coded as deferred**
- bounded uncompressed outputs are kept so preflight and evidence inputs stay usable
- no fake compressed artifact is claimed

## Retention policy

Supported retention modes:

- `KeepLatestOnly`
- `KeepLastNFiles(n)`
- `KeepAllWithinBudget`
- `DeleteRawAfterCanonicalAndManifest`
- `ArchiveCompressedRawOnly`

Current hardening guarantees:

- canonical CSV, manifest, and provenance stay intact for active evidence inputs
- raw archive cleanup is explicit and recorded in `retention_actions`
- cleanup is never silent

## Recommended default

For compact official runs:

- `raw_archive_policy = "CompactJson"`
- `default_retention_policy = "DeleteRawAfterCanonicalAndManifest"`
- compact row limits per entry
- small plan-level byte budget
