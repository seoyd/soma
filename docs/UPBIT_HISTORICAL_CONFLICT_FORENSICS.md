# Upbit historical conflict forensics

The rejected local daily-page artifact is inspected offline against the
immutable accepted snapshot. The report exposes counts, canonical field
categories, finality classifications, cursor classes, and deterministic
digests; it never exposes raw candle values or local artifact paths.

Rows with equal timestamps are compared by the canonical identity contract:
symbol, timestamp, IEEE-754 OHLCV bits, and optional trade-value bits. A
conflict is never silently reconciled. Potentially open daily bars are not
treated as finalized provider revisions, and finalized conflicting bars remain
separate evidence until a separately authorized reconciliation policy exists.

The strict backfill cursor is the accepted snapshot's oldest canonical
timestamp, used as the verified exclusive Upbit daily-contract bound. The plan
requires every returned row to be strictly older, computes the request count
from the actual missing evidence rows, allows one sequential request, and uses
zero live retries. Any overlap, range violation, transport failure, permission
failure, or parser failure stops the live phase.

Only a verified non-overlapping older page can be appended to history. The
original snapshot remains byte- and identity-stable; the resulting Protobuf V1
snapshot receives a distinct semantic digest and ID after atomic readback
verification.
