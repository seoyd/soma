# Sprint 12 Report

## Implemented items

- research campaign config and runner
- campaign aggregate
- evidence store and snapshot fingerprints
- campaign diff and regression guard
- hardened campaign readiness report
- deterministic campaign/diff/readiness rendering
- `soma-experiment campaign` and `soma-experiment compare`
- example campaign configs

## Tests

- campaign config
- campaign runner
- evidence store
- report diff
- regression guard
- readiness hardening
- campaign determinism
- campaign CLI safety

## Risk review

- expansion remains blocked by default
- missing previous report does not become a fake comparison
- regression now blocks expansion recommendation
- all paths remain local-only

## Deferred

- richer archive browsing than snapshot listing
- matrix-level snapshot archive beyond campaign-level v0 storage
- more advanced diff significance logic
- any live, broker, credential, or persona-expansion scope

## Recommended Sprint 13

Use the campaign archive on broader local datasets, then tighten thresholds only where repeated evidence justifies stricter or more specific gates.
