//! PDA seed constants used across account constraints and CPI signers.

/// Seed for the pool state account: `["pool", admin, pool_name]`.
pub const POOL_SEED: &[u8] = b"pool";

/// Seed for the SPL token vault: `["vault", pool]`.
pub const VAULT_SEED: &[u8] = b"vault";

/// Seed for a member account: `["member", pool, owner]`.
pub const MEMBER_SEED: &[u8] = b"member";

/// Seed for a loan account: `["loan", pool, borrower, loan_nonce]`.
pub const LOAN_SEED: &[u8] = b"loan";
