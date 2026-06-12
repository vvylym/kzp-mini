//! Handler for [`crate::deposit_savings`].

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::instructions::DepositSavings;
use crate::operations::validate_positive_amount;

/// Transfers tokens to the vault and credits member and pool savings ledgers.
pub fn handle(ctx: Context<DepositSavings>, amount: u64) -> Result<()> {
    validate_positive_amount(amount)?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            Transfer {
                from: ctx.accounts.member_token_account.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.member.to_account_info(),
            },
        ),
        amount,
    )?;

    let member = &mut ctx.accounts.member_account;
    member.savings_balance = member
        .savings_balance
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let pool = &mut ctx.accounts.pool;
    pool.total_savings = pool
        .total_savings
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
