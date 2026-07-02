# Cleanup Plan

## Current state

The temporary quarantine archive has been removed. The repository now keeps only the active Soma Zero workspace plus supporting non-code assets like `configs/`, `data/`, `docs/`, and `bin/`.

## What was deleted after the archive phase

The earlier quarantine step was used as an intermediate safety check before permanent deletion. The archived legacy reports, utility crates, router/online-learning crates, and old experiment crates have now been removed from the repository.

## What remains active

- root package and `src/`
- `configs/`, `data/`, `docs/`, `examples/`, and `bin/`

## What still needs human judgment

- the local temporary instruction artifact
- whether any remaining docs/history files should also be trimmed now that the archive is gone
