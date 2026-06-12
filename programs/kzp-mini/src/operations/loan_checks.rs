//! Loan request and co-sign validation logic.

use std::result::Result;

use anchor_lang::prelude::Pubkey;

use crate::constants::MAX_LOAN_MULTIPLIER;
use crate::error::PoolError;
use crate::operations::pool_ops::max_loan_for_savings;
use crate::state::Member;

/// Validates all business rules for creating a new loan request.
///
/// # Arguments
///
/// * `amount` - Requested principal (must be > 0 and ≤ `multiplier` × `borrower_savings`).
/// * `borrower` - Borrower wallet pubkey (checked against self-guarantee rules).
/// * `borrower_savings` - Borrower ledger balance used for the max-loan calculation.
/// * `borrower_has_active_loan` - Whether `member.active_loan` is already set.
/// * `guarantor_a`, `guarantor_b` - Nominated guarantor pubkeys (must differ from borrower and each other).
/// * `guarantor_a_has_active_loan`, `guarantor_b_has_active_loan` - Whether each guarantor is currently borrowing.
/// * `guarantor_a_guarantee_count`, `guarantor_b_guarantee_count` - Active + pending guarantee count per guarantor.
#[allow(clippy::too_many_arguments)]
pub fn validate_loan_request(
    amount: u64,
    borrower: Pubkey,
    borrower_savings: u64,
    borrower_has_active_loan: bool,
    guarantor_a: Pubkey,
    guarantor_b: Pubkey,
    guarantor_a_has_active_loan: bool,
    guarantor_b_has_active_loan: bool,
    guarantor_a_guarantee_count: usize,
    guarantor_b_guarantee_count: usize,
) -> Result<(), PoolError> {
    if amount == 0 {
        return Err(PoolError::LoanAmountMustBePositive);
    }
    if borrower_has_active_loan {
        return Err(PoolError::ExistingActiveLoan);
    }

    let max_loan = max_loan_for_savings(borrower_savings, MAX_LOAN_MULTIPLIER)
        .ok_or(PoolError::LoanExceedsMaxMultiplier)?;
    if amount > max_loan {
        return Err(PoolError::LoanExceedsMaxMultiplier);
    }

    if guarantor_a == borrower || guarantor_b == borrower {
        return Err(PoolError::SelfGuaranteeNotAllowed);
    }
    if guarantor_a == guarantor_b {
        return Err(PoolError::DuplicateGuarantors);
    }
    if guarantor_a_has_active_loan || guarantor_b_has_active_loan {
        return Err(PoolError::GuarantorHasActiveLoan);
    }
    if guarantor_a_guarantee_count >= Member::MAX_GUARANTEES
        || guarantor_b_guarantee_count >= Member::MAX_GUARANTEES
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

    #[test]
    fn rejects_zero_amount() {
        let (borrower, ga, gb) = keys();
        assert_eq!(
            validate_loan_request(0, borrower, 1_000, false, ga, gb, false, false, 0, 0)
                .unwrap_err(),
            PoolError::LoanAmountMustBePositive
        );
    }

    #[test]
    fn rejects_excess_multiplier() {
        let (borrower, ga, gb) = keys();
        assert_eq!(
            validate_loan_request(4_000, borrower, 1_000, false, ga, gb, false, false, 0, 0)
                .unwrap_err(),
            PoolError::LoanExceedsMaxMultiplier
        );
    }

    #[test]
    fn rejects_self_guarantee() {
        let (borrower, ga, _) = keys();
        assert_eq!(
            validate_loan_request(100, borrower, 1_000, false, borrower, ga, false, false, 0, 0)
                .unwrap_err(),
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
        let (borrower, ga, gb) = keys();
        assert_eq!(
            validate_loan_request(100, borrower, 1_000, true, ga, gb, false, false, 0, 0)
                .unwrap_err(),
            PoolError::ExistingActiveLoan
        );
    }

    #[test]
    fn rejects_duplicate_guarantors() {
        let (borrower, ga, _) = keys();
        assert_eq!(
            validate_loan_request(100, borrower, 1_000, false, ga, ga, false, false, 0, 0)
                .unwrap_err(),
            PoolError::DuplicateGuarantors
        );
    }

    #[test]
    fn rejects_guarantor_with_active_loan() {
        let (borrower, ga, gb) = keys();
        assert_eq!(
            validate_loan_request(100, borrower, 1_000, false, ga, gb, true, false, 0, 0)
                .unwrap_err(),
            PoolError::GuarantorHasActiveLoan
        );
    }

    #[test]
    fn rejects_guarantor_limit() {
        let (borrower, ga, gb) = keys();
        assert_eq!(
            validate_loan_request(
                100,
                borrower,
                1_000,
                false,
                ga,
                gb,
                false,
                false,
                Member::MAX_GUARANTEES,
                0
            )
            .unwrap_err(),
            PoolError::GuarantorLimitReached
        );
    }

    #[test]
    fn accepts_valid_request() {
        let (borrower, ga, gb) = keys();
        assert!(
            validate_loan_request(300, borrower, 1_000, false, ga, gb, false, false, 0, 0).is_ok()
        );
    }
}
