//! Repayment arithmetic and borrower validation.

use std::result::Result;

use anchor_lang::prelude::{ProgramError, Pubkey};

use crate::error::PoolError;
use crate::state::LoanStatus;

/// Validates that the signer is the loan borrower.
///
/// # Arguments
///
/// * `signer` - Wallet invoking the instruction.
/// * `loan_borrower` - `loan.borrower` stored on chain.
pub fn validate_borrower(signer: Pubkey, loan_borrower: Pubkey) -> Result<(), PoolError> {
    if signer != loan_borrower {
        return Err(PoolError::NotLoanBorrower);
    }
    Ok(())
}

/// Validates loan status and repayment amount against outstanding principal.
///
/// # Arguments
///
/// * `status` - Current `loan.status` (must be [`LoanStatus::Active`]).
/// * `outstanding` - Remaining principal on the loan.
/// * `amount` - Repayment amount (must be > 0 and ≤ `outstanding`).
pub fn validate_repayment(
    status: LoanStatus,
    outstanding: u64,
    amount: u64,
) -> Result<(), PoolError> {
    if status != LoanStatus::Active {
        return Err(PoolError::LoanNotActive);
    }
    if amount == 0 {
        return Err(PoolError::RepaymentAmountMustBePositive);
    }
    if amount > outstanding {
        return Err(PoolError::RepaymentExceedsOutstanding);
    }
    Ok(())
}

/// Applies a repayment to the outstanding balance.
///
/// # Arguments
///
/// * `outstanding` - Principal before this repayment.
/// * `amount` - Tokens being repaid (must be ≤ `outstanding`).
///
/// # Returns
///
/// `(new_outstanding, fully_repaid)` - Updated balance and whether the loan is closed.
pub fn apply_repayment(outstanding: u64, amount: u64) -> Result<(u64, bool), ProgramError> {
    let new_outstanding = outstanding
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    Ok((new_outstanding, new_outstanding == 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrower_check() {
        let borrower = Pubkey::new_unique();
        assert!(validate_borrower(borrower, borrower).is_ok());
        assert_eq!(
            validate_borrower(Pubkey::new_unique(), borrower).unwrap_err(),
            PoolError::NotLoanBorrower
        );
    }

    #[test]
    fn repayment_validation() {
        assert!(validate_repayment(LoanStatus::Active, 1_000, 500).is_ok());
        assert_eq!(
            validate_repayment(LoanStatus::Repaid, 0, 1).unwrap_err(),
            PoolError::LoanNotActive
        );
        assert_eq!(
            validate_repayment(LoanStatus::Active, 100, 200).unwrap_err(),
            PoolError::RepaymentExceedsOutstanding
        );
        assert_eq!(
            validate_repayment(LoanStatus::Active, 100, 0).unwrap_err(),
            PoolError::RepaymentAmountMustBePositive
        );
    }

    #[test]
    fn apply_repayment_math() {
        assert_eq!(apply_repayment(1_000, 400).unwrap(), (600, false));
        assert_eq!(apply_repayment(500, 500).unwrap(), (0, true));
    }
}
