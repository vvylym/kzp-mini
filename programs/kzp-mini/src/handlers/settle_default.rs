//! Handler for [`crate::settle_default`].

use anchor_lang::prelude::*;

use crate::error::PoolError;
use crate::instructions::SettleDefault;
use crate::operations::{
    release_savings, split_outstanding_50_50, validate_guarantor_default_coverage,
};
use crate::state::LoanStatus;

/// Admin marks a due active loan defaulted from reserved guarantor savings.
pub fn handle(ctx: Context<SettleDefault>) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    let now = Clock::get()?.unix_timestamp;
    require!(now >= loan.due_ts, PoolError::LoanNotDue);

    let outstanding = loan.outstanding;
    let (share_a, share_b) = validate_guarantor_default_coverage(
        outstanding,
        ctx.accounts.guarantor_a_member.savings_balance,
        ctx.accounts.guarantor_b_member.savings_balance,
    )?;

    let guarantor_a = &mut ctx.accounts.guarantor_a_member;
    let guarantor_b = &mut ctx.accounts.guarantor_b_member;
    guarantor_a.savings_balance = guarantor_a
        .savings_balance
        .checked_sub(share_a)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    guarantor_b.savings_balance = guarantor_b
        .savings_balance
        .checked_sub(share_b)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let (locked_share_a, locked_share_b) = split_outstanding_50_50(loan.principal);
    release_savings(guarantor_a, locked_share_a)?;
    release_savings(guarantor_b, locked_share_b)?;

    let pool = &mut ctx.accounts.pool;
    pool.total_savings = pool
        .total_savings
        .checked_sub(outstanding)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    pool.total_outstanding_loans = pool
        .total_outstanding_loans
        .checked_sub(outstanding)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    loan.outstanding = 0;
    loan.status = LoanStatus::Defaulted;

    let loan_key = loan.key();
    if ctx.accounts.borrower_member.active_loan == Some(loan_key) {
        ctx.accounts.borrower_member.active_loan = None;
    }
    ctx.accounts
        .guarantor_a_member
        .active_guarantees
        .retain(|g| g != &loan_key);
    ctx.accounts
        .guarantor_b_member
        .active_guarantees
        .retain(|g| g != &loan_key);

    Ok(())
}
