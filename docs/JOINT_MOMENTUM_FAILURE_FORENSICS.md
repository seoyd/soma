# Joint Momentum Failure Forensics

Joint Momentum replay V2 records execution health separately from model-evidence
outcome and anchor-audit availability. It is offline-only and never initializes a
provider, transport, credential, prospective-state, Chair, vote, reward, penalty,
promotion, or execution path.

Each scope/participant trace records deterministic stage status, a sanitized error
code when applicable, reason codes, and an artifact digest. The trace does not
contain OHLCV values, paths, credentials, model parameters, probabilities, or trade
actions. A technical failure maps to `NotEvaluatedTechnicalFailure` and
`ShadowAbstainTechnicalFailure`; it is not represented as a no-signal result.

The legacy V1 replay remains a historical record. V2 reproduces it for forensic
comparison without changing its committed status, then uses the first failed V2
stage to classify the root cause before a corrected replay is considered.
