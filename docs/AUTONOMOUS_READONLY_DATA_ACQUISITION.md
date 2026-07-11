# Autonomous Read-Only Data Acquisition

## Logical Autonomy, Centralized Control

Each active agent creates a deterministic data intent from its policy and the
configured runtime universe. It decides required datasets, optional datasets,
market scope, lookback, and freshness tolerance. It does not select URLs,
instantiate an HTTP client, parse provider payloads, or hold credentials.

All physical retrieval passes through `DataAcquisitionBroker`:

```text
agent intent -> acquisition plan -> read-only provider -> normalized snapshot
-> frozen agent evidence bundle -> proposal evidence binding
```

The broker selects only a provider registered with read-only capabilities,
deduplicates matching requests, enforces request and response-size budgets, and
returns structured receipts for failures as well as successes.

## Provider Contract and Modes

`ReadOnlyMarketDataProvider` accepts a provider request without an endpoint or
credential field and returns only normalized canonical rows. Provider
capabilities declare markets, dataset kinds, cadence, lookback, read-only
status, runtime enablement, and network approval.

Supported modes are:

- `Disabled`: no acquisition; evidence remains missing.
- `Mock`: deterministic test adapter only.
- `LocalSnapshotReplay`: reuse verified immutable snapshots without network.
- `ApprovedReadOnlyNetwork`: reserved for an explicitly enabled, approved
  read-only adapter.

No approved network adapter is configured by this sprint. Network mode therefore
fails closed with `NoApprovedReadOnlyProviderConfigured`; no public endpoint is
invented and no synthetic fallback is substituted.

## Snapshots, Provenance, and Freshness

Snapshots contain normalized historical rows plus provider, request, receipt,
schema, digest, range, timestamp, sanitization, and read-only metadata. Raw
provider bodies, Authorization material, account fields, order fields, URLs,
and credentials are rejected before a snapshot is stored.

The in-memory snapshot store verifies its content digest before replay. A frozen
snapshot set is created after acquisition and before proposal evidence binding;
the decision boundary has no provider reference.

Stale data is explicit. Fresh evidence is accepted, a last-known-good snapshot
may be reused only when its configured tolerance permits it, and stale or
missing required evidence produces Abstain or NoTrade-compatible output.

## Replay and Safety Boundary

Manual local CSV and evidence packs remain valid offline replay, audit, and
reproducibility paths. They are not replaced by fabricated acquisition results.

This architecture is paper-only. It adds no broker authentication, order,
cancel, account, transfer, withdrawal, live execution, runtime model, or
background scheduler.
