//! Accounts for [`crate::initialize_pool`].

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::Pool;
use crate::utils::seeds::{POOL_SEED, VAULT_SEED};

/// Accounts required to create a new pool and its SPL token vault.
#[derive(Accounts)]
#[instruction(pool_name: String, required_entry_fee: u64)]
pub struct InitializePool<'info> {
    /// Admin wallet; pays rent and is stored as `pool.admin`.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Pool state PDA: `["pool", admin, pool_name]`.
    #[account(
        init,
        payer = admin,
        space = Pool::LEN,
        seeds = [POOL_SEED, admin.key().as_ref(), pool_name.as_bytes()],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    /// Pool vault PDA: `["vault", pool]`; authority is itself for outbound CPIs.
    #[account(
        init,
        payer = admin,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = vault,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// SPL mint all members must use for entry fees, savings, and loans.
    pub token_mint: Account<'info, Mint>,

    /// System program for account creation.
    pub system_program: Program<'info, System>,

    /// SPL Token program for vault initialization.
    pub token_program: Program<'info, Token>,

    /// Rent sysvar for minimum balance calculations.
    pub rent: Sysvar<'info, Rent>,
}
