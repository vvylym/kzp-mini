//! Pool-level validation helpers.

use std::result::Result;

use crate::constants::MIN_POOL_NAME_LEN;
use crate::error::PoolError;

/// Validates that a pool name meets the minimum length requirement.
pub fn validate_pool_name(pool_name: &str) -> Result<(), PoolError> {
    if pool_name.len() < MIN_POOL_NAME_LEN {
        return Err(PoolError::PoolNameTooShort);
    }
    Ok(())
}

/// Validates that a deposit or repayment amount is strictly positive.
pub fn validate_positive_amount(amount: u64) -> Result<(), PoolError> {
    if amount == 0 {
        return Err(PoolError::DepositAmountMustBePositive);
    }
    Ok(())
}

/// Validates that a loan request amount is strictly positive.
pub fn validate_loan_amount(amount: u64) -> Result<(), PoolError> {
    if amount == 0 {
        return Err(PoolError::LoanAmountMustBePositive);
    }
    Ok(())
}

/// Validates that a repayment amount is strictly positive.
pub fn validate_repayment_amount(amount: u64) -> Result<(), PoolError> {
    if amount == 0 {
        return Err(PoolError::RepaymentAmountMustBePositive);
    }
    Ok(())
}

/// Computes the maximum loan principal allowed for a member's savings balance.
///
/// Returns `None` on overflow (`savings_balance * multiplier`).
pub fn max_loan_for_savings(savings_balance: u64, multiplier: u64) -> Option<u64> {
    savings_balance.checked_mul(multiplier)
}

/// Ensures the vault holds enough tokens to disburse a loan principal.
///
/// # Arguments
///
/// * `vault_amount` - Current SPL balance of the pool vault.
/// * `principal` - Tokens required for disbursement or member exit payout.
pub fn validate_vault_liquidity(vault_amount: u64, principal: u64) -> Result<(), PoolError> {
    if vault_amount < principal {
        return Err(PoolError::InsufficientVaultLiquidity);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_name_validation() {
        assert!(validate_pool_name("ab").is_err());
        assert!(validate_pool_name("abc").is_ok());
    }

    #[test]
    fn amount_validations() {
        assert!(validate_positive_amount(0).is_err());
        assert!(validate_loan_amount(0).is_err());
        assert!(validate_repayment_amount(0).is_err());
        assert!(validate_positive_amount(1).is_ok());
    }

    #[test]
    fn max_loan_math() {
        assert_eq!(max_loan_for_savings(1_000, 3), Some(3_000));
        assert_eq!(max_loan_for_savings(u64::MAX, 2), None);
    }

    #[test]
    fn vault_liquidity_for_exit() {
        assert!(validate_vault_liquidity(1_000, 1_000).is_ok());
        assert_eq!(
            validate_vault_liquidity(999, 1_000).unwrap_err(),
            PoolError::InsufficientVaultLiquidity
        );
    }
}
