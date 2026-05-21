# Data Provenance

Sprint 16 adds explicit provenance metadata so dataset origin is not guessed from performance.

## DataProvenance fields

- `source_kind`
- `source_label`
- `local_path`
- `generated_by`
- `user_supplied`
- `downloaded_by_soma`
- `remote_url_present`
- `license_note`
- `notes`

## Local-only policy

- remote URL-like paths are rejected
- `downloaded_by_soma` remains `false` in this sprint
- users must place their own CSV files locally

## User-supplied data

Use `data/local/` or another local path you control. Do not rely on downloads or exchange APIs.

## Licensing note

The user is responsible for confirming that any local CSV they provide is legal to use for their workflow.
