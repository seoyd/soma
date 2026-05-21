# Control Tower v1

Control Tower v0 was a static dashboard snapshot. Control Tower v1 keeps the same local-only read-only posture, but adds operational state aggregation from current local artifacts, richer KIS readiness/evidence monitoring, next-action planning, and local owner action draft generation.

## Panels
- provider / KIS monitor / evidence / committee / chair / risk
- candidate / paper position / owner / human confirm
- bottleneck / next action / audit timeline / health summary

## Safety
- local-only
- deterministic
- secret-redacted
- paper-only
- no broker, order, account, balance, holdings, position, or execution controls
- dashboard HTML can point to local draft files only; it cannot apply actions
