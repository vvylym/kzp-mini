//! Member exit eligibility checks.

use std::result::Result;

use crate::error::PoolError;

/// Validates that a member has no outstanding loan or guarantee obligations before exit.
///
/// # Arguments
///
/// * `active_loan` - Borrower's disbursed loan PDA, if any.
/// * `pending_loan` - Borrower's pending loan PDA, if any.
/// * `locked_savings` - Savings reserved for active guarantor liability.
/// * `active_guarantee_count` - Number of disbursed loans this member guarantees.
/// * `pending_guarantee_count` - Number of pending loans where this member has co-signed.
pub fn validate_exit_eligible(
    active_loan_is_some: bool,
    pending_loan_is_some: bool,
    locked_savings: u64,
    active_guarantee_count: u8,
    pending_guarantee_count: u8,
) -> Result<(), PoolError> {
    if active_loan_is_some {
        return Err(PoolError::OutstandingLoanExists);
    }
    if pending_loan_is_some {
        return Err(PoolError::OutstandingLoanExists);
    }
    if locked_savings > 0 {
        return Err(PoolError::ActiveGuaranteesExist);
    }
    if active_guarantee_count > 0 {
        return Err(PoolError::ActiveGuaranteesExist);
    }
    if pending_guarantee_count > 0 {
        return Err(PoolError::PendingGuaranteesExist);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_rules() {
        assert!(validate_exit_eligible(false, false, 0, 0, 0).is_ok());
        assert_eq!(
            validate_exit_eligible(true, false, 0, 0, 0).unwrap_err(),
            PoolError::OutstandingLoanExists
        );
        assert_eq!(
            validate_exit_eligible(false, true, 0, 0, 0).unwrap_err(),
            PoolError::OutstandingLoanExists
        );
        assert_eq!(
            validate_exit_eligible(false, false, 1, 0, 0).unwrap_err(),
            PoolError::ActiveGuaranteesExist
        );
        assert_eq!(
            validate_exit_eligible(false, false, 0, 1, 0).unwrap_err(),
            PoolError::ActiveGuaranteesExist
        );
        assert_eq!(
            validate_exit_eligible(false, false, 0, 0, 1).unwrap_err(),
            PoolError::PendingGuaranteesExist
        );
    }
}
