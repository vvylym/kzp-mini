//! Accounts for [`crate::co_sign_loan`].

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::error::PoolError;
use crate::state::{Loan, Member, Pool};
use crate::utils::seeds::{MEMBER_SEED, VAULT_SEED};

/// Accounts required for a guarantor to co-sign and optionally trigger disbursement.
#[derive(Accounts)]
pub struct CoSignLoan<'info> {
    /// Co-signing guarantor (must be `loan.guarantor_a` or `loan.guarantor_b`).
    #[account(mut)]
    pub guarantor: Signer<'info>,

    /// Pending or activating loan account.
    #[account(mut)]
    pub loan: Box<Account<'info, Loan>>,

    /// Pool whose counters and vault are updated on disbursement.
    #[account(
        mut,
        constraint = pool.key() == loan.pool,
    )]
    pub pool: Account<'info, Pool>,

    /// Member account for guarantor A.
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_a.as_ref()],
        bump = guarantor_a_member.bump,
        constraint = guarantor_a_member.owner == loan.guarantor_a,
        constraint = guarantor_a_member.pool == loan.pool,
    )]
    pub guarantor_a_member: Box<Account<'info, Member>>,

    /// Member account for guarantor B.
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_b.as_ref()],
        bump = guarantor_b_member.bump,
        constraint = guarantor_b_member.owner == loan.guarantor_b,
        constraint = guarantor_b_member.pool == loan.pool,
    )]
    pub guarantor_b_member: Box<Account<'info, Member>>,

    /// Pool vault debited when both guarantors have signed.
    #[account(
        mut,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump = pool.vault_bump,
        constraint = vault.key() == pool.vault @ PoolError::InvalidVaultAccount,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// Borrower's member account (receives `active_loan` on disbursement).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.borrower.as_ref()],
        bump = borrower_member.bump,
        constraint = borrower_member.owner == loan.borrower,
        constraint = borrower_member.pool == loan.pool,
    )]
    pub borrower_member: Box<Account<'info, Member>>,

    /// Borrower's token account receiving principal on disbursement.
    #[account(
        mut,
        constraint = borrower_token_account.mint == vault.mint,
        constraint = borrower_token_account.owner == loan.borrower,
    )]
    pub borrower_token_account: Account<'info, TokenAccount>,

    /// SPL Token program.
    pub token_program: Program<'info, Token>,
}
