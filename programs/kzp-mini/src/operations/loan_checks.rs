//! Loan request and co-sign validation logic.

use std::result::Result;

use anchor_lang::prelude::Pubkey;

use crate::constants::MAX_LOAN_MULTIPLIER;
use crate::error::PoolError;
use crate::operations::pool_ops::max_loan_for_savings;
use crate::state::Member;

/// Named inputs for loan-request validation.
#[derive(Debug, Clone, Copy)]
pub struct LoanRequestChecks {
    /// Requested principal (must be > 0 and ≤ multiplier × borrower savings).
    pub amount: u64,
    /// Borrower wallet pubkey.
    pub borrower: Pubkey,
    /// Borrower ledger balance used for the max-loan calculation.
    pub borrower_savings: u64,
    /// Whether `member.active_loan` is already set.
    pub borrower_has_active_loan: bool,
    /// Whether `member.pending_loan` is already set.
    pub borrower_has_pending_loan: bool,
    /// First nominated guarantor wallet.
    pub guarantor_a: Pubkey,
    /// Second nominated guarantor wallet.
    pub guarantor_b: Pubkey,
    /// Whether guarantor A is currently borrowing.
    pub guarantor_a_has_active_loan: bool,
    /// Whether guarantor B is currently borrowing.
    pub guarantor_b_has_active_loan: bool,
    /// Active + pending guarantee count for guarantor A.
    pub guarantor_a_guarantee_count: usize,
    /// Active + pending guarantee count for guarantor B.
    pub guarantor_b_guarantee_count: usize,
}

/// Validates all business rules for creating a new loan request.
pub fn validate_loan_request(checks: LoanRequestChecks) -> Result<(), PoolError> {
    if checks.amount == 0 {
        return Err(PoolError::LoanAmountMustBePositive);
    }
    if checks.borrower_has_active_loan {
        return Err(PoolError::ExistingActiveLoan);
    }
    if checks.borrower_has_pending_loan {
        return Err(PoolError::ExistingPendingLoan);
    }

    let max_loan = max_loan_for_savings(checks.borrower_savings, MAX_LOAN_MULTIPLIER)
        .ok_or(PoolError::LoanExceedsMaxMultiplier)?;
    if checks.amount > max_loan {
        return Err(PoolError::LoanExceedsMaxMultiplier);
    }

    if checks.guarantor_a == checks.borrower || checks.guarantor_b == checks.borrower {
        return Err(PoolError::SelfGuaranteeNotAllowed);
    }
    if checks.guarantor_a == checks.guarantor_b {
        return Err(PoolError::DuplicateGuarantors);
    }
    if checks.guarantor_a_has_active_loan || checks.guarantor_b_has_active_loan {
        return Err(PoolError::GuarantorHasActiveLoan);
    }
    if checks.guarantor_a_guarantee_count >= Member::MAX_GUARANTEES
        || checks.guarantor_b_guarantee_count >= Member::MAX_GUARANTEES
    {
        return Err(PoolError::GuarantorLimitReached);
    }

    Ok(())
}

/// Returns which guarantor slot the signer occupies, if any.
///
/// # Arguments
///
/// * `signer` - Wallet invoking a co-sign or withdraw instruction.
/// * `guarantor_a`, `guarantor_b` - Nominated guarantors stored on the loan account.
pub fn guarantor_slot(
    signer: Pubkey,
    guarantor_a: Pubkey,
    guarantor_b: Pubkey,
) -> Result<GuarantorSlot, PoolError> {
    if signer == guarantor_a {
        Ok(GuarantorSlot::A)
    } else if signer == guarantor_b {
        Ok(GuarantorSlot::B)
    } else {
        Err(PoolError::NotNominatedGuarantor)
    }
}

/// Guarantor position on a pending loan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuarantorSlot {
    /// First nominated guarantor (`loan.guarantor_a`).
    A,
    /// Second nominated guarantor (`loan.guarantor_b`).
    B,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> (Pubkey, Pubkey, Pubkey) {
        (
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        )
    }

    fn valid_request() -> LoanRequestChecks {
        let (borrower, guarantor_a, guarantor_b) = keys();
        LoanRequestChecks {
            amount: 300,
            borrower,
            borrower_savings: 1_000,
            borrower_has_active_loan: false,
            borrower_has_pending_loan: false,
            guarantor_a,
            guarantor_b,
            guarantor_a_has_active_loan: false,
            guarantor_b_has_active_loan: false,
            guarantor_a_guarantee_count: 0,
            guarantor_b_guarantee_count: 0,
        }
    }

    #[test]
    fn rejects_zero_amount() {
        let mut checks = valid_request();
        checks.amount = 0;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::LoanAmountMustBePositive
        );
    }

    #[test]
    fn rejects_excess_multiplier() {
        let mut checks = valid_request();
        checks.amount = 4_000;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::LoanExceedsMaxMultiplier
        );
    }

    #[test]
    fn rejects_self_guarantee() {
        let mut checks = valid_request();
        checks.guarantor_a = checks.borrower;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::SelfGuaranteeNotAllowed
        );
    }

    #[test]
    fn guarantor_slot_resolution() {
        let (a, b, other) = keys();
        assert_eq!(guarantor_slot(a, a, b).unwrap(), GuarantorSlot::A);
        assert_eq!(guarantor_slot(b, a, b).unwrap(), GuarantorSlot::B);
        assert_eq!(
            guarantor_slot(other, a, b).unwrap_err(),
            PoolError::NotNominatedGuarantor
        );
    }

    #[test]
    fn rejects_active_borrower_loan() {
        let mut checks = valid_request();
        checks.borrower_has_active_loan = true;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::ExistingActiveLoan
        );
    }

    #[test]
    fn rejects_pending_borrower_loan() {
        let mut checks = valid_request();
        checks.borrower_has_pending_loan = true;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::ExistingPendingLoan
        );
    }

    #[test]
    fn rejects_duplicate_guarantors() {
        let mut checks = valid_request();
        checks.guarantor_b = checks.guarantor_a;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::DuplicateGuarantors
        );
    }

    #[test]
    fn rejects_guarantor_with_active_loan() {
        let mut checks = valid_request();
        checks.guarantor_a_has_active_loan = true;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::GuarantorHasActiveLoan
        );
    }

    #[test]
    fn rejects_guarantor_limit() {
        let mut checks = valid_request();
        checks.guarantor_a_guarantee_count = Member::MAX_GUARANTEES;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::GuarantorLimitReached
        );
    }

    #[test]
    fn accepts_valid_request() {
        assert!(validate_loan_request(valid_request()).is_ok());
    }
}
