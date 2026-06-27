# Instructions

Account layouts and handler logic live together in `programs/kzp-mini/src/instructions/*`.

## `initialize_pool`

Creates a pool and SPL token vault.

| Account | Mut | Signer | Notes |
|---------|-----|--------|-------|
| admin | ✓ | ✓ | Becomes `pool.admin` |
| pool | ✓ | | PDA `["pool", admin, pool_name]` |
| vault | ✓ | | PDA `["vault", pool]`; authority = vault PDA |
| token_mint | | | SPL mint for pool currency |
| system_program | | | |
| token_program | | | |
| rent | | | |

**Args:** `pool_name: String`, `required_entry_fee: u64`

---

## `join_pool`

Pays entry fee and opens a member account.

| Account | Mut | Signer |
|---------|-----|--------|
| member | ✓ | ✓ |
| member_account | ✓ | |
| pool | ✓ | |
| vault | ✓ | |
| member_token_account | ✓ | |
| token_program | | |
| system_program | | |

**Args:** `entry_fee: u64` (must match `pool.required_entry_fee`)

---

## `deposit_savings`

Transfers tokens from member ATA to vault; increases `savings_balance` and `pool.total_savings`.

**Args:** `amount: u64` (> 0)

---

## `request_loan`

Opens a `Pending` loan with two nominated guarantors.

| Account | Mut | Signer |
|---------|-----|--------|
| borrower | ✓ | ✓ |
| member_account | ✓ | |
| pool | | |
| loan | ✓ | PDA `["loan", pool, borrower, loan_nonce_le]` |
| guarantor_a_member, guarantor_b_member | | |
| guarantor_a, guarantor_b | | UncheckedAccount; validated via member PDAs |
| system_program | | |

**Args:** `loan_nonce: u64`, `amount: u64`, `loan_term_seconds: i64`

**Rules:** amount ≤ 3× borrower savings; one unresolved loan per borrower (pending or active); guarantors distinct from borrower; each guarantor under guarantee cap (active + pending).

Stores `vault_bump` on the loan for disbursement CPI signing, stores `due_ts`, and reserves `member_account.pending_loan`.

---

## `co_sign_loan`

Guarantor approves a pending loan. When both guarantors have signed:

1. Vault liquidity checked (`vault.amount >= principal`)
2. Borrower and guarantor eligibility rechecked
3. Guarantor 50/50 liability is reserved in `locked_savings`
4. Loan → `Active`; principal disbursed to borrower ATA
5. Both guarantors move from `pending_guarantee_count` → `active_guarantee_count`
6. Borrower `pending_loan` moves to `active_loan`
7. `pool.total_outstanding_loans += principal`

Partial co-sign only sets `guarantor_*_signed` and increments `pending_guarantee_count`.

| Account | Mut | Signer |
|---------|-----|--------|
| guarantor | ✓ | ✓ |
| loan | ✓ | |
| guarantor_member | ✓ | signer member PDA |

When the co-sign completes activation, append remaining accounts in this exact order:

| Remaining account | Mut | Notes |
|-------------------|-----|-------|
| pool | ✓ | must match `loan.pool` |
| other_guarantor_member | ✓ | PDA for the other nominated guarantor |
| borrower_member | ✓ | PDA for `loan.borrower` |
| vault | ✓ | canonical pool vault |
| borrower_token_account | ✓ | receives principal |
| token_program | | SPL Token program |

---

## `repay_loan`

Borrower repays partially or fully to vault.

- Decrements `loan.outstanding` and `pool.total_outstanding_loans`
- On full repayment: closes the loan account to the borrower, clears matching borrower `active_loan`, releases loan-local guarantor backing from `locked_savings`, and decrements `active_guarantee_count`

Full repayment must append remaining accounts: `borrower_member`, `guarantor_a_member`, `guarantor_b_member`.

**Args:** `amount: u64`

---

## `cancel_loan`

Borrower cancels a **Pending** loan anytime. Closes loan account (rent to borrower); clears borrower `pending_loan` and decrements signed guarantor `pending_guarantee_count`.

| Account | Mut | Signer |
|---------|-----|--------|
| borrower | ✓ | ✓ |
| loan | ✓ | closed to borrower |
| borrower_member | ✓ | |
| guarantor_a_member, guarantor_b_member | ✓ | |

---

## `withdraw_cosign`

Guarantor revokes a partial co-sign while loan is **Pending**. Clears their signature flag and decrements `pending_guarantee_count`.

| Account | Mut | Signer |
|---------|-----|--------|
| guarantor | ✓ | ✓ |
| loan | ✓ | |
| guarantor_a_member, guarantor_b_member | ✓ | |

---

## `settle_default`

Any signer may initiate after `loan.due_ts`. Marks an **Active** loan as defaulted from reserved guarantor savings and closes the loan account to the borrower. Guarantors do not sign default settlement because liability was reserved at activation.

- Splits outstanding 50/50 (odd amounts: first guarantor gets ceiling)
- Deducts shares from each guarantor's `savings_balance`
- Decrements `pool.total_savings` and `pool.total_outstanding_loans` by outstanding
- Clears matching borrower `active_loan`, releases loan-local guarantor backing from `locked_savings`, and decrements `active_guarantee_count`

| Account | Mut | Signer |
|---------|-----|--------|
| crank | | ✓ |
| pool | ✓ | |
| loan | ✓ | closed to borrower |
| borrower | ✓ | close recipient; must equal `loan.borrower` |
| borrower_member | ✓ | |
| guarantor_a_member, guarantor_b_member | ✓ | |

Large account struct uses `Box<>` in the Anchor context to stay under BPF stack limits.

---

## `exit_pool`

Withdraws `savings_balance` from vault and closes member account.

**Blocked when:** `active_loan` set, `pending_loan` set, `locked_savings > 0`, non-zero `active_guarantee_count`, non-zero `pending_guarantee_count`, or `vault.amount < savings_balance` (`InsufficientVaultLiquidity`).

| Account | Mut | Signer |
|---------|-----|--------|
| member | ✓ | ✓ |
| member_account | ✓ | closed to member |
| pool | ✓ | |
| vault | ✓ | |
| member_token_account | ✓ | |
| token_program | | |
