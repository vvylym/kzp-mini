#!/usr/bin/env bash
# Local CI: format, lint, test, build, CLI smoke (mirrors .github/workflows/ci.yml).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> rustfmt"
cargo fmt --all -- --check

echo "==> clippy"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "==> anchor build"
NO_DNA=1 anchor build --ignore-keys

echo "==> unit + integration tests"
cargo llvm-cov nextest --workspace --all-targets --all-features --summary-only

echo "==> CLI smoke test"
cargo run -p kzp-cli -- config

echo "==> All checks passed"
