# Use Cases and Acceptance Criteria

BDD-style scenarios. **55 integration tests** in `tests/src/test_*.rs` mirror these specs (see [tests/README.md](../tests/README.md)).

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

### Edge - name too long

- **Given** a funded admin wallet
- **When** pool name exceeds the maximum seed-safe length
- **Then** initialization is rejected before account creation

### Boundary - maximum length name

- **Given** a funded admin wallet
- **When** pool name is exactly the maximum supported length
- **Then** pool and vault PDAs initialize successfully

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

### Safety - checked member count

- **Given** a pool near the representable member-count limit
- **When** a member joins and the counter would overflow
- **Then** the transaction fails instead of wrapping the counter

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

### Edge - borrower has pending loan

- **Given** borrower already has a `Pending` loan
- **When** borrower requests another loan with a different nonce
- **Then** the request fails with the existing-loan error

### Nominal - pending reservation clears after cancel

- **Given** borrower has a `Pending` loan
- **When** borrower cancels that loan
- **Then** borrower may request a new loan

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

### Edge - stale borrower eligibility at activation

- **Given** borrower opened a pending loan and then gained another unresolved loan
- **When** the pending loan receives the second co-sign
- **Then** activation fails and cannot overwrite borrower loan state

### Edge - stale guarantor eligibility at activation

- **Given** a nominated guarantor was eligible at request time
- **When** that guarantor becomes an active borrower before activation
- **Then** activation fails with the guarantor active-loan error

### Edge - insufficient guarantor unlocked savings

- **Given** a pending loan whose guarantor share exceeds a guarantor's unlocked savings
- **When** the second co-sign would activate the loan
- **Then** activation fails before disbursement

### Nominal - guarantor liability reserved on activation

- **Given** both guarantors have enough unlocked savings for their shares
- **When** the second co-sign activates the loan
- **Then** both guarantor shares are reserved until repayment or default

---

## UC-5: Cancel pending loan

**Actor:** Borrower

### Nominal

- **Given** pending loan (with or without partial co-signs)
- **When** borrower calls `cancel_loan`
- **Then** loan account closed, guarantor `pending_guarantees` cleared

### Edge - cancel active loan

- **Then** `LoanNotPending`

### Edge - non-borrower cancel

- **Given** a pending loan owned by borrower
- **When** another signer calls `cancel_loan`
- **Then** `NotLoanBorrower`

### Nominal - cancel unsigned pending loan

- **Given** a pending loan with no co-signatures
- **When** borrower calls `cancel_loan`
- **Then** loan closes and borrower pending reservation clears

---

## UC-6: Withdraw partial co-sign

**Actor:** Guarantor

### Nominal

- **Given** guarantor co-signed pending loan
- **When** `withdraw_cosign`
- **Then** signature flag cleared, `pending_guarantees` entry removed

### Edge - withdraw without co-signing

- **Then** `NotCoSigned`

### Edge - withdraw active loan co-sign

- **Given** guarantor co-signed a loan that has become `Active`
- **When** guarantor calls `withdraw_cosign`
- **Then** `LoanNotPending`

### Edge - withdraw by non-nominated member

- **Given** a pending loan with two nominated guarantors
- **When** a different member calls `withdraw_cosign`
- **Then** `NotNominatedGuarantor`

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

### Edge - default before due date

- **Given** an active loan whose due date has not passed
- **When** admin calls `settle_default`
- **Then** settlement fails with the due-date error

### Nominal - default after due date from reserved liability

- **Given** an active loan past due with guarantor shares already reserved
- **When** admin calls `settle_default`
- **Then** loan → `Defaulted`, outstanding is zeroed, reserved guarantor savings are debited, and no fresh guarantor signatures are required

### Edge - odd outstanding split

- **Given** an active loan with odd outstanding amount
- **When** admin settles default after due date
- **Then** guarantor A pays the ceiling share, guarantor B pays the remainder, and total debits equal outstanding

### Safety - default clears only matching loan

- **Given** borrower member state references an active loan
- **When** admin settles that loan as defaulted
- **Then** borrower `active_loan` is cleared only if it matches the settled loan

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

### Edge - reserved guarantor liability

- **Given** member has reserved savings for an active guarantee
- **When** `exit_pool`
- **Then** exit is blocked until the loan is repaid or defaulted
