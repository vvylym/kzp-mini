# Financial Model

KZP Mini tracks two parallel views of pool money: the **SPL vault** (actual tokens) and the **ledger** (member and pool counters).

## Savings ledger

- Each **Member** PDA holds `savings_balance` — withdrawable on `exit_pool`.
- **Pool** PDA holds `total_savings` — sum of member savings balances.
- `deposit_savings` increases both member balance and `total_savings`.

## Vault accounting

- The **vault** PDA holds real SPL tokens.
- Disbursements, repayments, entry fees, and exits move tokens via CPI.
- `pool.total_outstanding_loans` tracks active loan principal (updated on disburse, repay, default).

## Entry fees

- Paid on `join_pool` into the vault.
- Credited to the vault but **not** to `member.savings_balance`.
- Leaves a small **liquidity buffer** for disburse → repay → exit cycles.

## Loan limits

- Maximum principal: **3×** borrower `savings_balance` (`MAX_LOAN_MULTIPLIER`).
- One **active loan** per borrower.
- Up to **5** concurrent guarantees per guarantor (active + pending combined).

## Guarantee liability

- While **Pending:** guarantor obligations live in `pending_guarantees`; either party can unwind via `cancel_loan` / `withdraw_cosign`.
- After disbursement:** obligations move to `active_guarantees`; guarantors cannot exit until cleared.

## Default settlement

- Admin calls `settle_default` on an **Active** loan.
- Outstanding principal split **50/50** between guarantors (`div_ceil` on odd amounts).
- Each guarantor debited on the savings **ledger** and must sign SPL transfer to the vault.

## Fail-closed checks

| Instruction | Check |
|-------------|-------|
| `co_sign_loan` | `vault.amount >= principal` before disburse |
| `exit_pool` | `vault.amount >= member.savings_balance` before payout |

If ledger and vault diverge, exit and disburse are blocked rather than over-paying.
