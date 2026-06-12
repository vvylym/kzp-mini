# System Architecture

On-chain workplace mutual-aid pool: coworkers save together and borrow with **two guarantors** who accept partial liability on default.

The program enforces membership state, savings, loan limits (3× savings), co-sign before disbursement, repayment, cancellation, admin default settlement, and exit only when obligations are clear. **Who may join** and **when a default is justified** stay off-chain; on-chain code enforces **mechanics and fund safety**.

## Account hierarchy

```
                    Pool PDA
                        │
                        ▼
                   Vault PDA
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
   Member PDA      Member PDA      Member PDA
        │               │               │
        ▼               │               │
    Loan PDA            │               │
                        │               │
              (guarantor members) ──────┘
```

## Core accounts

| Account | Seeds | Role |
|---------|-------|------|
| **Pool** | `["pool", admin, pool_name]` | Config, aggregate counters |
| **Vault** | `["vault", pool]` | SPL token custody |
| **Member** | `["member", pool, owner]` | Savings ledger, obligations |
| **Loan** | `["loan", pool, borrower, loan_nonce_le]` | Pending → Active → Repaid / Defaulted |

PDA helpers: `kzp_mini::utils::pda` · CLI: `cli/src/pda.rs`.

## Instructions

| Instruction | Purpose |
|-------------|---------|
| `initialize_pool` | Admin creates pool + SPL vault |
| `join_pool` | Pay entry fee, open member PDA |
| `deposit_savings` | Credit savings; tokens to vault |
| `request_loan` | Open pending loan (≤ 3× savings, two guarantors) |
| `co_sign_loan` | Guarantor co-signs; disburses when both sign |
| `repay_loan` | Partial or full repayment |
| `cancel_loan` | Borrower cancels pending loan |
| `withdraw_cosign` | Guarantor revokes partial co-sign |
| `settle_default` | Admin default; 50/50 guarantor split |
| `exit_pool` | Withdraw savings; close member PDA |

Full account metas: [INSTRUCTIONS.md](./INSTRUCTIONS.md). Field layouts: [STATE.md](./STATE.md).

## Traditional ↔ on-chain mapping

| Traditional concept | On-chain implementation |
|---------------------|-------------------------|
| Pool / cash box | **Pool** PDA + **vault** PDA |
| Member record | **Member** PDA (`savings_balance`, obligations) |
| Loan application | **Loan** PDA lifecycle |
| Two guarantors | `guarantor_a` / `guarantor_b`; both co-sign to disburse |
| Partial approval | `guarantor_*_signed` + `pending_guarantees` |
| Disbursement | `co_sign_loan` CPI when vault ≥ principal |
| Repayment | `repay_loan` CPI + counter updates |
| Default | `settle_default` — 50/50 ledger + SPL from guarantors |
| Leave pool | `exit_pool` pays savings, closes PDA |

## Code layout

```
programs/kzp-mini/src/
├── lib.rs              #[program] dispatch → handlers::*::handle
├── state.rs            Pool, Member, Loan, LoanStatus
├── instructions/       Anchor account constraints only
├── handlers/           CPIs + state updates; one handle per instruction
├── operations/         Pure rules/math (unit-tested)
└── utils/              PDA seeds and helpers
```

**Design split:** `instructions/` = validation · `handlers/` = orchestration · `operations/` = pure business rules testable without Anchor contexts.

The file `instructions/withdraw_co_sign.rs` maps to the `withdraw_cosign` instruction name.

## Loan lifecycle

```
Pending → Active → Repaid
                 ↘ Defaulted
Pending → cancel_loan (account closed)
```

| Actor | Escape hatch while Pending |
|-------|----------------------------|
| Borrower | `cancel_loan` |
| Guarantor | `withdraw_cosign` |

**`pending_guarantees`** vs **`active_guarantees`:** partial co-sign only adds pending obligations; active guarantees are set after both co-signs and successful disbursement.

See [ACCOUNTING.md](./ACCOUNTING.md) for ledger vs vault. See [SECURITY.md](./SECURITY.md) for trust boundaries.
