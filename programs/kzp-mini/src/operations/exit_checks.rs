//! Member exit eligibility checks.

use std::result::Result;

use anchor_lang::prelude::Pubkey;

use crate::error::PoolError;

/// Validates that a member has no outstanding loan or guarantee obligations before exit.
///
/// # Arguments
///
/// * `active_loan` - Borrower's disbursed loan PDA, if any.
/// * `pending_loan` - Borrower's pending loan PDA, if any.
/// * `locked_savings` - Savings reserved for active guarantor liability.
/// * `active_guarantees` - Disbursed loans this member guarantees.
/// * `pending_guarantees` - Pending loans where this member has co-signed.
pub fn validate_exit_eligible(
    active_loan: Option<Pubkey>,
    pending_loan: Option<Pubkey>,
    locked_savings: u64,
    active_guarantees: &[Pubkey],
    pending_guarantees: &[Pubkey],
) -> Result<(), PoolError> {
    if active_loan.is_some() {
        return Err(PoolError::OutstandingLoanExists);
    }
    if pending_loan.is_some() {
        return Err(PoolError::OutstandingLoanExists);
    }
    if locked_savings > 0 {
        return Err(PoolError::ActiveGuaranteesExist);
    }
    if !active_guarantees.is_empty() {
        return Err(PoolError::ActiveGuaranteesExist);
    }
    if !pending_guarantees.is_empty() {
        return Err(PoolError::PendingGuaranteesExist);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_rules() {
        let loan = Pubkey::new_unique();
        let guarantee = Pubkey::new_unique();

        assert!(validate_exit_eligible(None, None, 0, &[], &[]).is_ok());
        assert_eq!(
            validate_exit_eligible(Some(loan), None, 0, &[], &[]).unwrap_err(),
            PoolError::OutstandingLoanExists
        );
        assert_eq!(
            validate_exit_eligible(None, Some(loan), 0, &[], &[]).unwrap_err(),
            PoolError::OutstandingLoanExists
        );
        assert_eq!(
            validate_exit_eligible(None, None, 1, &[], &[]).unwrap_err(),
            PoolError::ActiveGuaranteesExist
        );
        assert_eq!(
            validate_exit_eligible(None, None, 0, &[guarantee], &[]).unwrap_err(),
            PoolError::ActiveGuaranteesExist
        );
        assert_eq!(
            validate_exit_eligible(None, None, 0, &[], &[guarantee]).unwrap_err(),
            PoolError::PendingGuaranteesExist
        );
    }
}
