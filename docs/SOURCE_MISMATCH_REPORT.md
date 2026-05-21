# Source Mismatch Report

The source mismatch report compares overlapping official and yfinance datasets conservatively.

## Signals

- row-count delta
- timestamp mismatch count
- missing-row count
- average/max price drift in bps
- average volume delta ratio
- adjusted/raw policy mismatch
- gap mismatch count
- data-quality delta

## Severity

- `None`
- `Low`
- `Medium`
- `High`
- `NotComparable`

`High` means the sources should not be treated as stable substitutes. `NotComparable` means the data shapes differ too much, often because adjusted/raw policies differ.
