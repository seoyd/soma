# Owner Input Layer

Sprint 53 adds an **Owner Input Layer** for audited supervisory input.

## Owner role

The owner is a **supervisor/operator**, not a voting persona and not a live trading agent.
The owner can:
- add structured notes,
- request reanalysis,
- hold candidates,
- dismiss candidates,
- tighten risk conservatively,
- paper-confirm only when the paper-only protocol allows it.

The owner cannot:
- bypass Chair diagnostics,
- bypass the Risk Governor,
- force approval after `RiskDenied`,
- enable live trading,
- enable broker/order/account APIs,
- convert paper confirmation into a real order.

## Structured input first

`OwnerInput` is the default safe path.
Freeform text is allowed as an audited note, but it does **not** directly change decision state.
Structured payloads are preferred because they are deterministic, auditable, and easy to replay.

## Audit requirements

Every owner input carries:
- `owner_input_id`,
- kind/status/target metadata,
- reason codes,
- deterministic fingerprinting.

Blocked inputs remain visible in audit trails.
Unknown inputs are audited but rejected.
