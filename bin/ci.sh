#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Offline CI: prevent network access while using cached dependencies
# Note: Requires dependencies to be pre-cached via normal build
export RUSTFLAGS="${RUSTFLAGS:-} -C target-cpu=native -C link-arg=-s"

# Offline cargo config
OFFLINE_CONFIG="ci/offline.cargo-config"

log_step() {
  echo ""
  echo "==> $1"
}

run() {
  echo "    $*"
  "$@"
}

trap 'echo "\nCI pipeline failed"' ERR

log_step "Build workspace (offline)"
run cargo build --workspace --frozen --config "$OFFLINE_CONFIG"

log_step "Doc build (offline)"
run cargo doc --workspace --no-deps --frozen --config "$OFFLINE_CONFIG"

log_step "Test workspace (offline)"
run cargo test --workspace --frozen --config "$OFFLINE_CONFIG"

echo ""
echo "✓ OFFLINE CI OK"
echo ""
echo "All checks passed without network access:"
echo "  - Workspace built with --frozen"
echo "  - Documentation generated"
echo "  - Tests passed"
echo ""

# Additional integration steps (optional, commented out for offline mode):
# log_step "Online crawl/curate/verify"
# soma-online was removed during Sprint 03 legacy isolation.
#
# log_step "Adapt / train (simulated)"
# soma-train was removed in Sprint 02 final cleanup.
#
# log_step "Validation (paranoid)"
# soma-validate was removed during workspace cleanup.
#
# log_step "Soak (1h synthetic)"
# soma-soak was removed in Sprint 02 final cleanup.
#
# log_step "Canary guard"
# soma-canary was removed in Sprint 02 final cleanup.
#
# log_step "Orchestration (no deploy)"
# soma-orchestrate was removed in Sprint 02 final cleanup.
#
# log_step "Release bundle"
# soma-release was removed in Sprint 02 deep cleanup.
