# UI secret redaction preservation

Sprint 94 keeps dashboard secret redaction preserved across HTML, JSON, TXT, and diagnostic reporting.

- raw secret values never appear in stored outputs
- URL/token material stays redacted
- diagnostic/redaction reports stay secret-safe
- environment key names remain reference-only and never expose values
