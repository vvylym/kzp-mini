//! Vault liquidity checks shared by disbursement and exit flows.

use std::result::Result;

use crate::error::PoolError;

/// Ensures the vault holds enough tokens for a disbursement or member exit payout.
pub fn validate_vault_liquidity(vault_amount: u64, required_amount: u64) -> Result<(), PoolError> {
    if vault_amount < required_amount {
        return Err(PoolError::InsufficientVaultLiquidity);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_insufficient_vault_liquidity() {
        assert!(validate_vault_liquidity(1_000, 1_000).is_ok());
        assert_eq!(
            validate_vault_liquidity(999, 1_000).unwrap_err(),
            PoolError::InsufficientVaultLiquidity
        );
    }
}
