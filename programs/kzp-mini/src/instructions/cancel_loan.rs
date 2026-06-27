//! Accounts for [`crate::cancel_loan`].

use anchor_lang::prelude::*;

use crate::error::PoolError;
use crate::operations::clear_pending_guarantee;
use crate::state::{Loan, Member};
use crate::utils::seeds::MEMBER_SEED;

/// Accounts required for a borrower to cancel a pending loan.
#[derive(Accounts)]
pub struct CancelLoan<'info> {
    /// Borrower wallet; receives closed loan account rent.
    #[account(mut)]
    pub borrower: Signer<'info>,

    /// Pending loan to close (must belong to the borrower).
    #[account(
        mut,
        close = borrower,
        constraint = loan.borrower == borrower.key() @ PoolError::NotLoanBorrower,
        constraint = loan.status == crate::state::LoanStatus::Pending @ PoolError::LoanNotPending,
    )]
    pub loan: Account<'info, Loan>,

    /// Borrower's member account (pending loan reservation cleared).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), borrower.key().as_ref()],
        bump = borrower_member.bump,
        constraint = borrower_member.owner == borrower.key(),
        constraint = borrower_member.pool == loan.pool,
    )]
    pub borrower_member: Box<Account<'info, Member>>,

    /// Guarantor A member account (pending guarantee cleared).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_a.as_ref()],
        bump = guarantor_a_member.bump,
        constraint = guarantor_a_member.owner == loan.guarantor_a,
        constraint = guarantor_a_member.pool == loan.pool,
    )]
    pub guarantor_a_member: Box<Account<'info, Member>>,

    /// Guarantor B member account (pending guarantee cleared).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_b.as_ref()],
        bump = guarantor_b_member.bump,
        constraint = guarantor_b_member.owner == loan.guarantor_b,
        constraint = guarantor_b_member.pool == loan.pool,
    )]
    pub guarantor_b_member: Box<Account<'info, Member>>,
}

/// Clears guarantor pending refs before the loan account is closed.
pub fn handle_cancel_loan(ctx: Context<CancelLoan>) -> Result<()> {
    let loan_key = ctx.accounts.loan.key();
    if ctx.accounts.borrower_member.pending_loan == Some(loan_key) {
        ctx.accounts.borrower_member.pending_loan = None;
    }
    if ctx.accounts.loan.guarantor_a_signed {
        clear_pending_guarantee(&mut ctx.accounts.guarantor_a_member)?;
    }
    if ctx.accounts.loan.guarantor_b_signed {
        clear_pending_guarantee(&mut ctx.accounts.guarantor_b_member)?;
    }
    Ok(())
}
