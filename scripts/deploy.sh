#!/usr/bin/env bash
# Deploy kzp-mini to localnet, devnet, or mainnet.
set -euo pipefail

CLUSTER="${1:-localnet}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

case "$CLUSTER" in
  localnet|devnet|mainnet-beta|mainnet)
    ;;
  *)
    echo "Usage: $0 [localnet|devnet|mainnet]" >&2
    exit 1
    ;;
esac

RPC_CLUSTER="$CLUSTER"
if [[ "$CLUSTER" == "mainnet" ]]; then
  RPC_CLUSTER="mainnet-beta"
fi

echo "==> Building program"
cd "$ROOT"
mkdir -p target/deploy
cp -f "$ROOT/keys/kzp_mini-keypair.json" target/deploy/kzp_mini-keypair.json
NO_DNA=1 anchor build --ignore-keys

echo "==> Setting Solana CLI cluster to $RPC_CLUSTER"
solana config set --url "$RPC_CLUSTER"

echo "==> Deploying kzp_mini to $RPC_CLUSTER"
PROGRAM_ID="$(solana address -k "$ROOT/keys/kzp_mini-keypair.json")"
if solana program show "$PROGRAM_ID" --url "$RPC_CLUSTER" >/dev/null 2>&1; then
  NO_DNA=1 anchor program deploy --provider.cluster "$RPC_CLUSTER" --max-sign-attempts 200
else
  solana program deploy "$ROOT/target/deploy/kzp_mini.so" \
    --program-id "$ROOT/keys/kzp_mini-keypair.json" \
    --url "$RPC_CLUSTER" \
    --max-sign-attempts 200
fi

echo "==> Deploy complete"
solana program show "$(solana address -k "$ROOT/target/deploy/kzp_mini-keypair.json")"
