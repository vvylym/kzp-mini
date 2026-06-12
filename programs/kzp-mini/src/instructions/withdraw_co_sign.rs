//! Accounts for [`crate::withdraw_cosign`].

use anchor_lang::prelude::*;

use crate::state::{Loan, Member};
use crate::utils::seeds::MEMBER_SEED;

/// Accounts required for a guarantor to revoke a partial co-sign on a pending loan.
#[derive(Accounts)]
pub struct WithdrawCosign<'info> {
    /// Guarantor wallet revoking their co-sign.
    #[account(mut)]
    pub guarantor: Signer<'info>,

    /// Pending loan whose co-sign flag is cleared.
    #[account(
        mut,
        constraint = loan.status == crate::state::LoanStatus::Pending @ crate::error::PoolError::LoanNotPending,
    )]
    pub loan: Account<'info, Loan>,

    /// Guarantor A member account.
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_a.as_ref()],
        bump = guarantor_a_member.bump,
        constraint = guarantor_a_member.owner == loan.guarantor_a,
        constraint = guarantor_a_member.pool == loan.pool,
    )]
    pub guarantor_a_member: Box<Account<'info, Member>>,

    /// Guarantor B member account.
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_b.as_ref()],
        bump = guarantor_b_member.bump,
        constraint = guarantor_b_member.owner == loan.guarantor_b,
        constraint = guarantor_b_member.pool == loan.pool,
    )]
    pub guarantor_b_member: Box<Account<'info, Member>>,
}
