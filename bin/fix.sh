#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "[fix] running cargo fix --allow-dirty --allow-staged"
cargo fix --allow-dirty --allow-staged

echo "[fix] running cargo clippy --workspace -D warnings"
cargo clippy --workspace -D warnings
