//! Handler for [`crate::exit_pool`].

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::instructions::ExitPool;
use crate::operations::{validate_exit_eligible, validate_vault_liquidity};
use crate::utils::seeds::VAULT_SEED;

/// Validates eligibility and vault liquidity, then pays savings and closes the member PDA.
pub fn handle(ctx: Context<ExitPool>) -> Result<()> {
    let member = &ctx.accounts.member_account;
    validate_exit_eligible(
        member.active_loan,
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
