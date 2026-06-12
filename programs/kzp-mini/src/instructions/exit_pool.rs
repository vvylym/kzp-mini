//! Accounts for [`crate::exit_pool`].

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::error::PoolError;
use crate::state::{Member, Pool};
use crate::utils::seeds::{MEMBER_SEED, VAULT_SEED};

/// Accounts required to withdraw savings and close a member account.
#[derive(Accounts)]
pub struct ExitPool<'info> {
    /// Exiting member wallet; receives closed account rent and savings payout.
    #[account(mut)]
    pub member: Signer<'info>,

    /// Member state PDA closed after payout.
    #[account(
        mut,
        close = member,
        seeds = [MEMBER_SEED, pool.key().as_ref(), member.key().as_ref()],
        bump = member_account.bump,
        constraint = member_account.owner == member.key(),
        constraint = member_account.pool == pool.key(),
    )]
    pub member_account: Account<'info, Member>,

    /// Pool whose `total_members` and `total_savings` are decremented.
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Pool vault debited for the savings payout.
    #[account(
        mut,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump = pool.vault_bump,
        constraint = vault.key() == pool.vault @ PoolError::InvalidVaultAccount,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// Member's token account receiving the savings payout.
    #[account(
        mut,
        constraint = member_token_account.mint == pool.token_mint,
        constraint = member_token_account.owner == member.key(),
    )]
    pub member_token_account: Account<'info, TokenAccount>,

    /// SPL Token program.
    pub token_program: Program<'info, Token>,
}
