#!/usr/bin/env bash
# End-to-end devnet demo: deploy program, create pool, loan lifecycle, print explorer links.
# Usage: bash scripts/devnet-demo.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RPC="https://api.devnet.solana.com"
EXPLORER="https://explorer.solana.com/tx"
PROGRAM_ID="6kf3ieibPCviUiySokEPmfxXP3dB7bW4eb9aof7sdkX2"
POOL_NAME="SuperteamDemo"
ENTRY_FEE=50
DEPOSIT=1000000
LOAN_AMOUNT=500000
LOAN_NONCE=1
DEMO_DIR="$(mktemp -d)"
GA_WALLET="$DEMO_DIR/guarantor_a.json"
GB_WALLET="$DEMO_DIR/guarantor_b.json"
MINT_FILE="$DEMO_DIR/mint.txt"

cleanup() {
  rm -rf "$DEMO_DIR"
}
trap cleanup EXIT

kzp() {
  cargo run -q -p kzp-cli --release -- --cluster devnet "$@"
}

extract_sig() {
  grep -E '^Signature:' | awk '{print $2}'
}

print_tx() {
  local label="$1"
  local sig="$2"
  if [[ -n "$sig" && "$sig" != "1111111111111111111111111111111111111111111111111111111111111111" ]]; then
    echo "  $label: $EXPLORER/$sig?cluster=devnet"
  else
    echo "  $label: (no signature)"
  fi
}

echo "==> Building program and CLI"
NO_DNA=1 anchor build --ignore-keys
cargo build -q -p kzp-cli --release

echo "==> Configuring Solana CLI for devnet"
solana config set --url "$RPC" >/dev/null

echo "==> Deploying program (skip if already live)"
if ! solana program show "$PROGRAM_ID" --url "$RPC" >/dev/null 2>&1; then
  bash scripts/deploy.sh devnet
else
  echo "Program already deployed at $PROGRAM_ID"
fi

echo "==> Creating demo keypairs and SPL mint"
solana-keygen new --no-bip39-passphrase -o "$GA_WALLET" --force >/dev/null
solana-keygen new --no-bip39-passphrase -o "$GB_WALLET" --force >/dev/null

for wallet in "$GA_WALLET" "$GB_WALLET"; do
  solana airdrop 2 "$(solana-keygen pubkey "$wallet")" --url "$RPC" >/dev/null || true
done

MINT=$(spl-token create-token --url "$RPC" 2>&1 | awk '/Creating token/ {print $3}')
echo "$MINT" >"$MINT_FILE"
ADMIN=$(solana address)
spl-token create-account "$MINT" --url "$RPC" >/dev/null
spl-token mint "$MINT" 10000000000 "$ADMIN" --url "$RPC" >/dev/null

for wallet in "$GA_WALLET" "$GB_WALLET"; do
  pk=$(solana-keygen pubkey "$wallet")
  spl-token create-account "$MINT" --owner "$pk" --url "$RPC" --fee-payer "$ADMIN" >/dev/null
  spl-token mint "$MINT" 10000000000 "$pk" --url "$RPC" --fee-payer "$ADMIN" >/dev/null
done

GA_PK=$(solana-keygen pubkey "$GA_WALLET")
GB_PK=$(solana-keygen pubkey "$GB_WALLET")

declare -A TXS

echo "==> Initialize pool"
OUT=$(kzp pool initialize --name "$POOL_NAME" --entry-fee "$ENTRY_FEE" --mint "$MINT" 2>&1)
echo "$OUT"
POOL=$(echo "$OUT" | awk '/^Pool initialized:/ {print $3}')
TXS[init]=$(echo "$OUT" | extract_sig)

echo "==> Admin joins pool"
OUT=$(kzp pool join --pool "$POOL" --entry-fee "$ENTRY_FEE" 2>&1)
echo "$OUT"
TXS[join_admin]=$(echo "$OUT" | extract_sig)

echo "==> Guarantors join pool"
OUT=$(kzp --wallet "$GA_WALLET" pool join --pool "$POOL" --entry-fee "$ENTRY_FEE" 2>&1)
echo "$OUT"
TXS[join_ga]=$(echo "$OUT" | extract_sig)

OUT=$(kzp --wallet "$GB_WALLET" pool join --pool "$POOL" --entry-fee "$ENTRY_FEE" 2>&1)
echo "$OUT"
TXS[join_gb]=$(echo "$OUT" | extract_sig)

echo "==> Admin deposits savings"
OUT=$(kzp pool deposit --pool "$POOL" --amount "$DEPOSIT" 2>&1)
echo "$OUT"
TXS[deposit]=$(echo "$OUT" | extract_sig)

echo "==> Admin requests loan"
OUT=$(kzp loan request --pool "$POOL" --nonce "$LOAN_NONCE" --amount "$LOAN_AMOUNT" \
  --guarantor-a "$GA_PK" --guarantor-b "$GB_PK" 2>&1)
echo "$OUT"
LOAN=$(echo "$OUT" | awk '/^Loan requested:/ {print $3}')
TXS[request]=$(echo "$OUT" | extract_sig)

echo "==> Guarantor A co-signs"
OUT=$(kzp --wallet "$GA_WALLET" loan cosign --loan "$LOAN" 2>&1)
echo "$OUT"
TXS[cosign_a]=$(echo "$OUT" | extract_sig)

echo "==> Guarantor B co-signs (disbursement)"
OUT=$(kzp --wallet "$GB_WALLET" loan cosign --loan "$LOAN" 2>&1)
echo "$OUT"
TXS[cosign_b]=$(echo "$OUT" | extract_sig)

echo "==> Admin repays loan"
OUT=$(kzp loan repay --loan "$LOAN" --amount "$LOAN_AMOUNT" 2>&1)
echo "$OUT"
TXS[repay]=$(echo "$OUT" | extract_sig)

echo "==> Admin exits pool"
OUT=$(kzp pool exit --pool "$POOL" 2>&1)
echo "$OUT"
TXS[exit]=$(echo "$OUT" | extract_sig)

echo ""
echo "========================================"
echo "Devnet demo complete"
echo "========================================"
echo "Program:  $PROGRAM_ID"
echo "Pool:     $POOL"
echo "Loan:     $LOAN"
echo "Mint:     $MINT"
echo "Admin:    $ADMIN"
echo "Guarantor A: $GA_PK"
echo "Guarantor B: $GB_PK"
echo ""
echo "Transaction links:"
print_tx "Initialize pool" "${TXS[init]}"
print_tx "Admin join" "${TXS[join_admin]}"
print_tx "Guarantor A join" "${TXS[join_ga]}"
print_tx "Guarantor B join" "${TXS[join_gb]}"
print_tx "Deposit savings" "${TXS[deposit]}"
print_tx "Request loan" "${TXS[request]}"
print_tx "Co-sign (guarantor A)" "${TXS[cosign_a]}"
print_tx "Co-sign + disburse (guarantor B)" "${TXS[cosign_b]}"
print_tx "Repay loan" "${TXS[repay]}"
print_tx "Exit pool" "${TXS[exit]}"
