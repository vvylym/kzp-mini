#!/usr/bin/env bash
# Deploy kzp-mini to localnet, devnet, or mainnet.
set -euo pipefail

CLUSTER="${1:-localnet}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROGRAM_KEYPAIR="$ROOT/keys/program.json"

case "$CLUSTER" in
  localnet|devnet|mainnet-beta|mainnet)
    ;;
  *)
    echo "Usage: $0 [localnet|devnet|mainnet]" >&2
    exit 1
    ;;
esac

if [[ ! -f "$PROGRAM_KEYPAIR" ]]; then
  echo "ERROR: missing $PROGRAM_KEYPAIR (see keys/README.md)" >&2
  exit 1
fi

RPC_CLUSTER="$CLUSTER"
if [[ "$CLUSTER" == "mainnet" ]]; then
  RPC_CLUSTER="mainnet-beta"
fi

echo "==> Building program"
cd "$ROOT"
mkdir -p target/deploy
cp -f "$PROGRAM_KEYPAIR" target/deploy/kzp_mini-keypair.json
NO_DNA=1 anchor build --ignore-keys

echo "==> Setting Solana CLI cluster to $RPC_CLUSTER"
solana config set --url "$RPC_CLUSTER"

echo "==> Deploying kzp_mini to $RPC_CLUSTER"
PROGRAM_ID="$(solana address -k "$PROGRAM_KEYPAIR")"
solana program deploy "$ROOT/target/deploy/kzp_mini.so" \
  --program-id "$PROGRAM_KEYPAIR" \
  --url "$RPC_CLUSTER" \
  --max-sign-attempts 200

echo "==> Deploy complete"
solana program show "$(solana address -k "$ROOT/target/deploy/kzp_mini-keypair.json")"
