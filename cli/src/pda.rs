//! PDA derivation using `solana-sdk` types (compatible with the RPC client).

use kzp_mini::ID;
use kzp_mini::utils::seeds::{LOAN_SEED, MEMBER_SEED, POOL_SEED, VAULT_SEED};
use solana_sdk::pubkey::Pubkey;

fn program_id() -> Pubkey {
    Pubkey::new_from_array(ID.to_bytes())
}

/// Derives the pool PDA for a given admin and pool name.
pub fn pool_pda(admin: &Pubkey, pool_name: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[POOL_SEED, admin.as_ref(), pool_name.as_bytes()],
        &program_id(),
    )
}

/// Derives the token vault PDA for a pool.
pub fn vault_pda(pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[VAULT_SEED, pool.as_ref()], &program_id())
}

/// Derives the member PDA for a pool member wallet.
pub fn member_pda(pool: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MEMBER_SEED, pool.as_ref(), owner.as_ref()], &program_id())
}

/// Derives the loan PDA for a borrower and nonce.
pub fn loan_pda(pool: &Pubkey, borrower: &Pubkey, loan_nonce: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            LOAN_SEED,
            pool.as_ref(),
            borrower.as_ref(),
            &loan_nonce.to_le_bytes(),
        ],
        &program_id(),
    )
}
