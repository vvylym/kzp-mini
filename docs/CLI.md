# CLI Reference

The `kzp` CLI talks to the on-chain program over RPC. Amounts are in **base units** (smallest SPL denomination).

Setup: [GETTING_STARTED.md](./GETTING_STARTED.md).

## Pool setup and default cranking

```bash
# Create pool + vault (prints pool PDA)
kzp pool initialize --name "Fabryka Lodz KZP" --entry-fee 50 --mint <MINT>

# Mark default after due date from reserved guarantor liability.
# Any signer may crank this instruction; the borrower receives closed loan rent.
kzp loan settle-default --loan <LOAN> --pool <POOL>
```

## Members

```bash
kzp pool join --pool <POOL> --entry-fee 50
kzp pool deposit --pool <POOL> --amount 1000000
kzp pool exit --pool <POOL>
```

## Borrowers

```bash
kzp loan request --pool <POOL> --nonce 1 --amount 500000 \
  --guarantor-a <PUBKEY_A> --guarantor-b <PUBKEY_B> \
  --term-seconds 2592000

kzp loan repay --loan <LOAN> --amount 500000
kzp loan cancel --loan <LOAN>    # Pending only
```

Full repayment closes the loan account and returns rent to the borrower. Partial repayment sends only the base repay account set; the CLI appends finalization accounts automatically when the amount equals the fetched outstanding balance.

## Guarantors

```bash
kzp loan cosign --loan <LOAN>
kzp loan withdraw-cosign --loan <LOAN>    # Pending only, partial co-sign
```

Run `cosign` once per guarantor wallet. The first co-sign uses the slim partial account set. The second co-sign appends activation accounts automatically, reserves guarantor backing from savings, and disburses if the vault has liquidity.

## Diagnostics

```bash
kzp config
kzp --dry-run pool join --pool <POOL> --entry-fee 50
```

When the CLI fetches pool or loan accounts to derive follow-up accounts, it verifies that the fetched account is owned by the `kzp-mini` program before deserializing it.

## Global flags

| Flag | Description |
|------|-------------|
| `--cluster` | `localnet`, `devnet`, `mainnet` |
| `--rpc-url` | Override RPC endpoint |
| `--wallet` | Signing keypair JSON path |
| `--dry-run` | Simulate transaction; print logs on failure |
