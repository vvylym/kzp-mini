# KZP Minimal — Documentation

On-chain workplace mutual-aid pool (Kasa Zapomogowa Pracownicza) on Solana.

**Program ID:** `GsjUnBFvYtcxNwCrydPUjQTngGTqdx5v7APnWahnqwkx`

**Architecture** (traditional KZP vs Solana, code layout, tradeoffs): see the [root README](../README.md#architecture).

## Contents

| Document | Description |
|----------|-------------|
| [INSTRUCTIONS.md](./INSTRUCTIONS.md) | All instructions and account requirements |
| [STATE.md](./STATE.md) | On-chain account layouts and lifecycle |
| [ERRORS.md](./ERRORS.md) | Custom `PoolError` codes |
| [USE_CASES.md](./USE_CASES.md) | BDD use cases and acceptance criteria |
| [SECURITY.md](./SECURITY.md) | Threat model, mitigations, and accepted limitations |
| [../tests/README.md](../tests/README.md) | Integration test layout and conventions |

## Quick reference

### Instructions

| Instruction | Purpose |
|-------------|---------|
| `initialize_pool` | Admin creates pool + SPL token vault |
| `join_pool` | Pay entry fee, open member PDA |
| `deposit_savings` | Credit member savings ledger; tokens to vault |
| `request_loan` | Open pending loan (max 3× savings, two guarantors) |
| `co_sign_loan` | Guarantor co-signs; disburses when both sign |
| `repay_loan` | Partial or full repayment to vault |
| `cancel_loan` | Borrower cancels pending loan |
| `withdraw_cosign` | Guarantor revokes partial co-sign |
| `settle_default` | Admin default; 50/50 from guarantor savings + SPL to vault |
| `exit_pool` | Withdraw savings and close member PDA |

### PDAs

| Account | Seeds |
|---------|-------|
| Pool | `["pool", admin, pool_name]` |
| Vault | `["vault", pool]` |
| Member | `["member", pool, owner]` |
| Loan | `["loan", pool, borrower, loan_nonce_le]` |

Off-chain helpers: `kzp_mini::utils::pda::{pool_pda, vault_pda, member_pda, loan_pda}`.

### Repository layout

```
programs/kzp-mini/   Anchor program (on-chain)
tests/               Integration tests (solana-program-test)
cli/                 `kzp` CLI for devnet/localnet
scripts/             ci.sh, deploy.sh
docs/                Instruction, state, security reference
```

Install, CLI usage, and contributing: see the [root README](../README.md).
