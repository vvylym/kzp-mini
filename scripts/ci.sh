#!/usr/bin/env bash
# Local CI: format, lint, test, build (mirrors .github/workflows/ci.yml).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
PROGRAM_KEYPAIR="$ROOT/keys/program.json"

echo "==> rustfmt"
cargo fmt --all -- --check

if [[ ! -f "$PROGRAM_KEYPAIR" ]]; then
  echo "==> generating ephemeral keys/program.json for local CI"
  mkdir -p keys
  solana-keygen new -o "$PROGRAM_KEYPAIR" --no-bip39-passphrase --force >/dev/null
  cp "$PROGRAM_KEYPAIR" target/deploy/kzp_mini-keypair.json 2>/dev/null || mkdir -p target/deploy && cp "$PROGRAM_KEYPAIR" target/deploy/kzp_mini-keypair.json
  NO_DNA=1 anchor keys sync
fi

echo "==> anchor build"
mkdir -p target/deploy
cp -f "$PROGRAM_KEYPAIR" target/deploy/kzp_mini-keypair.json
NO_DNA=1 anchor build --ignore-keys

echo "==> clippy"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "==> nextest"
cargo nextest run --workspace --all-targets --all-features

echo "==> CLI smoke test"
mkdir -p "$HOME/.config/solana"
if [[ ! -f "$HOME/.config/solana/id.json" ]]; then
  solana-keygen new -o "$HOME/.config/solana/id.json" --no-bip39-passphrase --force >/dev/null
fi
cargo run -p kzp-cli -- config

echo "==> All checks passed"
