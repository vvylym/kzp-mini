# Keypairs (local only - never commit)

All `*.json` files in this directory are **gitignored**. Do not push private keys to GitHub, even for devnet.

| File | Purpose |
|------|---------|
| `program.json` | Program deploy keypair. Must match the **Program ID** in `README.md` / `declare_id!`. Required for `scripts/deploy.sh` and local `anchor build`. |
| `devnet/guarantor_a.json` | Demo guarantor wallet; created automatically by `scripts/devnet-demo.sh` if missing. |
| `devnet/guarantor_b.json` | Second demo guarantor wallet. |

## First-time setup

```bash
# After clone: restore or create the program keypair (get from deploy authority offline).
# If you are the deploy authority and need a fresh keypair:
mkdir -p keys/devnet
solana-keygen new -o keys/program.json --no-bip39-passphrase
cp keys/program.json target/deploy/kzp_mini-keypair.json
NO_DNA=1 anchor keys sync
NO_DNA=1 anchor build --ignore-keys
```

CI generates a **throwaway** program keypair per run; only your local `keys/program.json` pins the devnet program address.
