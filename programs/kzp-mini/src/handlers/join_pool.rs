//! Handler for [`crate::join_pool`].

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::error::PoolError;
use crate::instructions::JoinPool;

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
    member.active_loan = None;
    member.active_guarantees = Vec::new();
    member.pending_guarantees = Vec::new();
    member.bump = ctx.bumps.member_account;

    let pool = &mut ctx.accounts.pool;
    pool.total_members += 1;

    Ok(())
}
