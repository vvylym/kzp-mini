//! Accounts for [`crate::deposit_savings`].

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::error::PoolError;
use crate::state::{Member, Pool};
use crate::utils::seeds::{MEMBER_SEED, VAULT_SEED};

/// Accounts required to deposit SPL tokens into the pool vault.
#[derive(Accounts)]
pub struct DepositSavings<'info> {
    /// Depositing member wallet.
    #[account(mut)]
    pub member: Signer<'info>,

    /// Member state PDA whose `savings_balance` is credited.
    #[account(
        mut,
        seeds = [MEMBER_SEED, pool.key().as_ref(), member.key().as_ref()],
        bump = member_account.bump,
        constraint = member_account.owner == member.key(),
        constraint = member_account.pool == pool.key(),
    )]
    pub member_account: Account<'info, Member>,

    /// Pool whose aggregate `total_savings` is updated.
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Pool vault receiving the deposit.
    #[account(
        mut,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump = pool.vault_bump,
        constraint = vault.key() == pool.vault @ PoolError::InvalidVaultAccount,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// Member's SPL token account debited for the deposit.
    #[account(
        mut,
        constraint = member_token_account.mint == pool.token_mint,
        constraint = member_token_account.owner == member.key(),
    )]
    pub member_token_account: Account<'info, TokenAccount>,

    /// SPL Token program.
    pub token_program: Program<'info, Token>,
}
