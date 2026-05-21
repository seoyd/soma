# Previous collection comparison

Sprint 26 wires `previous_collection_report_path` into the official evidence workflow.

## What it compares

- previous ready entries
- current ready entries
- added ready entries
- removed ready entries
- previous missing auth providers
- current missing auth providers
- fixed missing auth providers
- newly missing auth providers

## Conservative behavior

- if the previous report file is missing, the workflow does not panic
- the comparison stays `comparable=false`
- the missing file is reason-coded so the operator can see why the delta is incomplete

## Why it matters

This makes readiness deltas explicit instead of forcing the operator to inspect two separate collection reports by hand.
