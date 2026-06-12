//! Accounts for [`crate::join_pool`].

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::error::PoolError;
use crate::state::{Member, Pool};
use crate::utils::seeds::{MEMBER_SEED, VAULT_SEED};

/// Accounts required to pay the entry fee and open a member PDA.
#[derive(Accounts)]
pub struct JoinPool<'info> {
    /// New member wallet; pays entry fee and member account rent.
    #[account(mut)]
    pub member: Signer<'info>,

    /// Member state PDA: `["member", pool, member]`.
    #[account(
        init,
        payer = member,
        space = Member::LEN,
        seeds = [MEMBER_SEED, pool.key().as_ref(), member.key().as_ref()],
        bump,
    )]
    pub member_account: Account<'info, Member>,

    /// Target pool to join.
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Pool vault receiving the entry fee.
    #[account(
        mut,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump = pool.vault_bump,
        constraint = vault.key() == pool.vault @ PoolError::InvalidVaultAccount,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// Member's SPL token account (mint must match `pool.token_mint`).
    #[account(
        mut,
        constraint = member_token_account.mint == pool.token_mint,
        constraint = member_token_account.owner == member.key(),
    )]
    pub member_token_account: Account<'info, TokenAccount>,

    /// SPL Token program.
    pub token_program: Program<'info, Token>,

    /// System program for member account creation.
    pub system_program: Program<'info, System>,
}
