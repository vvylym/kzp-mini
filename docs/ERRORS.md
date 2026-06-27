# Error Codes

Defined in `programs/kzp-mini/src/error.rs` as `PoolError`.

| Code | Name | When |
|------|------|------|
| 6000 | PoolNameTooShort | Pool name &lt; 3 chars |
| 6001 | InvalidEntryFeeAmount | Join fee mismatch |
| 6002 | AlreadyMember | Duplicate join |
| 6003 | DepositAmountMustBePositive | Zero deposit |
| 6004 | LoanAmountMustBePositive | Zero loan |
| 6005 | ExistingActiveLoan | Borrower has active loan |
| 6006 | LoanExceedsMaxMultiplier | Amount &gt; 3× savings |
| 6007 | SelfGuaranteeNotAllowed | Borrower nominated as guarantor |
| 6008 | DuplicateGuarantors | Same guarantor twice |
| 6009 | GuarantorNotMember | Guarantor not in pool |
| 6010 | GuarantorHasActiveLoan | Guarantor is borrowing |
| 6011 | GuarantorLimitReached | ≥ 5 active + pending guarantees |
| 6012 | NotNominatedGuarantor | Signer not on loan |
| 6013 | AlreadyCoSigned | Duplicate co-sign |
| 6014 | NotCoSigned | Withdraw without prior co-sign |
| 6015 | LoanNotPending | Action requires Pending status |
| 6016 | LoanNotActive | Action requires Active status |
| 6017 | NotLoanBorrower | Wrong borrower |
| 6018 | RepaymentExceedsOutstanding | Over-payment |
| 6019 | RepaymentAmountMustBePositive | Zero repayment |
| 6020 | OutstandingLoanExists | Exit with active loan |
| 6021 | ActiveGuaranteesExist | Exit with active guarantees |
| 6022 | PendingGuaranteesExist | Exit with pending co-signs |
| 6023 | InvalidVaultAccount | Vault PDA mismatch |
| 6024 | InsufficientVaultLiquidity | Vault under-funded at disbursement or exit |
| 6025 | NotPoolAdmin | Non-admin default settlement |
| 6026 | GuarantorInsufficientSavings | Default share exceeds savings ledger |
| 6027 | GuarantorInsufficientTokens | Default share exceeds guarantor ATA balance |
| 6028 | PoolNameTooLong | Pool name exceeds 32-byte PDA seed limit |
| 6029 | ExistingPendingLoan | Borrower has pending loan |

Anchor assigns base 6000 for custom errors. Verify against the IDL after deploy.

Integration tests assert via `6000 + PoolError as u32` (see `tests/src/helpers/constants.rs`).
