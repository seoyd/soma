# Sprint 16 Report

## Implemented items

- evidence source taxonomy
- data provenance metadata
- real-local dataset registration path
- real-only evidence counting
- synthetic-vs-real comparison
- `soma-experiment real-evidence --config ...`

## Whether real local data exists

The example config points to a placeholder path under `data/local/`. If the user has not supplied a CSV there, the example run must remain conservative and report missing real local data.

Current example result:

- readiness before: `NeedMoreExperiments`
- readiness after: `MissingRealLocalData`
- final recommendation: `MissingRealLocalData`
- real-only datasets/outcomes/variants: `0 / 0 / 0`

## Real-only targets

- real-local datasets: `>= 1`
- real-local outcome records: `>= 20`
- real-local comparable variants: `>= 2`

## Readiness before/after

Sprint 15 may have closed synthetic coverage, but Sprint 16 rechecks readiness with real-only evidence. Without valid real local data, readiness must remain conservative.

## Risk review

- no live API path added
- no broker path added
- no downloader added
- no runtime LLM path added
- synthetic/test evidence excluded from readiness counts

## Deferred items

- broader user-supplied real local dataset coverage
- multi-symbol real-only evidence comparison
- any design-review widening beyond current conservative gates

## Recommended Sprint 17

Gather at least one valid user-supplied local CSV with sufficient rows, rerun real-only evidence closure, and compare real-only results across more than one locally provided dataset before considering any broader scope change.
