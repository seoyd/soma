# Evidence Store

The evidence store is a local archive for campaign snapshots.

## Snapshot contents

Each `EvidenceSnapshot` records:

- `snapshot_id`
- `campaign_id`
- optional `created_at_ms`
- `input_fingerprint`
- `config_fingerprint`
- `report_fingerprint`
- saved report path
- saved summary path

## Fingerprints

Fingerprints are stable hashes over deterministic text content:

- matrix input content
- campaign config content
- campaign report content

Same input produces the same fingerprint. Different content changes the fingerprint.

## Limitations

- local filesystem only
- no wall clock unless passed in config
- no remote archive backend
- v0 stores campaign snapshots, not a full database
