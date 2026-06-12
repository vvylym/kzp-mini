//! Program-derived address helpers for off-chain clients and tests.

use anchor_lang::prelude::*;

use super::seeds::{LOAN_SEED, MEMBER_SEED, POOL_SEED, VAULT_SEED};
use crate::ID;

/// Derives the pool PDA for a given admin and pool name.
///
/// # Arguments
///
/// * `admin` - Pool creator wallet (seed component).
/// * `pool_name` - UTF-8 pool identifier (seed component).
pub fn pool_pda(admin: &Pubkey, pool_name: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[POOL_SEED, admin.as_ref(), pool_name.as_bytes()], &ID)
}

/// Derives the token vault PDA for a pool.
///
/// # Arguments
///
/// * `pool` - Pool state account pubkey.
pub fn vault_pda(pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[VAULT_SEED, pool.as_ref()], &ID)
}

/// Derives the member PDA for a pool member wallet.
///
/// # Arguments
///
/// * `pool` - Pool state account pubkey.
/// * `owner` - Member wallet pubkey.
pub fn member_pda(pool: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MEMBER_SEED, pool.as_ref(), owner.as_ref()], &ID)
}

/// Derives the loan PDA for a borrower and nonce.
///
/// # Arguments
///
/// * `pool` - Pool state account pubkey.
/// * `borrower` - Borrower wallet pubkey.
/// * `loan_nonce` - Per-borrower loan disambiguator (little-endian in seeds).
pub fn loan_pda(pool: &Pubkey, borrower: &Pubkey, loan_nonce: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            LOAN_SEED,
            pool.as_ref(),
            borrower.as_ref(),
            &loan_nonce.to_le_bytes(),
        ],
        &ID,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdas_are_deterministic() {
        let admin = Pubkey::new_unique();
        let (pool, bump) = pool_pda(&admin, "test-pool");
        let (pool2, bump2) = pool_pda(&admin, "test-pool");
        assert_eq!(pool, pool2);
        assert_eq!(bump, bump2);

        let (vault, _) = vault_pda(&pool);
        let owner = Pubkey::new_unique();
        let (member, _) = member_pda(&pool, &owner);
        let (loan, _) = loan_pda(&pool, &owner, 42);
        assert_ne!(vault, pool);
        assert_ne!(member, vault);
        assert_ne!(loan, member);
    }
}
