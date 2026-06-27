#!/usr/bin/env bash
# End-to-end devnet demo: deploy program, create pool, loan lifecycle, print explorer links.
# Usage: bash scripts/devnet-demo.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RPC="https://api.devnet.solana.com"
EXPLORER="https://explorer.solana.com/tx"
PROGRAM_KEYPAIR="$ROOT/keys/program.json"
PROGRAM_ID="$(solana address -k "$PROGRAM_KEYPAIR")"
POOL_NAME="KZPDemo$(date +%s)"
ENTRY_FEE=50
DEPOSIT=1000000
LOAN_AMOUNT=500000
LOAN_NONCE=1
WALLETS_DIR="$ROOT/keys/devnet"
GA_WALLET="$WALLETS_DIR/guarantor_a.json"
GB_WALLET="$WALLETS_DIR/guarantor_b.json"
GUARANTOR_SOL=0.05

if [[ ! -f "$PROGRAM_KEYPAIR" ]]; then
  echo "ERROR: missing $PROGRAM_KEYPAIR (see keys/README.md)" >&2
  exit 1
fi

mkdir -p "$WALLETS_DIR"
for wallet in "$GA_WALLET" "$GB_WALLET"; do
  if [[ ! -f "$wallet" ]]; then
    solana-keygen new --no-bip39-passphrase -o "$wallet" --force >/dev/null
  fi
done

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

fund_guarantor() {
  local dest="$1"
  local balance
  balance=$(solana balance "$dest" --url "$RPC" | awk '{print $1}')
  if awk "BEGIN {exit !($balance < $GUARANTOR_SOL)}"; then
    solana transfer "$dest" "$GUARANTOR_SOL" --url "$RPC" --allow-unfunded-recipient >/dev/null
  fi
}

echo "==> Building program and CLI"
mkdir -p target/deploy
cp -f "$PROGRAM_KEYPAIR" target/deploy/kzp_mini-keypair.json
NO_DNA=1 anchor build --ignore-keys
cargo build -q -p kzp-cli --release

echo "==> Configuring Solana CLI for devnet"
solana config set --url "$RPC" >/dev/null

echo "==> Deploying program (skip if already live)"
if ! solana program show "$PROGRAM_ID" --url "$RPC" >/dev/null 2>&1; then
  BALANCE=$(solana balance --url "$RPC" | awk '{print $1}')
  REQUIRED=2.85
  if awk "BEGIN {exit !($BALANCE < $REQUIRED)}"; then
    echo "ERROR: deploy needs ~${REQUIRED} SOL on devnet (wallet has ${BALANCE} SOL)." >&2
    exit 1
  fi
  bash scripts/deploy.sh devnet
else
  echo "Program already deployed at $PROGRAM_ID"
fi

echo "==> Funding guarantor wallets from admin"
ADMIN=$(solana address)
FEE_PAYER="${SOLANA_WALLET:-$HOME/.config/solana/id.json}"
GA_PK=$(solana-keygen pubkey "$GA_WALLET")
GB_PK=$(solana-keygen pubkey "$GB_WALLET")
fund_guarantor "$GA_PK"
fund_guarantor "$GB_PK"

echo "==> Creating SPL mint and token accounts"
MINT=$(spl-token create-token --decimals 6 --url "$RPC" --fee-payer "$FEE_PAYER" 2>&1 | awk '/^Address:/ {print $2}')
spl-token create-account "$MINT" --owner "$ADMIN" --url "$RPC" --fee-payer "$FEE_PAYER" >/dev/null
spl-token mint "$MINT" 1000000 --url "$RPC" --fee-payer "$FEE_PAYER" >/dev/null

for wallet in "$GA_WALLET" "$GB_WALLET"; do
  pk=$(solana-keygen pubkey "$wallet")
  spl-token create-account "$MINT" --owner "$pk" --url "$RPC" --fee-payer "$FEE_PAYER" >/dev/null
  spl-token mint "$MINT" 1000000 --recipient-owner "$pk" --url "$RPC" --fee-payer "$FEE_PAYER" >/dev/null
done

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

echo "==> Guarantors deposit savings for backing"
OUT=$(kzp --wallet "$GA_WALLET" pool deposit --pool "$POOL" --amount "$DEPOSIT" 2>&1)
echo "$OUT"
TXS[deposit_ga]=$(echo "$OUT" | extract_sig)

OUT=$(kzp --wallet "$GB_WALLET" pool deposit --pool "$POOL" --amount "$DEPOSIT" 2>&1)
echo "$OUT"
TXS[deposit_gb]=$(echo "$OUT" | extract_sig)

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
print_tx "Admin deposit savings" "${TXS[deposit]}"
print_tx "Guarantor A deposit savings" "${TXS[deposit_ga]}"
print_tx "Guarantor B deposit savings" "${TXS[deposit_gb]}"
print_tx "Request loan" "${TXS[request]}"
print_tx "Co-sign (guarantor A)" "${TXS[cosign_a]}"
print_tx "Co-sign + disburse (guarantor B)" "${TXS[cosign_b]}"
print_tx "Repay loan" "${TXS[repay]}"
print_tx "Exit pool" "${TXS[exit]}"
