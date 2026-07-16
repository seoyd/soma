# Toss local contract intake

Toss historical OHLCV contract material is accepted only through a local,
ignored manifest. The manifest is constrained to one daily OHLCV operation:
transport host and read method, request fields, response field locations,
timestamp and adjustment semantics, pagination, error fields, limits, a
read-only attestation, and credential environment names. It stores no
credential values.

Disclosure is classified as public-safe, local-confidential, redistribution
restricted, or unknown. Unknown and redistribution-restricted material is
fail-closed: it cannot enable an adapter or a provider request. Local
confidential material remains local; the repository records only sanitized
status and a semantic digest prefix.

Korean and US daily capabilities are qualified independently. A qualified
manifest may select only one explicitly preferred capability for a bounded,
read-only pilot. Until that occurs, no request mapping, parser, transport,
fixture, snapshot, or campaign is enabled.
