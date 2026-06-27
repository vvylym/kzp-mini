//! Accounts for [`crate::settle_default`].

use anchor_lang::prelude::*;

use crate::error::PoolError;
use crate::operations::{release_active_guarantee, split_outstanding_50_50};
use crate::state::{Loan, Member, Pool};
use crate::utils::seeds::MEMBER_SEED;

/// Accounts required for permissionless default settlement with 50/50 guarantor coverage.
#[derive(Accounts)]
pub struct SettleDefault<'info> {
    /// Any signer may crank a due default settlement.
    pub crank: Signer<'info>,

    /// Pool whose counters are updated.
    #[account(
        mut,
        constraint = pool.key() == loan.pool,
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// Active loan being marked defaulted.
    #[account(
        mut,
        close = borrower,
        constraint = loan.status == crate::state::LoanStatus::Active @ PoolError::LoanNotActive,
    )]
    pub loan: Box<Account<'info, Loan>>,

    /// Borrower wallet receives closed loan account rent.
    #[account(
        mut,
        constraint = borrower.key() == loan.borrower @ PoolError::NotLoanBorrower,
    )]
    pub borrower: SystemAccount<'info>,

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

/// Marks a due active loan defaulted from reserved guarantor savings.
pub fn handle_settle_default(ctx: Context<SettleDefault>) -> Result<()> {
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
    let loan_key = loan.key();
    release_active_guarantee(guarantor_a, loan.guarantor_a_locked_savings)?;
    release_active_guarantee(guarantor_b, loan.guarantor_b_locked_savings)?;

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
    loan.guarantor_a_locked_savings = 0;
    loan.guarantor_b_locked_savings = 0;
    loan.status = crate::state::LoanStatus::Defaulted;

    if ctx.accounts.borrower_member.active_loan == Some(loan_key) {
        ctx.accounts.borrower_member.active_loan = None;
    }

    Ok(())
}

fn validate_guarantor_default_coverage(
    outstanding: u64,
    guarantor_a_savings: u64,
    guarantor_b_savings: u64,
) -> std::result::Result<(u64, u64), PoolError> {
    let (share_a, share_b) = split_outstanding_50_50(outstanding);
    if guarantor_a_savings < share_a || guarantor_b_savings < share_b {
        return Err(PoolError::GuarantorInsufficientSavings);
    }
    Ok((share_a, share_b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_check_uses_shared_split() {
        assert_eq!(
            validate_guarantor_default_coverage(1_001, 501, 500).unwrap(),
            (501, 500)
        );
        assert_eq!(
            validate_guarantor_default_coverage(1_001, 500, 500).unwrap_err(),
            PoolError::GuarantorInsufficientSavings
        );
    }
}
