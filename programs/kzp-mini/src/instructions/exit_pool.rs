//! Accounts for [`crate::exit_pool`].

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::error::PoolError;
use crate::operations::{validate_exit_eligible, validate_vault_liquidity};
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

/// Validates eligibility and vault liquidity, then pays savings and closes the member PDA.
pub fn handle(ctx: Context<ExitPool>) -> Result<()> {
    let member = &ctx.accounts.member_account;
    validate_exit_eligible(
        member.active_loan,
        member.pending_loan,
        member.locked_savings,
        &member.active_guarantees,
        &member.pending_guarantees,
    )?;

    let savings = member.savings_balance;
    validate_vault_liquidity(ctx.accounts.vault.amount, savings)?;

    let pool = &mut ctx.accounts.pool;
    pool.total_members = pool
        .total_members
        .checked_sub(1)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    pool.total_savings = pool
        .total_savings
        .checked_sub(savings)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let pool_key = pool.key();
    let vault_bump = pool.vault_bump;
    let seeds = &[VAULT_SEED, pool_key.as_ref(), &[vault_bump]];
    let signer_seeds = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.member_token_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            signer_seeds,
        ),
        savings,
    )?;

    Ok(())
}
