# On-Chain State

See [Architecture](../README.md#architecture) for how ledger fields relate to the SPL vault and guarantee lists.

## Pool

| Field | Type | Description |
|-------|------|-------------|
| admin | Pubkey | Pool administrator (may call `settle_default`) |
| token_mint | Pubkey | SPL mint for all pool token flows |
| vault | Pubkey | Token vault PDA |
| required_entry_fee | u64 | One-time join fee (not credited to savings) |
| total_members | u64 | Open member account count |
| total_savings | u64 | Sum of member `savings_balance` ledgers |
| total_outstanding_loans | u64 | Sum of active loan outstanding (disburse, repay, default) |
| bump | u8 | Pool PDA bump |
| vault_bump | u8 | Vault PDA bump |

## Member

| Field | Type | Description |
|-------|------|-------------|
| pool | Pubkey | Parent pool |
| member_id | u64 | Join order index |
| owner | Pubkey | Member wallet |
| entry_fee_paid | u64 | Fee recorded at join |
| savings_balance | u64 | Ledger balance (withdrawn on `exit_pool`) |
| active_loan | Option\<Pubkey\> | Disbursed loan as borrower |
| pending_loan | Option\<Pubkey\> | Pending loan request as borrower |
| active_guarantees | Vec\<Pubkey\> | Disbursed loans guaranteed (max 5) |
| pending_guarantees | Vec\<Pubkey\> | Pending loans co-signed (max 5) |
| bump | u8 | Member PDA bump |

## Loan

| Field | Type | Description |
|-------|------|-------------|
| pool | Pubkey | Parent pool |
| borrower | Pubkey | Borrower wallet |
| principal | u64 | Original disbursed amount |
| outstanding | u64 | Remaining balance |
| guarantor_a, guarantor_b | Pubkey | Nominated guarantors |
| guarantor_a_signed, guarantor_b_signed | bool | Partial co-sign flags (while Pending) |
| status | LoanStatus | Lifecycle |
| bump | u8 | Loan PDA bump |
| vault_bump | u8 | Copied from pool at request (vault CPI signing) |

## LoanStatus

| Value | Meaning |
|-------|---------|
| Pending | Awaiting both co-signs; no disbursement |
| Active | Disbursed; outstanding may be > 0 |
| Repaid | Fully repaid |
| Defaulted | Admin settled; outstanding zeroed |

## PDA seeds

- Pool: `["pool", admin, pool_name]`
- Vault: `["vault", pool]`
- Member: `["member", pool, owner]`
- Loan: `["loan", pool, borrower, loan_nonce_le]`

Rust helpers: `kzp_mini::utils::pda` and `kzp_mini::utils::seeds`.
