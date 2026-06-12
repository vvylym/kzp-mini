//! Custom program errors for pool, member, loan, and token validation failures.

use anchor_lang::prelude::*;

/// Errors returned by the KZP minimal mutual-aid pool program.
#[error_code]
#[derive(PartialEq, Eq)]
pub enum PoolError {
    /// Pool name is shorter than [`crate::constants::MIN_POOL_NAME_LEN`].
    #[msg("Pool name must be at least 3 characters")]
    PoolNameTooShort,
    /// `join_pool` entry fee does not match `pool.required_entry_fee`.
    #[msg("Entry fee does not match the pool requirement")]
    InvalidEntryFeeAmount,
    /// Member PDA already exists for this wallet in the pool.
    #[msg("You are already a member of this pool")]
    AlreadyMember,
    /// Deposit amount must be greater than zero.
    #[msg("Deposit amount must be greater than zero")]
    DepositAmountMustBePositive,
    /// Loan request amount must be greater than zero.
    #[msg("Loan amount must be greater than zero")]
    LoanAmountMustBePositive,
    /// Borrower already has a disbursed loan (`member.active_loan` is set).
    #[msg("You already have an active loan")]
    ExistingActiveLoan,
    /// Requested principal exceeds [`crate::constants::MAX_LOAN_MULTIPLIER`] × savings.
    #[msg("Loan exceeds maximum multiplier of your savings balance")]
    LoanExceedsMaxMultiplier,
    /// Borrower cannot nominate themselves as a guarantor.
    #[msg("You cannot guarantee your own loan")]
    SelfGuaranteeNotAllowed,
    /// `guarantor_a` and `guarantor_b` must be distinct pubkeys.
    #[msg("Guarantors must be different people")]
    DuplicateGuarantors,
    /// Nominated guarantor has no member account in this pool.
    #[msg("One of the guarantors is not a pool member")]
    GuarantorNotMember,
    /// Nominated guarantor is currently a borrower on another active loan.
    #[msg("A guarantor already has an active loan")]
    GuarantorHasActiveLoan,
    /// Guarantor would exceed [`Member::MAX_GUARANTEES`] active + pending obligations.
    #[msg("A guarantor has reached their maximum number of guarantees")]
    GuarantorLimitReached,
    /// Co-signer is not `loan.guarantor_a` or `loan.guarantor_b`.
    #[msg("You are not a nominated guarantor for this loan")]
    NotNominatedGuarantor,
    /// Guarantor already co-signed this pending loan.
    #[msg("You have already co-signed this loan")]
    AlreadyCoSigned,
    /// `withdraw_cosign` called before the guarantor co-signed.
    #[msg("You have not co-signed this loan")]
    NotCoSigned,
    /// Instruction requires `LoanStatus::Pending`.
    #[msg("This loan is not in pending status")]
    LoanNotPending,
    /// Instruction requires `LoanStatus::Active`.
    #[msg("This loan is not active")]
    LoanNotActive,
    /// Signer is not `loan.borrower`.
    #[msg("You are not the borrower of this loan")]
    NotLoanBorrower,
    /// Repayment amount exceeds `loan.outstanding`.
    #[msg("Repayment exceeds outstanding balance")]
    RepaymentExceedsOutstanding,
    /// Repayment amount must be greater than zero.
    #[msg("Repayment amount must be positive")]
    RepaymentAmountMustBePositive,
    /// Member has a disbursed loan and cannot exit.
    #[msg("You have an outstanding loan")]
    OutstandingLoanExists,
    /// Member guarantees active loans and cannot exit.
    #[msg("You are a guarantor on active loans")]
    ActiveGuaranteesExist,
    /// Member has pending co-sign obligations and cannot exit.
    #[msg("You have pending co-sign obligations")]
    PendingGuaranteesExist,
    /// Provided vault account is not the pool's canonical vault PDA.
    #[msg("Vault account does not match this pool")]
    InvalidVaultAccount,
    /// Vault SPL balance is less than the requested transfer amount.
    #[msg("Vault has insufficient liquidity for this disbursement")]
    InsufficientVaultLiquidity,
    /// Signer is not `pool.admin`.
    #[msg("Only the pool admin may perform this action")]
    NotPoolAdmin,
    /// Guarantor `savings_balance` is below their default settlement share.
    #[msg("Guarantor savings are insufficient to cover default share")]
    GuarantorInsufficientSavings,
    /// Guarantor token ATA balance is below their default settlement share.
    #[msg("Guarantor token balance is insufficient to cover default share")]
    GuarantorInsufficientTokens,
}
