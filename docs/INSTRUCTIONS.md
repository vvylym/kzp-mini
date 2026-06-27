# Instructions

Account layouts are defined in `programs/kzp-mini/src/instructions/`. Handler logic lives in `programs/kzp-mini/src/handlers/*/handle`.

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

**Args:** `loan_nonce: u64`, `amount: u64`

**Rules:** amount ≤ 3× borrower savings; one unresolved loan per borrower (pending or active); guarantors distinct from borrower; each guarantor under guarantee cap (active + pending).

Stores `vault_bump` on the loan for disbursement CPI signing and reserves `member_account.pending_loan`.

---

## `co_sign_loan`

Guarantor approves a pending loan. When both guarantors have signed:

1. Vault liquidity checked (`vault.amount >= principal`)
2. Borrower and guarantor eligibility rechecked
3. Guarantor 50/50 liability is reserved in `locked_savings`
4. Loan → `Active`; principal disbursed to borrower ATA
5. Both guarantors move from `pending_guarantees` → `active_guarantees`
6. Borrower `pending_loan` moves to `active_loan`
7. `pool.total_outstanding_loans += principal`

Partial co-sign only sets `guarantor_*_signed` and adds `pending_guarantees`.

| Account | Mut | Signer |
|---------|-----|--------|
| guarantor | ✓ | ✓ |
| loan | ✓ | |
| pool | ✓ | |
| guarantor_a_member, guarantor_b_member | ✓ | |
| vault | ✓ | |
| borrower_member | ✓ | |
| borrower_token_account | ✓ | |
| token_program | | |

---

## `repay_loan`

Borrower repays partially or fully to vault.

- Decrements `loan.outstanding` and `pool.total_outstanding_loans`
- On full repayment: loan → `Repaid`, clears matching borrower `active_loan`, releases guarantor `locked_savings`, and clears guarantor `active_guarantees`

**Args:** `amount: u64`

---

## `cancel_loan`

Borrower cancels a **Pending** loan anytime. Closes loan account (rent to borrower); clears borrower `pending_loan` and guarantor `pending_guarantees`.

| Account | Mut | Signer |
|---------|-----|--------|
| borrower | ✓ | ✓ |
| loan | ✓ | closed to borrower |
| borrower_member | ✓ | |
| guarantor_a_member, guarantor_b_member | ✓ | |

---

## `withdraw_cosign`

Guarantor revokes a partial co-sign while loan is **Pending**. Clears their `pending_guarantees` entry and signature flag.

| Account | Mut | Signer |
|---------|-----|--------|
| guarantor | ✓ | ✓ |
| loan | ✓ | |
| guarantor_a_member, guarantor_b_member | ✓ | |

---

## `settle_default`

**Admin** initiates. Marks an **Active** loan as defaulted. **Both guarantors must sign** for SPL transfers.

- Splits outstanding 50/50 (odd amounts: first guarantor gets ceiling)
- Deducts shares from each guarantor's `savings_balance`
- CPI transfer from each guarantor ATA to vault
- Decrements `pool.total_savings` and `pool.total_outstanding_loans` by outstanding
- Clears matching borrower `active_loan`, releases guarantor `locked_savings`, and clears guarantor `active_guarantees`

| Account | Mut | Signer |
|---------|-----|--------|
| admin | | ✓ (must equal `pool.admin`) |
| pool | ✓ | |
| loan | ✓ | |
| guarantor_a, guarantor_b | | ✓ |
| borrower_member | ✓ | |
| guarantor_a_member, guarantor_b_member | ✓ | |
| vault | ✓ | |
| guarantor_a_token, guarantor_b_token | ✓ | |
| token_program | | |

Large account struct uses `Box<>` in the Anchor context to stay under BPF stack limits.

---

## `exit_pool`

Withdraws `savings_balance` from vault and closes member account.

**Blocked when:** `active_loan` set, `pending_loan` set, `locked_savings > 0`, non-empty `active_guarantees`, non-empty `pending_guarantees`, or `vault.amount < savings_balance` (`InsufficientVaultLiquidity`).

| Account | Mut | Signer |
|---------|-----|--------|
| member | ✓ | ✓ |
| member_account | ✓ | closed to member |
| pool | ✓ | |
| vault | ✓ | |
| member_token_account | ✓ | |
| token_program | | |
