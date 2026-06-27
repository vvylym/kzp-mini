//! Accounts for [`crate::join_pool`].

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

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

/// Transfers entry fee to vault and initializes member state.
pub fn handle(ctx: Context<JoinPool>, entry_fee: u64) -> Result<()> {
    let pool = &ctx.accounts.pool;
    require!(
        entry_fee == pool.required_entry_fee,
        PoolError::InvalidEntryFeeAmount
    );

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            Transfer {
                from: ctx.accounts.member_token_account.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.member.to_account_info(),
            },
        ),
        entry_fee,
    )?;

    let member = &mut ctx.accounts.member_account;
    member.pool = pool.key();
    member.member_id = pool.total_members;
    member.owner = ctx.accounts.member.key();
    member.entry_fee_paid = entry_fee;
    member.savings_balance = 0;
    member.locked_savings = 0;
    member.active_loan = None;
    member.pending_loan = None;
    member.active_guarantees = Vec::new();
    member.pending_guarantees = Vec::new();
    member.bump = ctx.bumps.member_account;

    let pool = &mut ctx.accounts.pool;
    pool.total_members = pool
        .total_members
        .checked_add(1)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
