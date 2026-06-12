//! Handler for [`crate::initialize_pool`].

use anchor_lang::prelude::*;

use crate::instructions::InitializePool;
use crate::operations::validate_pool_name;

/// Initializes pool and vault state after validating the pool name.
pub fn handle(
    ctx: Context<InitializePool>,
    pool_name: String,
    required_entry_fee: u64,
) -> Result<()> {
    validate_pool_name(&pool_name)?;

    let pool = &mut ctx.accounts.pool;
    pool.admin = ctx.accounts.admin.key();
    pool.token_mint = ctx.accounts.token_mint.key();
    pool.vault = ctx.accounts.vault.key();
    pool.required_entry_fee = required_entry_fee;
    pool.total_members = 0;
    pool.total_savings = 0;
    pool.total_outstanding_loans = 0;
    pool.bump = ctx.bumps.pool;
    pool.vault_bump = ctx.bumps.vault;

    Ok(())
}
