//! Accounts for [`crate::settle_default`].

use anchor_lang::prelude::*;

use crate::error::PoolError;
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
    )]
    pub borrower_member: Box<Account<'info, Member>>,

    /// Guarantor A member account (savings debited and guarantee cleared).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_a.as_ref()],
        bump = guarantor_a_member.bump,
        constraint = guarantor_a_member.owner == loan.guarantor_a,
    )]
    pub guarantor_a_member: Box<Account<'info, Member>>,

    /// Guarantor B member account (savings debited and guarantee cleared).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_b.as_ref()],
        bump = guarantor_b_member.bump,
        constraint = guarantor_b_member.owner == loan.guarantor_b,
    )]
    pub guarantor_b_member: Box<Account<'info, Member>>,
}
