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
NO_DNA=1 anchor build --ignore-keys

echo "==> Setting Solana CLI cluster to $RPC_CLUSTER"
solana config set --url "$RPC_CLUSTER"

echo "==> Deploying kzp_mini to $RPC_CLUSTER"
NO_DNA=1 anchor deploy --provider.cluster "$RPC_CLUSTER"

echo "==> Deploy complete"
solana program show "$(solana address -k "$ROOT/target/deploy/kzp_mini-keypair.json")"
