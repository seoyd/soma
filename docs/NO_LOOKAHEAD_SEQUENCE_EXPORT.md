# No-Lookahead Sequence Export

Sprint 62 reruns no-lookahead checks on the exported rows themselves.

- label timestamp must stay after the feature window end
- split policy remains chronological only
- random split is forbidden
- any export-time leakage blocks the dataset

