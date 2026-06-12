# Use Cases and Acceptance Criteria

BDD-style scenarios. **46 integration tests** in `tests/src/test_*.rs` mirror these specs (see [tests/README.md](../tests/README.md)).

---

## UC-1: Pool bootstrap

**Actor:** Admin

### Nominal - initialize pool

- **Given** a funded admin wallet and valid SPL mint
- **When** admin calls `initialize_pool` with name ≥ 3 chars and entry fee
- **Then** pool and vault PDAs exist, `total_outstanding_loans = 0`

### Edge - name too short

- **Given** admin wallet
- **When** pool name has fewer than 3 characters
- **Then** `PoolNameTooShort`

---

## UC-2: Membership

**Actor:** Member

### Nominal - join and deposit

- **Given** an initialized pool
- **When** user pays exact entry fee via `join_pool`, then `deposit_savings`
- **Then** member PDA exists, `savings_balance` reflects deposit, vault balance increases

### Edge - wrong entry fee

- **When** join fee ≠ `required_entry_fee`
- **Then** `InvalidEntryFeeAmount`

---

## UC-3: Loan request

**Actor:** Borrower

### Nominal - request within limit

- **Given** borrower with savings S, two eligible guarantors
- **When** `request_loan` with amount ≤ 3×S
- **Then** loan is `Pending`, both guarantors unsigned

### Edge - exceeds multiplier

- **When** amount &gt; 3× savings
- **Then** `LoanExceedsMaxMultiplier`

### Edge - self-guarantee / duplicate guarantors

- **Then** `SelfGuaranteeNotAllowed` or `DuplicateGuarantors`

---

## UC-4: Co-sign and disbursement

**Actors:** Guarantor A, Guarantor B

### Nominal - partial then full

- **Given** pending loan
- **When** guarantor A co-signs
- **Then** loan stays `Pending`, A has `pending_guarantees`, no token movement
- **When** guarantor B co-signs
- **Then** loan → `Active`, both have `active_guarantees`, borrower receives principal, `total_outstanding_loans` increases

### Edge - not nominated guarantor

- **Then** `NotNominatedGuarantor`

### Edge - double co-sign

- **Then** `AlreadyCoSigned`

### Edge - co-sign active loan

- **Then** `LoanNotPending`

### Edge - insufficient vault liquidity

- **Given** vault balance &lt; principal
- **When** second co-sign would disburse
- **Then** `InsufficientVaultLiquidity`

---

## UC-5: Cancel pending loan

**Actor:** Borrower

### Nominal

- **Given** pending loan (with or without partial co-signs)
- **When** borrower calls `cancel_loan`
- **Then** loan account closed, guarantor `pending_guarantees` cleared

### Edge - cancel active loan

- **Then** `LoanNotPending`

---

## UC-6: Withdraw partial co-sign

**Actor:** Guarantor

### Nominal

- **Given** guarantor co-signed pending loan
- **When** `withdraw_cosign`
- **Then** signature flag cleared, `pending_guarantees` entry removed

### Edge - withdraw without co-signing

- **Then** `NotCoSigned`

---

## UC-7: Repayment

**Actor:** Borrower

### Nominal - partial and full

- **Given** active loan
- **When** borrower repays amount ≤ outstanding
- **Then** vault increases, `total_outstanding_loans` decreases
- **When** final payment clears outstanding
- **Then** loan → `Repaid`, `active_loan` and guarantor `active_guarantees` cleared

### Edge - over-repay

- **Then** `RepaymentExceedsOutstanding`

---

## UC-8: Default settlement

**Actors:** Pool admin, guarantors

### Nominal

- **Given** active loan with outstanding O, each guarantor savings ≥ share and ATAs funded
- **When** admin calls `settle_default` with both guarantors signing
- **Then** loan → `Defaulted`, savings reduced 50/50, vault receives O tokens, counters updated

### Edge - non-admin

- **Then** `NotPoolAdmin`

### Edge - insufficient guarantor savings or tokens

- **Then** `GuarantorInsufficientSavings` or `GuarantorInsufficientTokens`

---

## UC-9: Exit pool

**Actor:** Member

### Nominal

- **Given** member with savings B, vault balance ≥ B, no obligations
- **When** `exit_pool`
- **Then** savings returned, member PDA closed

### Edge - vault under-funded

- **Given** vault liquidity tied up or ledger/vault drift
- **When** member calls `exit_pool` with `savings_balance` > `vault.amount`
- **Then** `InsufficientVaultLiquidity`

### Edge - outstanding loan

- **Then** `OutstandingLoanExists`

### Edge - active guarantees

- **Then** `ActiveGuaranteesExist`

### Edge - pending co-sign obligations

- **Given** member co-signed a pending loan
- **When** `exit_pool`
- **Then** `PendingGuaranteesExist`
