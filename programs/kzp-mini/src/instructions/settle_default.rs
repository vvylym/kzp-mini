//! Accounts for [`crate::settle_default`].

use anchor_lang::prelude::*;

use crate::error::PoolError;
use crate::operations::{
    release_active_guarantee, split_outstanding_50_50, validate_guarantor_default_coverage,
};
use crate::state::{Loan, Member, Pool};
use crate::utils::seeds::MEMBER_SEED;

/// Accounts required for admin default settlement with 50/50 guarantor coverage.
#[derive(Accounts)]
pub struct SettleDefault<'info> {
    /// Pool admin wallet (must match `pool.admin`; initiates settlement).
    #[account(
        constraint = admin.key() == pool.admin @ PoolError::NotPoolAdmin,
    )]
    pub admin: Signer<'info>,

    /// Pool whose counters are updated.
    #[account(
        mut,
        constraint = pool.key() == loan.pool,
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// Active loan being marked defaulted.
    #[account(
        mut,
        constraint = loan.status == crate::state::LoanStatus::Active @ PoolError::LoanNotActive,
    )]
    pub loan: Box<Account<'info, Loan>>,

    /// Borrower's member account (`active_loan` cleared).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.borrower.as_ref()],
        bump = borrower_member.bump,
        constraint = borrower_member.owner == loan.borrower,
        constraint = borrower_member.pool == loan.pool,
    )]
    pub borrower_member: Box<Account<'info, Member>>,

    /// Guarantor A member account (savings debited and guarantee cleared).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_a.as_ref()],
        bump = guarantor_a_member.bump,
        constraint = guarantor_a_member.owner == loan.guarantor_a,
        constraint = guarantor_a_member.pool == loan.pool,
    )]
    pub guarantor_a_member: Box<Account<'info, Member>>,

    /// Guarantor B member account (savings debited and guarantee cleared).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_b.as_ref()],
        bump = guarantor_b_member.bump,
        constraint = guarantor_b_member.owner == loan.guarantor_b,
        constraint = guarantor_b_member.pool == loan.pool,
    )]
    pub guarantor_b_member: Box<Account<'info, Member>>,
}

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
    let loan_key = loan.key();
    release_active_guarantee(guarantor_a, &loan_key, locked_share_a)?;
    release_active_guarantee(guarantor_b, &loan_key, locked_share_b)?;

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
    loan.status = crate::state::LoanStatus::Defaulted;

    if ctx.accounts.borrower_member.active_loan == Some(loan_key) {
        ctx.accounts.borrower_member.active_loan = None;
    }

    Ok(())
}
