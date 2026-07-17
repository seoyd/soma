# Prior prospective rejection forensics

The prior Momentum prospective acquisition receipt is immutable evidence.  The
forensic classifier verifies its digest, preserves its one attempted request and
zero retries, and emits only sanitized metadata.

Legacy receipts may record an aggregate provider failure without the pipeline
stage, HTTP class, or parser provenance needed to identify a unique cause.  The
classifier treats that condition as `Unknown` with
`BlockedByUnknownCause`; it does not infer a parser, transport, permission, or
rate-limit diagnosis.  This blocks every new network action.

The classifier is deterministic and records a forensic digest, a sanitized
status class, a first reconstructed stage (`EvidenceInsufficient` when the
receipt cannot support one), and reason codes.  No response body, request URL,
credentials, local path, or price data is emitted.
