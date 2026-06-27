//! Accounts for [`crate::request_loan`].

use anchor_lang::prelude::*;

use crate::error::PoolError;
use crate::state::{Loan, Member, Pool};
use crate::utils::seeds::{LOAN_SEED, MEMBER_SEED};

/// Accounts required to open a pending loan with two nominated guarantors.
#[derive(Accounts)]
#[instruction(loan_nonce: u64)]
pub struct RequestLoan<'info> {
    /// Borrower wallet; pays loan account rent and must be a pool member.
    #[account(mut)]
    pub borrower: Signer<'info>,

    /// Borrower's member account used for limit checks.
    #[account(
        mut,
        seeds = [MEMBER_SEED, pool.key().as_ref(), borrower.key().as_ref()],
        bump = member_account.bump,
        constraint = member_account.owner == borrower.key(),
        constraint = member_account.pool == pool.key(),
    )]
    pub member_account: Account<'info, Member>,

    /// Pool the loan belongs to.
    pub pool: Account<'info, Pool>,

    /// Loan state PDA: `["loan", pool, borrower, loan_nonce_le]`.
    #[account(
        init,
        payer = borrower,
        space = Loan::LEN,
        seeds = [
            LOAN_SEED,
            pool.key().as_ref(),
            borrower.key().as_ref(),
            &loan_nonce.to_le_bytes(),
        ],
        bump,
    )]
    pub loan: Account<'info, Loan>,

    /// First guarantor's member account (must exist in this pool).
    #[account(
        seeds = [MEMBER_SEED, pool.key().as_ref(), guarantor_a.key().as_ref()],
        bump = guarantor_a_member.bump,
        constraint = guarantor_a_member.owner == guarantor_a.key() @ PoolError::GuarantorNotMember,
        constraint = guarantor_a_member.pool == pool.key() @ PoolError::GuarantorNotMember,
    )]
    pub guarantor_a_member: Account<'info, Member>,

    /// First guarantor wallet pubkey (used for PDA derivation only).
    /// CHECK: validated via `guarantor_a_member`.
    pub guarantor_a: UncheckedAccount<'info>,

    /// Second guarantor's member account (must exist in this pool).
    #[account(
        seeds = [MEMBER_SEED, pool.key().as_ref(), guarantor_b.key().as_ref()],
        bump = guarantor_b_member.bump,
        constraint = guarantor_b_member.owner == guarantor_b.key() @ PoolError::GuarantorNotMember,
        constraint = guarantor_b_member.pool == pool.key() @ PoolError::GuarantorNotMember,
    )]
    pub guarantor_b_member: Account<'info, Member>,

    /// Second guarantor wallet pubkey (used for PDA derivation only).
    /// CHECK: validated via `guarantor_b_member`.
    pub guarantor_b: UncheckedAccount<'info>,

    /// System program for loan account creation.
    pub system_program: Program<'info, System>,
}
